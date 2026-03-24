use clap::{ArgAction, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "praxis")]
#[command(about = "Structured thinking and AI workflow execution")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the structured thinking workflow on a problem
    Think {
        /// The problem to think through
        problem: String,

        /// Force streaming output even when stdout is not a terminal
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "no_stream")]
        stream: bool,

        /// Disable streaming output
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "stream")]
        no_stream: bool,

        /// Read local repository context from the current working directory
        #[arg(long, action = ArgAction::SetTrue)]
        repo: bool,
    },
}
