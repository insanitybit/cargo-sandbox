use std::collections::HashMap;

#[derive(Debug, Default, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerArgs {
    /// The hostname to use for the container, as a valid RFC 1123 hostname.
    #[serde(rename = "Hostname")]
    pub hostname: String,

    /// The domain name to use for the container.
    #[serde(rename = "Domainname")]
    pub domainname: String,

    /// The user that commands are run as inside the container.
    #[serde(rename = "User")]
    pub user: String,

    /// Whether to attach to stdin
    #[serde(rename = "AttachStdin")]
    pub attach_stdin: bool,

    /// Whether to attach to stdout
    #[serde(rename = "AttachStdout")]
    pub attach_stdout: bool,

    /// Whether to attach to stderr
    #[serde(rename = "AttachStderr")]
    pub attach_stderr: bool,

    /// Attach standard streams to a TTY, including stdin if it is not closed
    #[serde(rename = "Tty")]
    pub tty: bool,

    /// Open stdin
    #[serde(rename = "OpenStdin")]
    pub open_stdin: bool,

    /// Close stdin after one attached client disconnects
    #[serde(rename = "StdinOnce")]
    pub stdin_once: bool,

    /// A list of environment variables in the form ["VAR=value", ...]
    /// A variable without `=` is removed from the environment, rather than to have an empty value.
    #[serde(rename = "Env")]
    pub env: Vec<String>,

    /// Command to run specified as a string or an array of strings.
    #[serde(rename = "Cmd")]
    pub cmd: Vec<String>,

    #[serde(rename = "Entrypoint")]
    pub entrypoint: String,

    /// The name (or reference) of the image to use when creating the container, or which was used when the container was created.
    #[serde(rename = "Image")]
    pub image: String,

    /// User-defined key/value metadata.
    #[serde(rename = "Labels")]
    pub labels: HashMap<String, String>,

    /// An object mapping mount point paths inside the container to empty objects.
    #[serde(rename = "Volumes")]
    pub volumes: HashMap<String, HashMap<(), ()>>,

    /// A list of volume bindings for this container.
    #[serde(rename = "Binds")]
    pub binds: Vec<String>,

    /// The working directory for commands to run in.
    #[serde(rename = "WorkingDir")]
    pub working_dir: String,

    /// Disable networking for the container.
    #[serde(rename = "NetworkDisabled")]
    pub network_disabled: Option<bool>,

    /// MAC address of the container.
    #[serde(rename = "MacAddress")]
    pub mac_address: Option<String>,

    /// An object mapping ports to an empty object in the form:
    /// `{"<port>/<tcp|udp|sctp>": {}}`
    #[serde(rename = "ExposedPorts")]
    pub exposed_ports: HashMap<String, HashMap<(), ()>>,
}
