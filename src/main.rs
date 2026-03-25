mod cli;
mod commands;
mod config;
mod context;
mod db;
mod ingest;
mod llm;
mod observability;
mod storage;
mod workflows;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();
    let config = config::load_config();

    let result: Result<(), Box<dyn std::error::Error>> = match cli.command {
        Commands::Think {
            problem,
            stream,
            no_stream,
            repo,
            show_reasoning,
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
                show_reasoning,
            },
        )
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn std::error::Error>),

        Commands::Ingest {
            source,
            opencode_dir,
            claude_code_dir,
        } => {
            let conn = match db::init_db(&config.praxis_dir) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("error: failed to open database: {}", e);
                    std::process::exit(1);
                }
            };
            ingest::run_ingest(
                &config,
                &conn,
                source.as_deref(),
                opencode_dir.as_deref(),
                claude_code_dir.as_deref(),
            )
        }

        Commands::Runs {
            source,
            model,
            project,
            since,
            limit,
            json,
        } => commands::runs::run(
            &config,
            commands::runs::RunsOptions {
                source,
                model,
                project,
                since,
                limit,
                json,
            },
        ),

        Commands::Stats { by, since, until, json } => commands::stats::run(
            &config,
            commands::stats::StatsOptions { by, since, until, json },
        ),
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
