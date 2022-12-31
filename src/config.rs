use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// alias for `cargo check`
    Check {},
    /// alias for `cargo build`
    Build {},
    /// alias for `cargo publish`
    Publish {
        /// cargo publish --token
        #[clap(short, long)]
        token: String, // todo: use a Secret crate
    },
}
