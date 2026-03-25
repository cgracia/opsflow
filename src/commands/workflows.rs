use colored::Colorize;

use crate::config::PraxisConfig;
use crate::db;
use crate::registry;

pub struct WorkflowsOptions {
    pub show_status: bool,
    pub tag: Option<String>,
    pub stale_only: bool,
    pub show_name: Option<String>,
}

pub fn run(
    config: &PraxisConfig,
    opts: WorkflowsOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ref name) = opts.show_name {
        return show_one(config, name);
    }

    let mut entries = registry::load_workflows(&config.praxis_dir)?;

    if entries.is_empty() {
        println!("No workflows registered.");
        println!("Run {} to auto-discover workflows.", "praxis discover".cyan());
        return Ok(());
    }

    // Enrich with staleness if requested
    if opts.show_status || opts.stale_only {
        let conn = db::init_db(&config.praxis_dir)?;
        registry::check_staleness(&mut entries, &conn)?;
    }

    // Apply tag filter
    if let Some(ref tag) = opts.tag {
        entries.retain(|e| {
            e.workflow
                .tags
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .any(|t| t == tag)
        });
    }

    // Apply stale filter
    if opts.stale_only {
        entries.retain(|e| e.stale);
    }

    if entries.is_empty() {
        println!("No workflows match the given filters.");
        return Ok(());
    }

    if opts.show_status || opts.stale_only {
        print_with_status(&entries);
    } else {
        print_list(&entries);
    }

    Ok(())
}

fn show_one(config: &PraxisConfig, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries = registry::load_workflows(&config.praxis_dir)?;
    let conn = db::init_db(&config.praxis_dir)?;
    registry::check_staleness(&mut entries, &conn)?;

    let entry = entries
        .iter()
        .find(|e| e.name == name)
        .ok_or_else(|| format!("workflow '{}' not found", name))?;

    let w = &entry.workflow;

    println!("{}", format!("Workflow: {}", entry.name).bold());
    println!("  Description:  {}", w.description);
    println!("  Trigger:      {}", w.trigger);
    if let Some(ref sched) = w.schedule {
        println!("  Schedule:     {}", sched);
    }
    println!("  Source:       {}", w.source_command);
    println!("  Command path: {}", w.command_path);
    if let Some(ref reads) = w.reads_from {
        println!("  Reads from:   {}", reads.join(", "));
    }
    if let Some(ref writes) = w.writes_to {
        println!("  Writes to:    {}", writes.join(", "));
    }
    if let Some(ref tags) = w.tags {
        println!("  Tags:         {}", tags.join(", "));
    }
    if let Some(freq) = w.expected_frequency_hours {
        println!("  Frequency:    every {}h", freq);
    }
    println!("  Enabled:      {}", w.is_enabled());

    if let Some(ts) = entry.last_output_at {
        println!("  Last output:  {}", ts.format("%Y-%m-%d %H:%M UTC"));
    } else {
        println!("  Last output:  never");
    }

    if let Some(_) = w.expected_frequency_hours {
        if entry.stale {
            println!("  Health:       {}", "STALE".red().bold());
        } else if entry.last_output_at.is_some() {
            println!("  Health:       {}", "OK".green());
        } else {
            println!("  Health:       {}", "NEVER RAN".yellow());
        }
    }

    Ok(())
}

fn print_list(entries: &[registry::WorkflowEntry]) {
    let total = entries.len();
    let enabled = entries.iter().filter(|e| e.workflow.is_enabled()).count();
    println!("{}", format!("Workflows ({} registered, {} enabled)", total, enabled).bold());
    println!();
    for entry in entries {
        let status_icon = if entry.workflow.is_enabled() { "●" } else { "○" };
        println!("  {} {}  —  {}", status_icon, entry.name.cyan(), entry.workflow.description);
    }
}

fn print_with_status(entries: &[registry::WorkflowEntry]) {
    let total = entries.len();
    let stale_count = entries.iter().filter(|e| e.stale).count();
    let healthy = total - stale_count;
    println!(
        "{}",
        format!(
            "Workflows ({} registered, {} healthy, {} stale)",
            total, healthy, stale_count
        )
        .bold()
    );
    println!();
    for entry in entries {
        if entry.stale {
            let age_str = entry
                .last_output_at
                .map(|ts| {
                    let hours = (chrono::Utc::now() - ts).num_hours();
                    if hours < 48 {
                        format!("{}h ago", hours)
                    } else {
                        format!("{}d ago", hours / 24)
                    }
                })
                .unwrap_or_else(|| "never".to_string());
            println!(
                "  {} {}  —  {} (last ran: {})",
                "⚠".yellow(),
                entry.name.yellow(),
                entry.workflow.description,
                age_str
            );
        } else if !entry.workflow.is_enabled() {
            println!("  {} {}  —  {}", "○".dimmed(), entry.name.dimmed(), entry.workflow.description);
        } else {
            println!("  {} {}  —  {}", "✓".green(), entry.name.green(), entry.workflow.description);
        }
    }
}
