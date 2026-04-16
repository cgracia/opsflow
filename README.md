# Praxis

Instrument your AI usage. Understand where your tokens go.

## Why

I use multiple AI coding tools daily — OpenCode, Claude Code, local models via Ollama. I had no visibility into which models were actually performing, where my tokens were going, or whether expensive features like extended thinking were paying off for anything.

Existing tools like Langfuse are built for production LLM applications. They are not built for developer workflows where you want to know: "across 700 coding sessions, which model gave me the best results for which type of task, and what did it cost?"

Praxis fills that gap. It sits below the AI tools you already use, reads their log files, and gives you a queryable history of your actual usage.

> **Status: Alpha.** Claude Code ingestion works. OpenCode ingestion is in progress (schema parser WIP). Data is local, no cloud dependencies.

---

Praxis is a terminal-native observability layer for AI-assisted workflows. It ingests session logs from OpenCode and Claude Code, normalises them into a local DuckDB database, and makes them queryable — so you can answer real questions about your AI usage.

```
$ praxis ingest
Ingestion complete:
  opencode:     892 runs, 47 sessions (12 files skipped)
  claude-code:  342 runs, 18 sessions (3 files skipped)
  praxis:         13 runs, 0 sessions (0 files skipped)

  Total: 1,247 new runs across 65 sessions

$ praxis stats
Praxis — AI Usage Summary (last 30 days)
──────────────────────────────────────────────────
  Total runs:          1,247
  Total tokens:        2.4M in / 890K out
  Estimated cost:      $47.82
  Sources:             opencode (892), claude-code (342), praxis (13)
  Top models:          claude-sonnet-4 (678), claude-opus-4 (312), qwen3.5:9B (257)
  Top projects:        myapp (456), infra (234), praxis (89)
──────────────────────────────────────────────────

$ praxis stats --by day
Date         Runs   Tokens In   Tokens Out   Cost (est.)   Top Model
2026-03-25     42      84.2K       31.1K        $1.89      claude-sonnet-4
2026-03-24     67     142.5K       58.3K        $3.41      claude-opus-4
...

$ praxis runs --since 1d
Timestamp                Source       Model                  Tokens In   Tokens Out         Cost   Project
2026-03-25 14:30:22     opencode     claude-sonnet-4…            1.5K        0.3K         $0.01   myapp
2026-03-25 12:00:00     praxis       qwen3.5:9B                   312         189             —   —
...
```

## Why

Chat-based AI interaction is unstructured, ephemeral, and opaque. You spend tokens and money with no visibility into which models cost what, which projects consume the most, or how usage trends over time.

Praxis fixes this by sitting one layer below the AI tools you already use: it reads their log files, normalises them into a unified schema, and gives you a queryable history of your AI usage.

The `think` command (Phase 1) is still here — a structured reasoning workflow that saves artifacts and metadata. But the bigger idea is the observability layer itself: **instrument actual AI usage, index it, make it queryable**.

## Install

```bash
git clone https://github.com/cgracia/praxis
cd praxis
cargo install --path .
```

Requires Rust 1.70+.

## Quick start

```bash
# Ingest your AI tool logs
praxis ingest

# See what you've been spending
praxis stats
praxis stats --by model
praxis stats --by project
praxis stats --since 7d

# Browse recent activity
praxis runs
praxis runs --source opencode
praxis runs --since 1d --model claude-sonnet
```

## Data sources

### OpenCode (primary — most reliable)

Location: `~/.local/share/opencode/` (or `OPENCODE_DATA_DIR`)

Token counts come directly from API responses — they're accurate. Cost is calculated from tokens using the LiteLLM pricing database (fetched and cached locally).

```bash
praxis ingest --source opencode
```

### Claude Code (supported, with caveats)

Location: `~/.claude/projects/`

