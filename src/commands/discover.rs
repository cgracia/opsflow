use colored::Colorize;

use crate::config::PraxisConfig;
use crate::registry::discover;

pub fn run(config: &PraxisConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Scanning for workflows…".dimmed());

    let discovered = discover::run_discover(&config.praxis_dir)?;

    if discovered.is_empty() {
        println!("No workflows discovered.");
        println!(
            "Tip: place OpenCode commands in {} or {}.",
            ".opencode/commands/".cyan(),
            "~/.config/opencode/commands/".cyan()
        );
        return Ok(());
    }

    println!(
        "\n{}\n",
        format!("Discovered {} workflows:", discovered.len()).bold()
    );

    for dw in &discovered {
        let source = &dw.workflow.source_command;
        let icon = match source.as_str() {
            "opencode" => "◆",
            "systemd" => "⚙",
            "script" => "▶",
            _ => "•",
        };
        println!(
            "  {} {} [{}]  —  {}",
            icon.cyan(),
            dw.name.bold(),
            source,
            dw.workflow.description
        );
    }

    println!();
    let target = config.praxis_dir.join("workflows.toml");
    if target.exists() {
        println!(
            "{}",
            "Existing workflows.toml found — wrote draft to workflows.draft.toml.".yellow()
        );
        println!("Review the draft and merge entries manually.");
    } else {
        println!(
            "Wrote {} to {}.",
            "workflows.toml".cyan(),
            config.praxis_dir.display()
        );
        println!(
            "Next: edit {} to fill in reads_from, writes_to, tags, and expected_frequency_hours.",
            config.praxis_dir.join("workflows.toml").display()
        );
    }

    Ok(())
}
