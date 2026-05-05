# Praxis — Vision

**Status:** Phase 3 (Control Plane)
**Last updated:** 2026-05-05

---

## The problem

Technical support at scale generates more signal than any manager can process manually. Tickets, fleet telemetry, SLA clocks, team knowledge, weekly metrics. The standard response is more headcount or more dashboards. Neither scales past a certain point. What I needed was a reasoning layer across all of it, surfacing what actually needs action.

## Thesis

AI should be treated like a system, not a conversation. Applied to operations, that means structured workflows, persistent artifacts, measurable execution, and continuous optimisation instead of guesswork.

Praxis applies systems thinking and observability principles to AI-augmented operations management.

## Phase 2 pivot: AI workflow observability (complete)

Phase 1 (the `think` command) delivered on the structured workflow and observability ideas. Phase 2 extended this into cross-source analytics: ingesting logs from OpenCode, Claude Code, and Praxis itself into DuckDB, with cost calculation via the LiteLLM pricing database.

Key deliverables (shipped):
- `praxis ingest` — scan log directories, normalise, load into DuckDB
- `praxis runs` — list/filter AI activity
- `praxis stats` — aggregates by day, model, project, source
- LiteLLM pricing engine for cost calculation
- Data quality tracking (reliable vs. unreliable token counts)

## Design decisions — Phase 2

### Leverage ccusage, don't reinvent it

[ccusage](https://github.com/ryoppippi/ccusage) and @ccusage/opencode are mature tools that already solve log parsing and cost calculation. Key learnings absorbed:

1. **LiteLLM pricing database** — fetch and cache `model_prices_and_context_window.json`, same source ccusage uses.
2. **OpenCode token data is reliable** — token counts come from API responses, unlike Claude Code's streaming placeholders.
3. **Claude Code JSONL is broken for token accounting** — input tokens are 100-174x too low, output tokens 10-17x too low. This is a known Anthropic bug ([#22686](https://github.com/anthropics/claude-code/issues/22686)). Mark all Claude Code data as unreliable, document it clearly, move on.
4. **ccusage JSON output as ingestion shortcut** — rather than reimplementing Claude Code parsing, accept ccusage `--json` output as an import format.

### Unified schema in DuckDB

DuckDB gives us SQL over local data with no server to run. The schema has three tables: `sessions`, `runs`, `ingestion_log`. Ingestion is incremental — files are tracked by path and mtime, re-ingested only when changed.

### OpenCode is the primary source

OpenCode session/message JSON files contain reliable token counts from API responses. Cost is zero in the files (must be calculated), but tokens are trustworthy. This makes OpenCode the highest-quality data source.

## Who this is for

Technical managers and operators running complex support or reliability functions. People who have built ad-hoc scripts, dashboards, or personal workflows around their operations, and want something coherent that sits across their existing tooling.

Comfortable in a terminal. Already using AI for decision-making, analysis, and planning. Frustrated by the gap between what the tools show and what actually needs attention. Value explicit systems over magical abstractions.

## Core principles

1. **Structured workflows > chat.** Every interaction follows a defined method. The method constrains the output to be useful, not just fluent.
2. **Artifacts > ephemeral answers.** Every run produces a persistent, inspectable output. Thinking accumulates instead of evaporating.
3. **Observability from day one.** Every run generates metadata: cost, duration, token usage, model. You can't optimise what you can't measure.
4. **Local-first, inspectable.** All data stored as files you own. Markdown, JSON, and DuckDB. No cloud dependency, no lock-in, no hidden state.
5. **Human-in-the-loop.** No hidden automation, no autonomous agents. The user initiates every run, reviews every output, decides what to do with it.
6. **Context is explicit.** Local files, repo context, and past artifacts are declared inputs. Praxis should never silently absorb ambient state and pretend it is obvious.

## Design philosophy

Praxis should feel like `git`, `curl`, `jq` — a sharp tool that does one thing well, composes with everything, and never surprises you.

Explicit, not magical. Structured, not verbose. Inspectable, not opaque.

## Context and guardrails

Context access is powerful and dangerous. The default must therefore be conservative.

Principles:

- **No silent repo reading.** The user must opt in to local context access.
- **Default-deny sensitive material.** Env files, credentials, keys, secret folders, and similar inputs should be excluded unless explicitly forced.
- **Provenance in artifacts.** If context was used, the saved artifact should record what was read.
- **Policy over prompts.** Enduring security rules should live in product behavior and configuration, not just in model instructions.
- **Reasoning, not exfiltration.** Praxis should help interpret local context, not become a stealth data export mechanism.

## Evolution path

Each phase is independently useful. No phase requires the next to deliver value.

### Phase 1 — Foundation (complete)

One workflow (`think`), one artifact format, observability from day one.

### Phase 2 — AI Workflow Observability (complete)

Ingest logs from OpenCode and Claude Code into DuckDB. Unified cross-source analytics with cost calculation.

### Phase 3 — Control Plane (current)

Operational decision-making from the observability foundation. Workflow discovery, signal collection, LLM triage, task management, morning status dashboard. See above for deliverables.

### Phase 4 — Model comparison and optimisation (planned)

A/B analysis across models. Usage patterns correlated with outcomes. Anomaly detection (sessions that cost 10x more than normal).

### Phase 5 — Context and memory (planned)

Local RAG over accumulated artifacts. Feed previous thinking into new runs. Graph context linking related decisions.

### Phase 6 — Integrations (planned)

Connectors for external systems. Ingestion pipelines. The system becomes a hub for structured AI interaction across your toolchain.

## Technical decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust | Instant startup, single binary, type safety, fits the "sharp tool" philosophy |
| LLM protocol | OpenAI-compatible API | Works with Ollama, LM Studio, vLLM, llama.cpp, and cloud providers |
| Default backend | Local (Ollama) | Cost control, privacy, no API key needed to start |
| Storage (artifacts) | Flat files (markdown + JSON) | Inspectable, scriptable, no migrations, composable with Unix tools |
| Storage (analytics) | DuckDB | Analytical SQL, zero server overhead, excellent performance for local data |
| Pricing data | LiteLLM pricing DB | Comprehensive, maintained, same source as ccusage |
| Config | TOML + env vars | Standard CLI pattern — config file for defaults, env vars for overrides |

## What would kill this project

1. **Not using it.** If the tool sits unused after a week, it's failed. The UX of a single run must justify the command.
2. **Scope creep before validation.** Building RAG and integrations before the core loop works daily.
3. **Prompt mediocrity.** If the workflows produce generic, verbose, hedge-everything output, the structured format is just overhead. The prompts are the product.

## What would validate Phase 3

1. **Daily `praxis sync && praxis status`.** Running this every morning surfaces the right things to act on.
2. **Signal triage reduces morning review time.** The five things that matter are already surfaced, not buried in Slack and Jira.
3. **Workflow health visibility.** Knowing which operational reports are stale before someone complains.

## Differentiation

The closest tool in this space is [ccusage](https://github.com/ryoppippi/ccusage) for log parsing and cost calculation. Praxis differs in three ways: it is a control plane for operations (not just cost tracking), it connects workflow outputs to triage and task management, and it produces a prioritised action dashboard rather than raw analytics. The signal layer is the moat: structured outputs from real operational workflows, triaged by LLM, surfaced as action items.
