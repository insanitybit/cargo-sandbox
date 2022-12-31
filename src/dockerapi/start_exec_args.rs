#[derive(Debug, serde::Serialize)]
pub struct StartExecArgs {
    /// Detach from the command.
    #[serde(rename = "Detach")]
    pub detach: bool,
    /// Allocate a pseudo-TTY.
    #[serde(rename = "Tty")]
    pub tty: bool,
}
