use std::io::{BufRead, BufReader, Write};

use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};

use crate::config::PraxisConfig;

pub struct LlmResponse {
    pub text: String,
    pub reasoning: Option<String>,
    pub model: String,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_read_tokens: Option<u64>,
    pub cache_write_tokens: Option<u64>,
}

pub enum StreamEvent<'a> {
    Content(&'a str),
    Reasoning(&'a str),
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<StreamOptions>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct StreamOptions {
    include_usage: bool,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
    model: Option<String>,
}

#[derive(Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Deserialize)]
struct AssistantMessage {
    content: String,
    #[serde(default)]
    reasoning: Option<String>,
}

#[derive(Deserialize)]
struct StreamChatResponse {
    choices: Vec<StreamChoice>,
    usage: Option<Usage>,
    model: Option<String>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: Option<StreamDelta>,
    message: Option<AssistantMessage>,
}

#[derive(Deserialize)]
struct StreamDelta {
    content: Option<String>,
    reasoning: Option<String>,
}

#[derive(Deserialize)]
struct PromptTokensDetails {
    cached_tokens: Option<u64>,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    /// OpenAI format: prompt_tokens_details.cached_tokens
    prompt_tokens_details: Option<PromptTokensDetails>,
    /// Anthropic format (passed through by LiteLLM / direct Anthropic API)
    cache_read_input_tokens: Option<u64>,
    cache_creation_input_tokens: Option<u64>,
}

impl Usage {
    fn cache_read(&self) -> Option<u64> {
        self.cache_read_input_tokens
            .or_else(|| self.prompt_tokens_details.as_ref().and_then(|d| d.cached_tokens))
    }

    fn cache_write(&self) -> Option<u64> {
        self.cache_creation_input_tokens
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LlmRequestOptions {
    pub stream: bool,
}

pub fn generate_response(
    system_prompt: &str,
    user_prompt: &str,
    config: &PraxisConfig,
    options: LlmRequestOptions,
    mut on_chunk: impl FnMut(StreamEvent<'_>) -> Result<(), String>,
) -> Result<LlmResponse, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(config.llm_timeout_secs))
        .build()
        .map_err(|e| format!("error: Failed to build HTTP client: {}", e))?;

    let request_body = ChatRequest {
        model: config.llm_model.clone(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ],
        temperature: 0.3,
        stream: options.stream,
        max_tokens: config.llm_max_output_tokens,
        stream_options: options.stream.then_some(StreamOptions { include_usage: true }),
    };

    let url = format!("{}/chat/completions", config.llm_api_base.trim_end_matches('/'));

    let mut req = client.post(&url).json(&request_body);

    if let Some(api_key) = &config.llm_api_key {
        req = req.bearer_auth(api_key);
    }

    let response = req.send().map_err(|e| {
        if e.is_timeout() {
            format!(
                "error: LLM request timed out after {}s while calling {} with model {}\nHint: local models can take longer on first-token and full completion latency. Try increasing PRAXIS_LLM_TIMEOUT_SECS or lowering PRAXIS_LLM_MAX_OUTPUT_TOKENS.\nHint: to benchmark the raw model outside praxis, run: ollama run {} {:?}",
                config.llm_timeout_secs,
                config.llm_api_base,
                config.llm_model,
                config.llm_model,
                user_prompt
            )
        } else if e.is_connect() {
            format!(
                "error: Failed to connect to LLM at {}: {}\nHint: Is your local model running? (e.g., `ollama serve`)",
                config.llm_api_base, e
            )
        } else {
            format!("error: LLM request failed: {}", e)
        }
    })?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        return Err(format!("error: LLM API returned status {}: {}", status, body));
    }

    if options.stream {
        return read_streaming_response(response, config, &mut on_chunk);
    }

    let chat_response: ChatResponse = response
        .json()
        .map_err(|e| format!("error: Failed to parse LLM response: {}", e))?;

    let text = chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "error: LLM returned no choices".to_string())?;
    let reasoning = chat_response
        .choices
        .first()
        .and_then(|c| c.message.reasoning.clone());

    let (input_tokens, output_tokens, cache_read_tokens, cache_write_tokens) =
        if let Some(usage) = chat_response.usage {
            (usage.prompt_tokens, usage.completion_tokens, usage.cache_read(), usage.cache_write())
        } else {
            (None, None, None, None)
        };

    let model = chat_response.model.unwrap_or_else(|| config.llm_model.clone());

    Ok(LlmResponse {
        text,
        reasoning,
        model,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_write_tokens,
    })
}

fn read_streaming_response(
    response: reqwest::blocking::Response,
    config: &PraxisConfig,
    on_chunk: &mut impl FnMut(StreamEvent<'_>) -> Result<(), String>,
) -> Result<LlmResponse, String> {
    let is_event_stream = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.contains("text/event-stream"))
        .unwrap_or(false);

    if !is_event_stream {
        return read_non_streaming_fallback(response, config, on_chunk);
    }

    let mut reader = BufReader::new(response);
    let mut line = String::new();
    let mut text = String::new();
    let mut reasoning_text = String::new();
    let mut model = config.llm_model.clone();
    let mut input_tokens = None;
    let mut output_tokens = None;
    let mut cache_read_tokens = None;
    let mut cache_write_tokens = None;

    loop {
        line.clear();
        let bytes = reader
            .read_line(&mut line)
            .map_err(|e| format!("error: Failed to read streaming response: {}", e))?;

        if bytes == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.starts_with("data: ") {
            continue;
        }

        let payload = trimmed.trim_start_matches("data: ").trim();
        if payload == "[DONE]" {
            break;
        }

        let chunk: StreamChatResponse = serde_json::from_str(payload)
            .map_err(|e| format!("error: Failed to parse streaming LLM chunk: {}", e))?;

        if let Some(chunk_model) = chunk.model {
            model = chunk_model;
        }

        if let Some(usage) = chunk.usage {
            input_tokens = usage.prompt_tokens.or(input_tokens);
            output_tokens = usage.completion_tokens.or(output_tokens);
            cache_read_tokens = usage.cache_read().or(cache_read_tokens);
            cache_write_tokens = usage.cache_write().or(cache_write_tokens);
        }

        for choice in chunk.choices {
            let reasoning = choice
                .delta
                .as_ref()
                .and_then(|delta| delta.reasoning.as_deref());
            if let Some(reasoning) = reasoning {
                on_chunk(StreamEvent::Reasoning(reasoning))?;
                reasoning_text.push_str(reasoning);
            }

            let content = choice
                .delta
                .and_then(|delta| delta.content)
                .or_else(|| choice.message.map(|message| message.content));
            if let Some(content) = content {
                on_chunk(StreamEvent::Content(&content))?;
                text.push_str(&content);
            }
        }
    }

    std::io::stdout()
        .flush()
        .map_err(|e| format!("error: Failed to flush streaming output: {}", e))?;

    Ok(LlmResponse {
        text,
        reasoning: (!reasoning_text.is_empty()).then_some(reasoning_text),
        model,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_write_tokens,
    })
}

fn read_non_streaming_fallback(
    response: reqwest::blocking::Response,
    config: &PraxisConfig,
    on_chunk: &mut impl FnMut(StreamEvent<'_>) -> Result<(), String>,
) -> Result<LlmResponse, String> {
    let chat_response: ChatResponse = response
        .json()
        .map_err(|e| format!("error: Failed to parse LLM response: {}", e))?;

    let text = chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "error: LLM returned no choices".to_string())?;
    let reasoning = chat_response
        .choices
        .first()
        .and_then(|c| c.message.reasoning.clone());

    on_chunk(StreamEvent::Content(&text))?;

    let (input_tokens, output_tokens, cache_read_tokens, cache_write_tokens) =
        if let Some(usage) = chat_response.usage {
            (usage.prompt_tokens, usage.completion_tokens, usage.cache_read(), usage.cache_write())
        } else {
            (None, None, None, None)
        };

    let model = chat_response.model.unwrap_or_else(|| config.llm_model.clone());

    Ok(LlmResponse {
        text,
        reasoning,
        model,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_write_tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_delta_stream_chunks() {
        let chunk: StreamChatResponse = serde_json::from_str(
            r#"{"model":"qwen3.5:9B","choices":[{"delta":{"content":"hello","reasoning":"plan"}}]}"#,
        )
        .unwrap();
        let content = chunk.choices[0].delta.as_ref().and_then(|delta| delta.content.as_deref());
        assert_eq!(content, Some("hello"));
        let reasoning = chunk.choices[0]
            .delta
            .as_ref()
            .and_then(|delta| delta.reasoning.as_deref());
        assert_eq!(reasoning, Some("plan"));
    }

    #[test]
    fn parses_message_stream_chunks() {
        let chunk: StreamChatResponse = serde_json::from_str(
            r#"{"model":"qwen3.5:9B","choices":[{"message":{"content":"hello"}}]}"#,
        )
        .unwrap();
        let content = chunk.choices[0].message.as_ref().map(|message| message.content.as_str());
        assert_eq!(content, Some("hello"));
    }

    #[test]
    fn parses_non_stream_reasoning_field() {
        let response: ChatResponse = serde_json::from_str(
            r#"{"model":"qwen3.5:9B","choices":[{"message":{"content":"","reasoning":"long chain"}}]}"#,
        )
        .unwrap();
        assert_eq!(
            response.choices[0].message.reasoning.as_deref(),
            Some("long chain")
        );
    }
}