**Important:** Claude Code JSONL token data is fundamentally unreliable due to a known Anthropic bug ([#22686](https://github.com/anthropics/claude-code/issues/22686)). Input tokens are 100-174x too low; output tokens exclude thinking tokens. Cache fields are accurate.

Praxis ingests Claude Code data but clearly marks all token/cost values as unreliable estimates (`~`). Cache token counts (which are accurate) are preserved.

```bash
praxis ingest --source claude-code
```

Alternatively, use ccusage as a parsing layer:

```bash
# Install ccusage: npm install -g ccusage
ccusage session --json > ~/.praxis/imports/ccusage-claude-code.json
praxis ingest --source ccusage
```

### Praxis native

Every `praxis think` run saves a `.meta.json` file. The `ingest` command picks these up automatically.

```bash
praxis ingest --source praxis
```

## Commands

### `praxis ingest`

Scans log directories, normalises data, loads new records into DuckDB. Idempotent — safe to run repeatedly.

```bash
praxis ingest                              # All sources
praxis ingest --source opencode           # One source
praxis ingest --source claude-code
praxis ingest --source praxis
praxis ingest --source ccusage            # Import ccusage JSON exports
praxis ingest --opencode-dir /custom/path # Override directory
```

### `praxis runs`

List recent AI runs across all sources.

```bash
praxis runs                                # Last 20 runs
praxis runs --since 7d                     # Last 7 days
praxis runs --since 2026-03-20             # Since specific date
praxis runs --source opencode              # One source
praxis runs --model claude-sonnet          # Filter by model (substring)
praxis runs --project myapp               # Filter by project (substring)
praxis runs --limit 50                     # More results
praxis runs --json                         # JSON output for scripting
```

### `praxis stats`

Aggregate statistics over ingested data.

```bash
praxis stats                               # Summary (last 30 days)
praxis stats --by day                      # Daily breakdown
praxis stats --by model                    # By model
praxis stats --by project                  # By project
praxis stats --by source                   # By source
praxis stats --since 2026-03-01            # Custom date range
praxis stats --since 7d                    # Last 7 days
praxis stats --json                        # JSON output
```

### `praxis think`

Structured reasoning workflow — unchanged from Phase 1.

```bash
praxis think "Should I rewrite the auth service in Rust?"
praxis think --repo "What database should I use for this project?"
```

## Configuration

```toml
# ~/.praxis/config.toml
llm_api_base      = "http://localhost:11434/v1"
llm_model         = "qwen3.5:9B"
llm_api_key       = ""
llm_timeout_secs  = 600

# Source directories (auto-detected if not set)
opencode_dir      = "~/.local/share/opencode"
claude_code_dir   = "~/.claude/projects"
```

| Variable | Description |
|---|---|
| `PRAXIS_DIR` | Override storage directory (default: `~/.praxis`) |
| `PRAXIS_LLM_API_BASE` | API base URL |
| `PRAXIS_LLM_MODEL` | Model name |
| `PRAXIS_LLM_API_KEY` | Bearer token |
| `PRAXIS_OPENCODE_DIR` | Override OpenCode data directory |
| `PRAXIS_CLAUDE_CODE_DIR` | Override Claude Code projects directory |

## Data storage

All data is local. No cloud dependencies.

```
~/.praxis/
├── praxis.db              # DuckDB database (sessions, runs, ingestion_log tables)
├── pricing-cache.json     # LiteLLM pricing database (refreshed every 7 days)
├── config.toml
├── runs/                  # praxis think artifacts + .meta.json files
└── imports/               # Drop ccusage JSON exports here
    └── ccusage-*.json
```

The database has three tables:

- **`sessions`** — one row per AI session (OpenCode session, Claude Code JSONL file, etc.)
- **`runs`** — one row per message/exchange — the core analytics table
- **`ingestion_log`** — tracks what's been ingested for incremental updates

Query the database directly with DuckDB if you want more than `praxis stats` provides:

```bash
duckdb ~/.praxis/praxis.db "SELECT model, COUNT(*) FROM runs GROUP BY model ORDER BY 2 DESC"
```

## Data quality

| Source | Token accuracy | Cost accuracy |
|---|---|---|
| OpenCode | Reliable (from API responses) | Calculated from tokens (reliable) |
| Claude Code JSONL | **Unreliable** (100x+ undercount, Anthropic bug #22686) | **Unreliable** (based on wrong counts) |
| ccusage import | Inherits source quality | Uses LiteLLM pricing (same as us) |
| praxis native | Reliable or estimated (marked) | Calculated from tokens |

Unreliable values are displayed with `~` prefixes. The data is useful for understanding relative usage patterns even when absolute numbers are wrong.

## Architecture

```
src/
├── main.rs               # Entry point and CLI dispatch
├── cli/mod.rs            # Argument parsing (clap)
├── config/mod.rs         # Config resolution: defaults → file → env
├── context/mod.rs        # Repository context detection
├── db/mod.rs             # DuckDB connection, schema, insert helpers
├── ingest/
│   ├── mod.rs            # Orchestrator: scan sources, dispatch parsers, print summary
│   ├── pricing.rs        # LiteLLM pricing DB fetch, cache, cost calculation
│   ├── praxis_native.rs  # Parse praxis .meta.json files
│   ├── opencode.rs       # Parse OpenCode session + message JSON files
│   ├── claude_code.rs    # Parse Claude Code JSONL (with data quality warnings)
│   └── ccusage.rs        # Import ccusage --json output
├── commands/
│   ├── mod.rs            # Module entry
│   ├── runs.rs           # praxis runs — list/filter runs
│   ├── stats.rs          # praxis stats — aggregate queries
│   ├── discover.rs       # Discover and register workflows
│   ├── collect.rs        # Collect signal data from workflows
│   ├── signals.rs        # praxis signals — list/filter signals
│   ├── status.rs         # praxis status — dashboard overview
│   ├── sync.rs           # Sync workflow state
│   ├── tasks.rs          # Todoist task management
│   ├── triage.rs         # Triage unclassified signals
│   └── workflows.rs      # praxis workflows — list/show workflows
├── registry/mod.rs       # Workflow registry and discovery
│   └── discover.rs       # Auto-discover workflow definitions
├── signals/mod.rs        # Signal collection and triage
│   └── triage.rs         # Signal triage workflow
├── llm/mod.rs            # OpenAI-compatible API client
├── observability/mod.rs  # Token estimation, run metadata
├── storage/mod.rs        # Artifact persistence
├── todoist/mod.rs        # Todoist API client
└── workflows/mod.rs      # Think workflow
```

## Testing

```bash
cargo test
```

Unit tests cover all parsers, the pricing engine, query building, and output formatting. Integration tests verify the full ingest → query cycle with sample fixtures.

## Roadmap

| Phase | What | Status |
|---|---|---|
| 1. Foundation | `think` workflow, artifacts, observability | Complete |
| 2. AI Workflow Observability | `ingest`, `runs`, `stats`, DuckDB analytics | **Current** |
| 3. Model comparison | A/B analysis, "which model works best for what" | Planned |
| 4. Context and memory | RAG over artifacts, cross-run insights | Planned |
| 5. Composition | Workflow chaining, tool plugins | Future |
| 6. Integrations | External system connectors | Future |

## Licence

MIT
