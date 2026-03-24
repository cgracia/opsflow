mod cli;
mod config;
mod llm;
mod observability;
mod storage;
mod workflows;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();
    let config = config::load_config();

    let result = match cli.command {
        Commands::Think { problem } => workflows::run_think(&problem, &config),
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
