use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RunMetadata {
    pub run_id: String,
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub input_text: String,
    pub input_length_chars: usize,
    pub output_length_chars: usize,
    pub model: String,
    pub tokens_input: u64,
    pub tokens_output: u64,
    pub tokens_input_estimated: bool,
    pub tokens_output_estimated: bool,
    pub duration_ms: u128,
    pub praxis_version: String,
}

/// Estimate token count using ~4 chars per token heuristic.
pub fn estimate_tokens(text: &str) -> u64 {
    ((text.len() as f64) / 4.0).ceil() as u64
}

pub fn build_metadata(
    run_id: &str,
    timestamp: DateTime<Utc>,
    command: &str,
    input_text: &str,
    output_text: &str,
    model: &str,
    raw_input_tokens: Option<u64>,
    raw_output_tokens: Option<u64>,
    duration_ms: u128,
) -> RunMetadata {
    let (tokens_input, tokens_input_estimated) = match raw_input_tokens {
        Some(t) => (t, false),
        None => (estimate_tokens(input_text), true),
    };

    let (tokens_output, tokens_output_estimated) = match raw_output_tokens {
        Some(t) => (t, false),
        None => (estimate_tokens(output_text), true),
    };

    RunMetadata {
        run_id: run_id.to_string(),
        timestamp,
        command: command.to_string(),
        input_text: input_text.to_string(),
        input_length_chars: input_text.len(),
        output_length_chars: output_text.len(),
        model: model.to_string(),
        tokens_input,
        tokens_output,
        tokens_input_estimated,
        tokens_output_estimated,
        duration_ms,
        praxis_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}
