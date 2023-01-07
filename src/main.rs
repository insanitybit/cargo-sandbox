use maplit::hashmap;

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
                        format!("cargo-sandbox.project-name={}", project_name),
                        format!("cargo-sandbox.container-type={}", container_type.as_str()),
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

    let user = DOCKER_USER.into();

    client
        .create_container(CreateContainerArgs {
            cmd: vec!["sleep".into(), "infinity".into()],
            image: format!("cargo-sandbox-{}", container_type.as_str()),
            labels: hashmap! {
                "cargo-sandbox.version".into() => env!("CARGO_PKG_VERSION").to_string(),
                "cargo-sandbox.project-name".into() => project_name.into(),
                "cargo-sandbox.container-type".into() => container_type.as_str().into(),
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

async fn cargo_build(client: &Client, project_name: &str, args: Vec<String>) -> eyre::Result<()> {
    let build_container =
        get_or_create_container(client, project_name, ContainerType::Build, false).await?;
    start_container(client, &build_container).await?;

    let cargo_cmd = "source ~/.profile && riff run cargo ".to_owned() + &args.join(" ");
    let cmd = vec!["/bin/bash".into(), "-c".into(), cargo_cmd];

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
                cmd,
                env: get_env(),
            },
        )
        .await?;

    Ok(())
}

async fn cargo_check(client: &Client, project_name: &str, args: Vec<String>) -> eyre::Result<()> {
    let build_container =
        get_or_create_container(client, project_name, ContainerType::Build, false).await?;
    start_container(client, &build_container).await?;

    let cargo_cmd = "source ~/.profile && riff run cargo ".to_owned() + &args.join(" ");
    let cmd = vec!["/bin/bash".into(), "-c".into(), cargo_cmd];

    println!("executing command, {:#?}", &cmd);
    println!("container id {}", &build_container.id);

    client
        .exec(
            build_container.id,
            CreateExecArgs {
                attach_stdin: true,
                attach_stdout: true,
                attach_stderr: true,
                detach_keys: "".to_string(),
                tty: false,
                cmd,
                env: get_env(),
            },
        )
        .await?;

    Ok(())
}

async fn cargo_publish(client: &Client, project_name: &str, args: Vec<String>) -> eyre::Result<()> {
    println!("building");
    // If `--no-verify` is provided, don't run the build
    let mut publish_args = args.clone();
    if !args.contains(&"--no-verify".to_string()) {
        cargo_check(client, project_name, vec!["check".into()]).await?;
        publish_args.insert(0, "--no-verify".to_string());
    }

    println!("publishing");
    let build_container =
        get_or_create_container(client, project_name, ContainerType::Publish, false).await?;
    start_container(client, &build_container).await?;
    // Ensure that `--no-verify` is passed to cargo publish
    client
        .exec(
            build_container.id,
            CreateExecArgs {
                attach_stdin: true,
                attach_stdout: true,
                attach_stderr: true,
                detach_keys: "".to_string(),
                tty: false,
                cmd: [vec!["cargo".into()], publish_args].concat(),
                env: get_env(),
            },
        )
        .await?;
    Ok(())
}


async fn cargo_login(client: &Client, project_name: &str, args: Vec<String>) -> eyre::Result<()> {
    println!("building");
    let build_container =
        get_or_create_container(client, project_name, ContainerType::Publish, true).await?;
    start_container(client, &build_container).await?;

    println!("login");

    client
        .exec(
            build_container.id,
            CreateExecArgs {
                attach_stdin: true,
                attach_stdout: true,
                attach_stderr: true,
                detach_keys: "".to_string(),
                tty: false,
                cmd: [vec!["cargo".into()], args].concat(),
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
    let mut args: Vec<String> = std::env::args().collect();
    println!("{:?}", args);

    let argv = args.split_off(1);
    let _argc = args;
    println!("{_argc:?} - {argv:?}");

    match argv[0].as_ref() {
        "build" => {
            let project_name = get_project_name();
            let client = Client::local("/var/run/docker.sock");
            cargo_build(&client, &project_name, argv).await?;
        }
        "check" => {
            let project_name = get_project_name();
            let client = Client::local("/var/run/docker.sock");
            cargo_check(&client, &project_name, argv).await?;
        }
        "publish" => {
            let project_name = get_project_name();
            let client = Client::local("/var/run/docker.sock");
            cargo_publish(&client, &project_name, argv).await?;
        }
        "login" => {
            let project_name = get_project_name();
            let client = Client::local("/var/run/docker.sock");
            cargo_login(&client, &project_name, argv).await?;
        }
        unknown => {
            println!("Unknown command: {unknown}");
        }
    }

    Ok(())
}
