# Praxis

A CLI tool for structured reasoning. Give it a problem; get back a concise, decision-oriented analysis powered by a local or remote LLM.

```
$ praxis think "Should I rewrite the auth service in Rust?"

Thinking...

[Problem Framing]
...
[Recommendation]
...

────────────────────────────────────────────────────────────
  run id:  a3f9b12c
  model:   llama3:8b
  time:    4231ms
  tokens:  312 in / 189 out
  saved:   ~/.praxis/runs/20240615-120000-a3f9b12c-think.md
  meta:    ~/.praxis/runs/20240615-120000-a3f9b12c-think.meta.json
────────────────────────────────────────────────────────────
```

## Requirements

- Rust 1.70+
- An OpenAI-compatible LLM endpoint — [Ollama](https://ollama.com/) is the zero-config default

## Installation

```bash
git clone https://github.com/cgracia/praxis
cd praxis
cargo install --path .
```

## Quick start

```bash
# Start Ollama (if using local models)
ollama serve
ollama pull llama3

# Run
praxis think "What database should I use for this project?"
```

## Configuration

Praxis resolves config in this order (highest priority last):

1. Built-in defaults (Ollama at `localhost:11434`, model `llama3`)
2. `~/.praxis/config.toml`
3. Environment variables

### Config file

```toml
# ~/.praxis/config.toml
llm_api_base = "http://localhost:11434/v1"
llm_model    = "llama3:8b"
llm_api_key  = ""          # leave empty for Ollama
```

### Environment variables

| Variable             | Description                        |
|----------------------|------------------------------------|
| `PRAXIS_LLM_API_BASE`| API base URL                       |
| `PRAXIS_LLM_MODEL`   | Model name                         |
| `PRAXIS_LLM_API_KEY` | Bearer token (optional)            |
| `PRAXIS_DIR`         | Override storage directory         |

### Using a remote provider (e.g. OpenAI)

```bash
export PRAXIS_LLM_API_BASE="https://api.openai.com/v1"
export PRAXIS_LLM_MODEL="gpt-4o"
export PRAXIS_LLM_API_KEY="sk-..."
praxis think "..."
```

## Artifacts

Each run saves two files to `~/.praxis/runs/`:

- **`<timestamp>-<id>-think.md`** — Markdown with YAML frontmatter, the problem, and the full LLM response
- **`<timestamp>-<id>-think.meta.json`** — JSON metadata: model, token counts, duration, praxis version

Token counts are exact when the API provides them; otherwise estimated at ~4 chars/token (marked `tokens~:` in the footer).

## Commands

| Command              | Description                            |
|----------------------|----------------------------------------|
| `praxis think <problem>` | Run the structured thinking workflow |
| `praxis --version`   | Print version                          |
| `praxis --help`      | Print help                             |

## Testing

```bash
cargo test
```

### Testing strategy

Tests are co-located with their module in `#[cfg(test)]` blocks.

**Unit tests** cover all pure functions:

| Module          | What's tested                                                   |
|-----------------|-----------------------------------------------------------------|
| `observability` | `estimate_tokens` (empty, rounding, typical), `build_metadata` (real tokens, estimated, mixed, char lengths) |
| `storage`       | `yaml_quote` (plain, colon, escaping), `run_prefix` format, `build_frontmatter` field presence and quoting, `save_run` file creation and content |
| `workflows`     | `is_numbered_item` (single-digit, multi-digit, rejects edge cases) |

**Integration tests** (not yet implemented — contributions welcome):
- Mock HTTP server to test `llm::generate_response` against a fake OpenAI-compatible endpoint
- End-to-end `praxis think` invocation against a live Ollama instance (gated, not in CI by default)

## License

MIT
