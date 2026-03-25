# Praxis — Vision

**Status:** Phase 2 (AI Workflow Observability)
**Last updated:** 2026-03-25

---

## The problem

AI usage today is unstructured, ephemeral, and opaque.

Chat-based interaction produces inconsistent quality. Valuable reasoning disappears when the conversation ends. There's no visibility into cost, performance, or output quality. No feedback loops. No way to know whether a local model would have been sufficient, whether the same prompt performs better with a different model, or how much you spent this month on things you could have done in your head.

Most tools in this space are building agents — autonomous systems that act on your behalf. Praxis goes the other direction: structured tools that help *you* think better, with full observability into what happened and what it cost.

## Thesis

AI should be treated like a system, not a conversation.

That means:

- **Structured workflows** instead of freeform prompts
- **Persistent artifacts** instead of ephemeral answers
- **Measurable execution** instead of hidden behaviour
- **Continuous optimisation** instead of guesswork

Praxis applies systems thinking and observability principles to personal AI usage.

## Phase 2 pivot: the real value is observability

Phase 1 (the `think` command) delivered on the structured workflow and observability ideas. But the real insight from Phase 1 usage is that the *data* is more valuable than the *workflow tool*.

**The thesis for Phase 2:** instrument actual AI usage across all your tools, index it, and make it queryable. Don't limit Praxis to its own runs — absorb everything. OpenCode, Claude Code, and Praxis native runs all generate logs. Those logs contain the data you need to answer real questions.

Phase 2 makes Praxis the observability layer for AI-assisted work, not just a structured prompting tool.

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

Technical professionals who already use AI heavily and feel the pain of unstructured usage. People who have built ad-hoc scripts, prompt templates, or personal workflows around AI — and want something coherent.

Comfortable in a terminal. Already using AI for decision-making, analysis, and planning. Frustrated by the lack of structure, persistence, and visibility. Value explicit systems over magical abstractions.

Not for people who want a chatbot, a no-code builder, or a general-purpose assistant.

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

*Validation: did I reach for `praxis think` instead of opening a chat window?*

### Phase 2 — AI Workflow Observability (current)

Ingest logs from OpenCode and Claude Code into DuckDB. Query that data: `praxis runs`, `praxis stats`. Unified cross-source analytics in a local database.

Key deliverables:
- `praxis ingest` — scan log directories, normalise, load into DuckDB
- `praxis runs` — list/filter AI activity
- `praxis stats` — aggregates by day, model, project, source
- LiteLLM pricing engine for cost calculation
- Data quality tracking (reliable vs. unreliable token counts)

*Validation: can I answer "how much did AI cost me this week" and "which model am I using most" with real data?*

### Phase 3 — Model comparison and optimisation

A/B analysis across models. "Which model gives better results for which kind of task?" Usage patterns correlated with outcomes. Anomaly detection (sessions that cost 10x more than normal).

*Validation: does the data reveal something actionable about model choice or usage patterns?*

### Phase 4 — Context and memory

Local RAG over accumulated artifacts. Feed previous thinking into new runs. Graph context linking related decisions. Explicit project context, policy-controlled file access, and artifact provenance become first-class concepts.

*Validation: does Praxis surface relevant past thinking when working on a related problem?*

### Phase 5 — Tool execution and composition

Workflow composition (output of one feeds input of another). Plugin system for external data sources. Script execution within workflows.

*Validation: can I build a pipeline that pulls data, reasons about it, and produces a recommendation — in one invocation?*

### Phase 6 — Integrations

Connectors for external systems. Ingestion pipelines. The system becomes a hub for structured AI interaction across your toolchain.

*Validation: does Praxis replace the ad-hoc scripts I've built for work automation?*

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

## What would validate Phase 2

1. **Daily `praxis ingest && praxis stats`.** Running this every morning feels like something worth doing.
2. **Surprise insights.** The data reveals something non-obvious about model usage, cost, or workflow patterns.
3. **Real cost visibility.** "How much did AI cost me this week?" answered with real data, not estimates.
4. **Project attribution.** "Which of my projects is most AI-intensive?" answered from the database.

## Differentiation

The closest tool in this space is [ccusage](https://github.com/ryoppippi/ccusage) — log parsing and cost calculation. Praxis differs in three ways: unified cross-source analytics in a persistent database (not per-session reports), structured reasoning workflows with artifacts, and a deliberate evolution toward model comparison and cross-run memory. The database layer is the moat — it turns personal AI usage into a dataset you can learn from over time.
