use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "praxis")]
#[command(about = "Structured thinking and AI workflow execution")]
#[command(version = "0.1.0")]
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
    },
}
