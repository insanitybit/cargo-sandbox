use std::path::Path;

use eyre::Context;
use futures::{Stream, StreamExt};
use hyper::Client as HyperClient;
use hyper::{Body, Uri};

use crate::dockerapi::container_summary::ContainerSummary;
use crate::dockerapi::create_container_args::CreateContainerArgs;
use crate::dockerapi::create_container_response::CreateContainerResponse;
use crate::dockerapi::create_exec_args::CreateExecArgs;
use crate::dockerapi::create_exec_response::CreateExecResponse;
use crate::dockerapi::list_containers::{ListContainersArgs, ListContainersResponse};
use crate::dockerapi::start_exec_args::StartExecArgs;
use crate::dockerapi::start_exec_response::StartExecResponse;
use crate::dockerapi::unix_connector::UnixSocketConnector;

#[derive(Clone)]
pub struct Client {
    inner_client: HyperClient<UnixSocketConnector, Body>,
}

impl Client {
    /// Create a new client for a local docker API.
    /// ```
    /// fn main() {
    ///     Client::local("/var/run/docker.sock")
    /// }
    /// ```
    pub fn local<P: AsRef<Path>>(path: P) -> Self {
        let connector: UnixSocketConnector = UnixSocketConnector::new(path);
        Self {
            inner_client: HyperClient::builder().build(connector),
        }
    }

    pub async fn attach(&self, container_id: &str) -> eyre::Result<()> {
        let client = &self.inner_client;
        let uri = format!("http://localhost/containers/{}/attach?stream=1&stdout=1&stdin=1&stderr=1", container_id).parse::<Uri>()?;

        let request = hyper::Request::post(uri)
            // .header("Content-Type", "application/json")
            .body(Body::empty())?;

        let res = client.request(request).await?;

        let mut body = res.into_body();
        print_docker_encoded_stream(body).await?;

        Ok(())
    }

    pub async fn create_container(
        &self,
        args: CreateContainerArgs,
    ) -> eyre::Result<CreateContainerResponse> {
        let client = &self.inner_client;
        let uri = Uri::from_static("http://localhost/containers/create");

        let request = hyper::Request::post(uri)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&args)?))?;

        let res = client.request(request).await?;

        let body = read_body_to_vec(res).await?;

        Ok(serde_json::from_slice(&body)?)
    }

    pub async fn remove_container(
        &self,
        container_id: String,
        force: bool,
        remove_anonymouse_volumes: bool,
    ) -> eyre::Result<()> {

        let client = &self.inner_client;
        let uri = format!("http://localhost/containers/{}", container_id).parse::<Uri>()?;

        let request = hyper::Request::delete(uri)
            .header("Content-Type", "application/json")
            .body(
            Body::from(serde_json::to_vec(&serde_json::json!({
                "force": force,
                "remove_anonymouse_volumes": remove_anonymouse_volumes,
            })).unwrap())
        )?;

        let res = client.request(request).await?;
        Ok(())
    }

    pub async fn start_container(&self, container_id: &str) -> eyre::Result<()> {
        let client = &self.inner_client;
        let uri = format!("http://localhost/containers/{}/start", container_id).parse::<Uri>()?;

        let request = hyper::Request::post(uri).body(Body::empty())?;

        let res = client.request(request).await?;
        if !res.status().is_success() {
            let body = read_body_to_vec(res).await?;
            // todo: This is a json response with a `message` field
            eyre::bail!("start_container: {:?}", String::from_utf8_lossy(&body));
        }

        Ok(())
    }

    pub async fn stop_container(&self, container_id: &str) -> eyre::Result<()> {
        let client = &self.inner_client;
        let uri = format!("http://localhost/containers/{}/kill", container_id).parse::<Uri>()?;

        let request = hyper::Request::post(uri)
            .header("Content-Type", "application/json")
            .body(Body::empty())?;

        let res = client.request(request).await?;

        let body = read_body_to_vec(res).await?;
        // println!("{}", serde_json::from_slice::<serde_json::Value>(&body)?);

        Ok(serde_json::from_slice(&body)?)
    }

    pub async fn wait(&self, container_id: String) -> eyre::Result<()> {
        let client = &self.inner_client;
        let uri = format!("http://localhost/containers/{}/wait", container_id).parse::<Uri>()?;

        let request = hyper::Request::post(uri).body(Body::empty())?;

        let res = client.request(request).await?;

        let body = read_body_to_vec(res).await?;
        // println!("{}", serde_json::from_slice::<serde_json::Value>(&body)?);

        Ok(())
    }

    pub async fn exec(&self, container_id: String, args: CreateExecArgs) -> eyre::Result<()> {
        let exec_id = self.create_exec(container_id, args).await?;
        self.start_exec(
            &exec_id.id,
            StartExecArgs {
                detach: false,
                tty: false,
            },
        )
        .await?;
        Ok(())
    }

    pub async fn create_exec(
        &self,
        container_id: String,
        args: CreateExecArgs,
    ) -> eyre::Result<CreateExecResponse> {
        let client = &self.inner_client;
        let uri = format!("http://localhost/containers/{}/exec", container_id).parse::<Uri>()?;

        let request = hyper::Request::post(uri)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&args)?))?;

        let res = client.request(request).await?;

        let body = read_body_to_vec(res).await?;

        // println!(
        //     "create_exec {}",
        //     serde_json::from_slice::<serde_json::Value>(&body)?
        // );
        Ok(serde_json::from_slice(&body)?)
    }

    pub async fn start_exec(
        &self,
        exec_id: &str,
        args: StartExecArgs,
    ) -> eyre::Result<StartExecResponse> {
        let client = &self.inner_client;
        let uri = format!("http://localhost/exec/{}/start", exec_id).parse::<Uri>()?;

        let request = hyper::Request::post(uri)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&args)?))?;

        let res = client.request(request).await?;

        let mut body = res.into_body();
        print!("\n");
        while let Some(chunk) = body.next().await {
            let chunk = chunk?;
            if chunk.is_empty() {
                continue;
            }
            print!("{}", String::from_utf8_lossy(&chunk));
        }

        Ok(StartExecResponse {})
    }

    pub async fn list_containers(
        &self,
        args: ListContainersArgs,
    ) -> eyre::Result<ListContainersResponse> {
        let client = &self.inner_client;

        let args = serde_url_params::to_string(&args)?;
        // println!("\nargs= {}\n", args);
        let uri: Uri = format!("http://localhost/containers/json?{}", args).parse()?;
        let res = client.get(uri).await?;

        let body = read_body_to_vec(res).await?;
        // println!("{}", serde_json::from_slice::<serde_json::Value>(&body)?);
        let containers: Vec<ContainerSummary> =
            serde_json::from_slice(&body).context("ListContainersResponse")?;
        Ok(ListContainersResponse { containers })
    }
}

