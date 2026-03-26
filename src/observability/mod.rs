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
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
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
    cache_read_tokens: Option<u64>,
    cache_write_tokens: Option<u64>,
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
        cache_read_tokens: cache_read_tokens.unwrap_or(0),
        cache_write_tokens: cache_write_tokens.unwrap_or(0),
        duration_ms,
        praxis_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn estimate_tokens_empty() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn estimate_tokens_rounds_up() {
        // 1 char → ceil(1/4) = 1
        assert_eq!(estimate_tokens("a"), 1);
        // 4 chars → 1
        assert_eq!(estimate_tokens("abcd"), 1);
        // 5 chars → 2
        assert_eq!(estimate_tokens("abcde"), 2);
    }

    #[test]
    fn estimate_tokens_typical_sentence() {
        let text = "How should I structure my Rust project?"; // 39 chars → ceil(39/4) = 10
        assert_eq!(estimate_tokens(text), 10);
    }

    #[test]
    fn build_metadata_uses_provided_tokens() {
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let meta = build_metadata(
            "run-id-1",
            ts,
            "think",
            "my problem",
            "my output",
            "qwen3.5:9B",
            Some(50),
            Some(100),
            None,
            None,
            42,
        );
        assert_eq!(meta.tokens_input, 50);
        assert_eq!(meta.tokens_output, 100);
        assert!(!meta.tokens_input_estimated);
        assert!(!meta.tokens_output_estimated);
    }

    #[test]
    fn build_metadata_estimates_tokens_when_none() {
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let input = "abcd"; // 4 chars → 1 token
        let output = "abcde"; // 5 chars → 2 tokens
        let meta = build_metadata("id", ts, "think", input, output, "qwen3.5:9B", None, None, None, None, 10);
        assert_eq!(meta.tokens_input, 1);
        assert_eq!(meta.tokens_output, 2);
        assert!(meta.tokens_input_estimated);
        assert!(meta.tokens_output_estimated);
    }

    #[test]
    fn build_metadata_mixed_tokens() {
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let meta = build_metadata("id", ts, "think", "input", "output", "model", Some(5), None, None, None, 0);
        assert_eq!(meta.tokens_input, 5);
        assert!(!meta.tokens_input_estimated);
        assert!(meta.tokens_output_estimated);
    }

    #[test]
    fn build_metadata_records_lengths() {
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let meta = build_metadata("id", ts, "think", "hello", "world!", "m", None, None, None, None, 0);
        assert_eq!(meta.input_length_chars, 5);
        assert_eq!(meta.output_length_chars, 6);
    }
}
