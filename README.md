# Praxis

Treat AI like a system, not a conversation.

Praxis is a terminal-native tool for structured reasoning with built-in observability. Give it a problem; get back a decision-oriented analysis, a persistent artifact, and full metadata on what happened — model, tokens, cost, duration.

Every interaction is a **workflow run**. Every run follows a structured method, produces a reusable artifact, and generates metadata. Over time, this creates a history of thinking you can search, reference, and optimise.

```
$ praxis think "Should I rewrite the auth service in Rust?"

Thinking...

Should I rewrite the auth service in Rust?

Problem Framing
The current auth service works but is a maintenance burden...

Constraints
  • Team has no Rust experience
  • Auth is security-critical — rewrites introduce risk
  • Current service handles 2k req/s without issues

Options
  1. Full rewrite in Rust
  2. Incremental migration — extract hot paths only
  3. Keep current service, invest in test coverage

Trade-offs
  Option 1: Maximum long-term perf / highest risk, longest timeline
  Option 2: Targeted gains / partial complexity, two codebases during transition
  Option 3: Lowest risk / doesn't address maintenance burden

Recommendation
  Option 2. Extract the token validation hot path first...

────────────────────────────────────────────────────────────
  run id:  a3f9b12c
  model:   qwen3.5:9B
  time:    4231ms
  tokens:  312 in / 189 out
  saved:   ~/.praxis/runs/20260324-120000-a3f9b12c-think.md
  meta:    ~/.praxis/runs/20260324-120000-a3f9b12c-think.meta.json
────────────────────────────────────────────────────────────
```

## Why

Chat-based AI interaction is unstructured, ephemeral, and opaque. You get an answer, it disappears, and you have no idea what it cost or whether a different model would have done better.

Praxis makes AI usage **structured** (defined workflows, not freeform prompts), **persistent** (every run saved as a searchable artifact), and **observable** (token counts, model, duration, cost — tracked from day one).

See [VISION.md](VISION.md) for the full thesis and roadmap.

## Install

```bash
git clone https://github.com/cgracia/praxis
cd praxis
cargo install --path .
```

Requires Rust 1.70+ and an OpenAI-compatible LLM endpoint. [Ollama](https://ollama.com/) is the zero-config default.

## Quick start

```bash
# Local model (default)
ollama serve && ollama pull qwen3.5:9B
praxis think "What database should I use for this project?"

# Ground the answer in the current repo
praxis think --repo "What database should I use for this project?"

# Remote provider
export PRAXIS_LLM_API_BASE="https://api.openai.com/v1"
export PRAXIS_LLM_MODEL="gpt-4o"
export PRAXIS_LLM_API_KEY="sk-..."
praxis think "Should we build or buy the billing system?"
```

## How it works

Each `praxis think` run:

1. Sends your problem to an LLM with a structured system prompt that enforces a specific output format (Problem Framing → Constraints → Options → Trade-offs → Recommendation)
2. Streams the result in interactive terminals by default, or prints the full response at the end in non-streaming mode
3. Saves a **markdown artifact** with YAML frontmatter to `~/.praxis/runs/`
4. Saves a **JSON metadata file** alongside it — run ID, model, tokens in/out, duration, timestamp

The artifacts accumulate into a personal knowledge base of structured reasoning. The metadata accumulates into a dataset of your AI usage.

## Configuration

Praxis resolves config in this order (highest priority last):

1. Built-in defaults — Ollama at `localhost:11434`, model `qwen3.5:9B`
2. Config file — `~/.praxis/config.toml`
3. Environment variables

```toml
# ~/.praxis/config.toml
llm_api_base = "http://localhost:11434/v1"
llm_model    = "qwen3.5:9B"
llm_api_key  = ""          # leave empty for Ollama
llm_timeout_secs = 600
llm_max_output_tokens = 700
```

| Variable | Description |
|---|---|
| `PRAXIS_LLM_API_BASE` | API base URL |
| `PRAXIS_LLM_MODEL` | Model name |
| `PRAXIS_LLM_API_KEY` | Bearer token (optional for local models) |
| `PRAXIS_LLM_TIMEOUT_SECS` | HTTP timeout for the full LLM response |
| `PRAXIS_LLM_MAX_OUTPUT_TOKENS` | Cap completion length to reduce latency |
| `PRAXIS_DIR` | Override storage directory (default: `~/.praxis`) |

## Local model notes

`praxis` streams by default when writing to an interactive terminal. That makes local models feel much better for humans, even when total completion time is still significant.

When stdout is not a TTY, `praxis` defaults to non-streaming output so scripts and tool integrations get a stable full response.

If local runs feel too slow:

```bash
export PRAXIS_LLM_TIMEOUT_SECS=900
export PRAXIS_LLM_MAX_OUTPUT_TOKENS=400
```

To override the default behavior:

```bash
praxis think --no-stream "What database should I use?"
praxis think --stream "What database should I use?"
```

To compare raw model behavior outside `praxis`:

```bash
ollama run qwen3.5:9B "What database should I use for this project?"
```

## Local context

`praxis think` is context-free by default. It does not silently inspect your filesystem.

To explicitly ground a run in the current repository:

```bash
praxis think --repo "What database should I use for this project?"
```

When `--repo` is enabled, Praxis reads a small, filtered set of project files from the current working directory, excludes common secret-bearing paths and files, injects that material into the prompt, and records the included file list in the saved artifact.

This is intentionally explicit. Local context is a declared input, not hidden state.

## Artifacts

Each run produces two files in `~/.praxis/runs/`:

| File | Contents |
|---|---|
| `<timestamp>-<id>-think.md` | Markdown with YAML frontmatter, the problem, and the full structured response |
| `<timestamp>-<id>-think.meta.json` | JSON metadata: model, token counts, duration, praxis version |

Token counts are exact when the API provides usage data; otherwise estimated at ~4 chars/token (marked `tokens~:` in the footer).

Files are plain text — searchable with `grep`, parseable with `jq`, composable with anything.

## Commands

| Command | Description |
|---|---|
| `praxis think <problem>` | Run the structured thinking workflow |
| `praxis think --repo <problem>` | Run with explicit context from the current repo |
| `praxis think --no-stream <problem>` | Disable streaming output |
| `praxis --version` | Print version |
| `praxis --help` | Print help |

## Architecture

```
src/
├── main.rs              # Entry point
├── cli/mod.rs           # Argument parsing (clap)
├── config/mod.rs        # Config resolution: defaults → file → env
├── llm/mod.rs           # OpenAI-compatible API client
├── observability/mod.rs # Metadata generation, token estimation
├── storage/mod.rs       # Artifact and metadata persistence
└── workflows/mod.rs     # Workflow orchestration and output formatting
```

## Testing

```bash
cargo test
```

Unit tests cover all pure functions — token estimation, metadata construction, YAML serialisation, storage, and output formatting. See the test modules in each `mod.rs` for specifics.

Integration tests against a live LLM endpoint are planned but not yet implemented.

## Roadmap

Praxis is in **Phase 1** — one workflow, one artifact format, observability from day one. The evolution path (detailed in [VISION.md](VISION.md)):

| Phase | What | Status |
|---|---|---|
| 1. Foundation | `think` workflow, artifacts, metadata | **Current** |
| 2. Workflows | `decide`, `plan`, `review`, `summarise` | Planned |
| 3. Observability | `praxis stats`, cost tracking, model comparison | Planned |
| 4. Context | Local RAG over artifacts, cross-run memory | Planned |
| 5. Composition | Workflow chaining, tool plugins | Future |
| 6. Integrations | External system connectors | Future |

## Licence

MIT
