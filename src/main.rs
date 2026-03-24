mod cli;
mod config;
mod context;
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
        Commands::Think {
            problem,
            stream,
            no_stream,
            repo,
        } => workflows::run_think(
            &problem,
            &config,
            workflows::ThinkOptions {
                stream: if stream {
                    Some(true)
                } else if no_stream {
                    Some(false)
                } else {
                    None
                },
                repo_context: repo,
            },
        ),
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
