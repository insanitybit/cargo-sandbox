#![allow(dead_code, unused)]
use maplit::hashmap;

use container_type::ContainerType;
use dockerapi::client::Client;

use crate::dockerapi::container_summary::ContainerSummary;
use crate::dockerapi::create_container_args::CreateContainerArgs;
use crate::dockerapi::create_exec_args::CreateExecArgs;

mod config;
mod container;
mod container_type;
mod dockerapi;

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

async fn find_and_remove_container(
    client: &Client,
    project_name: &str,
    container_type: ContainerType,
) -> eyre::Result<()> {
    let container = find_container(client, project_name, container_type).await?;
    if let Some(container) = container {
        client
            .remove_container(container.id, false, true)
            .await?;
    }

    Ok(())
}

async fn create_container(
    client: &Client,
    project_name: &str,
    container_type: ContainerType,
    command: Vec<String>,
    network_disabled: bool,
) -> eyre::Result<ContainerSummary> {
    // let binds: Vec<String> = vec![];
    let binds = vec![format!(
        "{}/:/home/{DOCKER_USER}/{}:cached",
        std::env::current_dir()?.to_str().unwrap(),
        project_name
    )];

    let user = DOCKER_USER.into();

    client
        .create_container(CreateContainerArgs {
            cmd: command,
            // entrypoint: command.join(" "),
            image: format!("cargo-sandbox-{}", container_type.as_str()),
            labels: hashmap! {
                "cargo-sandbox.version".into() => env!("CARGO_PKG_VERSION").to_string(),
                "cargo-sandbox.project-name".into() => project_name.into(),
                "cargo-sandbox.container-type".into() => container_type.as_str().into(),
            },
            working_dir: format!("/home/{DOCKER_USER}/{project_name}"),
            binds,
            user,
            tty: false,
            network_disabled: Some(network_disabled),
            attach_stdout: true,
            attach_stderr: true,
            attach_stdin: true,
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

fn make_cargo_cmd(
    riff: bool,
    args: Vec<String>,
) -> Vec<String> {
    if riff {
        let mut cargo_cmd = vec!["riff".to_string(), "run".to_string(), "cargo".to_string()];
        cargo_cmd.extend(args);
        cargo_cmd
    } else {
        let mut cargo_cmd = vec!["cargo".to_string()];
        cargo_cmd.extend(args);
        cargo_cmd
    }
}

async fn ephemeral_exec(client: &Client, project_name: &str, cargo_command: Vec<String>, container_type: ContainerType) -> eyre::Result<()> {

    // First we should remove the container if it exists
    find_and_remove_container(client, project_name, container_type).await?;

    let build_container =
        create_container(
            client,
            project_name,
            container_type,
            cargo_command,
            false
        ).await?;

    let attach_client = client.clone();
    let container_id = build_container.id.clone();
    let attach = tokio::spawn(async move {
        println!("attaching");
        attach_client.attach(&container_id).await?;
        println!("attached");
        Ok::<(), eyre::Error>(())
    });

    println!("starting");
    start_container(client, &build_container).await?;
    println!("started");

    attach.await?;

    client.remove_container(build_container.id, true, true).await?;

    Ok(())
}

async fn cargo_build(client: &Client, project_name: &str, args: Vec<String>) -> eyre::Result<()> {
    let cargo_cmd = make_cargo_cmd(false, args);
    ephemeral_exec(client, project_name, cargo_cmd, ContainerType::Build).await?;

    Ok(())
}

async fn cargo_check(client: &Client, project_name: &str, args: Vec<String>) -> eyre::Result<()> {
    let cargo_cmd = make_cargo_cmd(false, args);
    ephemeral_exec(client, project_name, cargo_cmd, ContainerType::Build).await?;

    Ok(())
}

fn insert_after(args: &mut Vec<String>, needle: &str, insert: String) {
    let index = args.iter().position(|arg| arg == needle).unwrap();
    args.insert(index + 1, insert.to_string());
}

async fn cargo_publish(client: &Client, project_name: &str, mut args: Vec<String>) -> eyre::Result<()> {
    let cargo_cmd = make_cargo_cmd(false, args);
    // First verify the package unless we are told not to
    if !cargo_cmd.iter().any( |a| a == "--no-verify") {
        let mut cargo_cmd = cargo_cmd.clone();
        // We only want to verify, so we ensure that `dry-run` is present in the args
        if !cargo_cmd.iter().any(|a| a == "--dry-run") {
            insert_after(&mut cargo_cmd, "publish", "--dry-run".to_string());
        }
        ephemeral_exec(client, project_name, cargo_cmd, ContainerType::Build).await?;
    }

    // Second, publish the package unless we are told not to
    if !cargo_cmd.iter().any(|a| a == "--dry-run") {
        let mut cargo_cmd = cargo_cmd.clone();
        // We only want to publish, so we ensure that `no-verify` is present in the args
        if !cargo_cmd.iter().any(|a| a == "--no-verify") {
            insert_after(&mut cargo_cmd, "publish", "--very-verify".to_string());
        }
        ephemeral_exec(client, project_name, cargo_cmd, ContainerType::Publish).await?;
    }
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
        // "login" => {
        //     let project_name = get_project_name();
        //     let client = Client::local("/var/run/docker.sock");
        //     cargo_login(&client, &project_name, argv).await?;
        // }
        unknown => {
            println!("Unknown command: {unknown}");
        }
    }

    Ok(())
}