fn body_size_hint(body: &Body) -> usize {
    let hint = body.size_hint();
    hint.1.unwrap_or(std::cmp::max(hint.0, 16))
}

async fn read_body_to_vec(res: hyper::Response<Body>) -> eyre::Result<Vec<u8>> {
    let mut res = res.into_body();
    let hint = body_size_hint(&res);

    let mut body = Vec::with_capacity(hint);
    while let Some(chunk) = res.next().await {
        body.extend_from_slice(&chunk?);
    }
    Ok(body)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
enum StreamType {
    Stdout = 0,
    Stderr = 1,
}

#[derive(Debug, thiserror::Error)]
enum StreamTypeError {
    #[error("invalid byte: {0}")]
    InvalidByte(u8),
}

impl TryFrom<u8> for StreamType {
    type Error = StreamTypeError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0 => Ok(StreamType::Stdout),
            1 => Ok(StreamType::Stdout),
            2 => Ok(StreamType::Stderr),
            _ => Err(StreamTypeError::InvalidByte(byte)),
        }
    }
}


// Reads and prints out the docker encoded stream:
// https://docs.docker.com/engine/api/v1.41/#operation/ContainerAttach
//
// The header contains the information which the stream writes (stdout or stderr). It also contains the size of the associated frame encoded in the last four bytes (uint32).
//
// It is encoded on the first eight bytes like this:
//
// header := [8]byte{STREAM_TYPE, 0, 0, 0, SIZE1, SIZE2, SIZE3, SIZE4}
// STREAM_TYPE can be:
//
// 0: stdin (is written on stdout)
// 1: stdout
// 2: stderr
// SIZE1, SIZE2, SIZE3, SIZE4 are the four bytes of the uint32 size encoded as big endian.
//
// Following the header is the payload, which is the specified number of bytes of STREAM_TYPE.
//
// The simplest way to implement this protocol is the following:
//
// Read 8 bytes.
// Choose stdout or stderr depending on the first byte.
// Extract the frame size from the last four bytes.
// Read the extracted size and output it on the correct output.
// Goto 1.
// #[tracing::instrument(skip(body), err)]
async fn print_docker_encoded_stream<>(body: Body) -> eyre::Result<()> {
    let mut body = body;
    let mut buf = Vec::with_capacity(128);
    // let mut frame = Vec::with_capacity(64);

    let mut header_is_decoded = false;
    let mut stream_type = StreamType::Stdout;
    let mut size = 0;

    while let Some(chunk) = body.next().await {
        buf.extend_from_slice(&chunk?);
        // If we have enough bytes for a header, decode the header
        if !header_is_decoded && buf.len() >= 8 {
            stream_type = StreamType::try_from(buf[0])?;
            size = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;

            header_is_decoded = true;

            // Eat the header
            buf.drain(..8);
        }

        // If we don't have enough bytes for the frame, keep reading
        if buf.len() < size {
            continue;
        }

        // If we have enough bytes for the frame, print it and reset the state
        if header_is_decoded {
            header_is_decoded = false;
            match stream_type {
                StreamType::Stdout => print!("{}", String::from_utf8_lossy(&buf[..size])),
                StreamType::Stderr => eprint!("{}", String::from_utf8_lossy(&buf[..size])),
            }
            buf.drain(..size);
        }
    }
    Ok(())
}