use std::time::Instant;

use chrono::Utc;
use colored::Colorize;
use uuid::Uuid;

use crate::config::PraxisConfig;
use crate::llm;
use crate::observability;
use crate::storage;

const THINK_SYSTEM_PROMPT: &str = r#"You are a structured reasoning engine. Your sole purpose is to help users think clearly and make decisions.

When given a problem, you MUST respond with EXACTLY this structure — no more, no less:

## Problem Framing
[One concise sentence restating the core problem.]

## Constraints
[Bullet list of real constraints: time, resources, technical, organizational, etc.]

## Options
[Numbered list of distinct options or approaches. Each option in one sentence.]

## Trade-offs
[For each option, one line of pros vs cons. Format: "Option N: <pro> / <con>"]

## Recommendation
[One clear recommendation with a one-sentence rationale.]

Rules:
- No preamble. No pleasantries. No follow-up questions.
- Be concise and decision-oriented.
- Do not add sections beyond those listed above.
- Use plain, direct language.
- If you cannot determine something, state the assumption explicitly."#;

pub fn run_think(problem: &str, config: &PraxisConfig) -> Result<(), String> {
    let run_id = Uuid::new_v4().to_string();
    let short_id = &run_id[..8];
    let timestamp = Utc::now();

    println!("{}", "Thinking...".dimmed());
    println!();

    let start = Instant::now();

    let llm_response = llm::generate_response(THINK_SYSTEM_PROMPT, problem, config)?;

    let duration_ms = start.elapsed().as_millis();

    // Print structured output to terminal
    print_output(problem, &llm_response.text);

    // Build metadata
    let metadata = observability::build_metadata(
        &run_id,
        timestamp,
        "think",
        problem,
        &llm_response.text,
        &llm_response.model,
        llm_response.input_tokens,
        llm_response.output_tokens,
        duration_ms,
    );

    // Persist artifact + metadata
    let paths = storage::save_run(
        &config.praxis_dir,
        &timestamp,
        short_id,
        "think",
        problem,
        &llm_response.text,
        &metadata,
    )?;

    // Run summary
    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!(
        "  {}  {}",
        "run id:".dimmed(),
        short_id.cyan()
    );
    println!(
        "  {}  {}",
        "model: ".dimmed(),
        metadata.model.cyan()
    );
    println!(
        "  {}  {}ms",
        "time:  ".dimmed(),
        metadata.duration_ms.to_string().cyan()
    );

    let token_label = if metadata.tokens_input_estimated || metadata.tokens_output_estimated {
        "tokens:".to_string()
    } else {
        "tokens:".to_string()
    };
    let token_suffix = if metadata.tokens_input_estimated || metadata.tokens_output_estimated {
        " (estimated)".dimmed().to_string()
    } else {
        String::new()
    };
    println!(
        "  {}  {} in / {} out{}",
        token_label.dimmed(),
        metadata.tokens_input.to_string().cyan(),
        metadata.tokens_output.to_string().cyan(),
        token_suffix
    );
    println!(
        "  {}  {}",
        "saved: ".dimmed(),
        paths.artifact.display().to_string().dimmed()
    );
    println!(
        "  {}  {}",
        "meta:  ".dimmed(),
        paths.metadata.display().to_string().dimmed()
    );
    println!("{}", "─".repeat(60).dimmed());

    Ok(())
}

fn print_output(problem: &str, output: &str) {
    println!("{}", problem.bold().white());
    println!();

    for line in output.lines() {
        if line.starts_with("## ") {
            let heading = line.trim_start_matches("## ");
            println!("{}", heading.cyan().bold());
        } else if line.starts_with("- ") || line.starts_with("* ") {
            let content = &line[2..];
            println!("  {} {}", "•".dimmed(), content);
        } else if line.len() > 2 && line.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) && line.chars().nth(1) == Some('.') {
            println!("  {}", line);
        } else {
            println!("{}", line);
        }
    }
}
