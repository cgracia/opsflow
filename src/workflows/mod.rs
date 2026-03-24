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
        "tokens~:"
    } else {
        "tokens: "
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

/// Returns true for lines like "1. foo", "10. bar", "99. baz".
fn is_numbered_item(line: &str) -> bool {
    let rest = line.trim_start_matches(|c: char| c.is_ascii_digit());
    rest != line && rest.starts_with(". ")
}

pub fn print_output(problem: &str, output: &str) {
    println!("{}", problem.bold().white());
    println!();

    for line in output.lines() {
        if line.starts_with("## ") {
            let heading = line.trim_start_matches("## ");
            println!("{}", heading.cyan().bold());
        } else if line.starts_with("- ") || line.starts_with("* ") {
            let content = &line[2..];
            println!("  {} {}", "•".dimmed(), content);
        } else if is_numbered_item(line) {
            println!("  {}", line);
        } else {
            println!("{}", line);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_numbered_item ---

    #[test]
    fn numbered_item_single_digit() {
        assert!(is_numbered_item("1. First option"));
        assert!(is_numbered_item("9. Last single digit"));
    }

    #[test]
    fn numbered_item_multi_digit() {
        assert!(is_numbered_item("10. Tenth option"));
        assert!(is_numbered_item("42. Some option"));
        assert!(is_numbered_item("100. One hundredth"));
    }

    #[test]
    fn numbered_item_rejects_non_numeric_start() {
        assert!(!is_numbered_item("- bullet"));
        assert!(!is_numbered_item("## heading"));
        assert!(!is_numbered_item("plain text"));
        assert!(!is_numbered_item(""));
    }

    #[test]
    fn numbered_item_requires_dot_space() {
        // Digit followed by dot but no space — not a list item
        assert!(!is_numbered_item("1.option"));
        // Digit with space but no dot — not a list item
        assert!(!is_numbered_item("1 option"));
    }

    #[test]
    fn numbered_item_rejects_bare_number() {
        assert!(!is_numbered_item("1"));
        assert!(!is_numbered_item("10"));
    }
}
