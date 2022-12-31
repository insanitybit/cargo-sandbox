#[derive(Debug, serde::Serialize)]
pub struct CreateExecArgs {
    /// Attach to stdin of the exec command.
    #[serde(rename = "AttachStdin")]
    pub attach_stdin: bool,
    /// Attach to stdout of the exec command.
    #[serde(rename = "AttachStdout")]
    pub attach_stdout: bool,
    /// Attach to stderr of the exec command.
    #[serde(rename = "AttachStderr")]
    pub attach_stderr: bool,
    /// Escape keys for detaching a container.
    #[serde(rename = "DetachKeys")]
    pub detach_keys: String,
    /// Allocate a TTY for the exec command.
    #[serde(rename = "Tty")]
    pub tty: bool,
    /// The command to run in the container.
    #[serde(rename = "Cmd")]
    pub cmd: Vec<String>,
    /// Environment variables to set in the container for the exec command.
    #[serde(rename = "Env")]
    pub env: Vec<String>,
}
