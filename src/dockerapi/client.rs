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

        // println!("{}", serde_json::from_slice::<serde_json::Value>(&body)?);
        Ok(serde_json::from_slice(&body)?)
    }

    pub async fn start_container(&self, container_id: &str) -> eyre::Result<()> {
        let client = &self.inner_client;
        let uri = format!("http://localhost/containers/{}/start", container_id).parse::<Uri>()?;

        let request = hyper::Request::post(uri).body(Body::empty())?;

        let res = client.request(request).await?;

        let _body = read_body_to_vec(res).await?;
        // body should be empty
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

        println!(
            "create_exec {}",
            serde_json::from_slice::<serde_json::Value>(&body)?
        );
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
