use clap::Parser;
use maplit::hashmap;

use crate::config::{Cli, Command};
use container_type::ContainerType;
use dockerapi::client::Client;

use crate::dockerapi::container_summary::ContainerSummary;
use crate::dockerapi::create_container_args::CreateContainerArgs;
use crate::dockerapi::create_exec_args::CreateExecArgs;

pub mod config;
pub mod container;
pub mod container_type;
pub mod dockerapi;

const DOCKER_USER: &'static str = "cargo-sandbox-user";

async fn find_container(
    client: &Client,
    project_name: &str,
    container_type: ContainerType,
) -> eyre::Result<Option<ContainerSummary>> {
    let list_containers_response = client
        .list_containers(dockerapi::list_containers::ListContainersArgs {
            all_containers: true,
            limit: Some(1),
            filters: Some(
                serde_json::json!({
                    "label": [
                        format!("cargo-sandboxed.project-name={}", project_name),
                        format!("cargo-sandboxed.container-type={}", container_type.as_str()),
                    ]
                })
                .to_string(),
            ),
            ..Default::default()
        })
        .await?;

    Ok(list_containers_response.containers.into_iter().next())
}

async fn get_or_create_container(
    client: &Client,
    project_name: &str,
    container_type: ContainerType,
    network_disabled: bool,
) -> eyre::Result<ContainerSummary> {
    let container = find_container(client, project_name, container_type).await?;
    if let Some(container) = container {
        return Ok(container);
    }
    let binds = vec![format!(
        "{}/:/home/{DOCKER_USER}/{}:cached",
        std::env::current_dir()?.to_str().unwrap(),
        project_name
    )];

    // let user = format!("{}:{DOCKER_USER}", users::get_current_uid());
    let user = "".into();

    client
        .create_container(CreateContainerArgs {
            cmd: vec!["sleep".into(), "infinity".into()],
            image: format!("cargo-sandboxed-{}", container_type.as_str()),
            labels: hashmap! {
                "cargo-sandboxed.version".into() => env!("CARGO_PKG_VERSION").to_string(),
                "cargo-sandboxed.project-name".into() => project_name.into(),
                "cargo-sandboxed.container-type".into() => container_type.as_str().into(),
            },
            working_dir: format!("/home/{DOCKER_USER}/{project_name}"),
            binds,
            user,
            network_disabled: Some(network_disabled),
            ..Default::default()
        })
        .await?;

    let container = find_container(client, project_name, container_type).await?;
    if let Some(container) = container {
        return Ok(container);
    }

    panic!("This should not be possible")
}

async fn start_container(client: &Client, container: &ContainerSummary) -> eyre::Result<()> {
    match container.state.as_str() {
        state @ ("created" | "dead" | "exited" | "paused") => {
            println!("Container is {state} - starting it");
            client.start_container(&container.id).await?;
        }
        state @ "running" => {
            println!("Container is already running: {state}");
        }
        other => {
            println!("Container is in an unknown state: {other}");
        }
    }
    Ok(())
}

async fn cargo_build(client: &Client, project_name: &str) -> eyre::Result<()> {
    let build_container =
        get_or_create_container(client, project_name, ContainerType::Build, false).await?;
    start_container(client, &build_container).await?;

    println!("executing command");
    client
        .exec(
            build_container.id,
            CreateExecArgs {
                attach_stdin: true,
                attach_stdout: true,
                attach_stderr: true,
                detach_keys: "".to_string(),
                tty: false,
                cmd: vec!["cargo".into(), "build".into()],
                env: get_env(),
            },
        )
        .await?;

    Ok(())
}

async fn cargo_check(client: &Client, project_name: &str) -> eyre::Result<()> {
    let build_container =
        get_or_create_container(client, project_name, ContainerType::Build, false).await?;
    println!("{:#?}", build_container);
    start_container(client, &build_container).await?;

    println!("executing command");
    client
        .exec(
            build_container.id,
            CreateExecArgs {
                attach_stdin: true,
                attach_stdout: true,
                attach_stderr: true,
                detach_keys: "".to_string(),
                tty: false,
                cmd: vec!["cargo".into(), "check".into()],
                env: get_env(),
            },
        )
        .await?;

    Ok(())
}

async fn cargo_publish(client: &Client, project_name: &str, token: &str) -> eyre::Result<()> {
    println!("building");
    let build_container =
        get_or_create_container(client, project_name, ContainerType::Publish, true).await?;
    start_container(client, &build_container).await?;

    println!("publishing");

    client
        .exec(
            build_container.id,
            CreateExecArgs {
                attach_stdin: true,
                attach_stdout: true,
                attach_stderr: true,
                detach_keys: "".to_string(),
                tty: false,
                cmd: vec![
                    "cargo".into(),
                    "publish".into(),
                    "--token".into(),
                    token.into(),
                ],
                env: get_env(),
            },
        )
        .await?;
    Ok(())
}

fn get_env() -> Vec<String> {
    std::env::vars()
        .filter_map(|(key, value)| {
            if should_pass_key(&key) {
                Some(format!("{}={}", key, value))
            } else {
                None
            }
        })
        .collect()
}

fn should_pass_key(key: &str) -> bool {
    const EQUALS: &[&str] = &["CARGO_BUILD_JOBS", "RUST_RECURSION_COUNT"];
    EQUALS.contains(&key)
}

fn get_project_name() -> String {
    let current_dir = std::env::current_dir().unwrap();
    let project_name = current_dir
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap();
    println!("project name: {}", project_name);
    project_name.to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Build {} => {
            let client = Client::local("/var/run/docker.sock");
            cargo_build(&client, &get_project_name()).await?;
        }
        Command::Check {} => {
            let client = Client::local("/var/run/docker.sock");
            cargo_check(&client, &get_project_name()).await?;
        }
        Command::Publish { token } => {
            let client = Client::local("/var/run/docker.sock");
            cargo_publish(&client, &get_project_name(), &token).await?;
        }
    }

    Ok(())
}
