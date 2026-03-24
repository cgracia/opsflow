# Praxis — Vision

**Status:** Phase 1 (MVP)
**Last updated:** 2026-03-24

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

## Who this is for

Technical professionals who already use AI heavily and feel the pain of unstructured usage. People who have built ad-hoc scripts, prompt templates, or personal workflows around AI — and want something coherent.

Comfortable in a terminal. Already using AI for decision-making, analysis, and planning. Frustrated by the lack of structure, persistence, and visibility. Value explicit systems over magical abstractions.

Not for people who want a chatbot, a no-code builder, or a general-purpose assistant.

## Core principles

1. **Structured workflows > chat.** Every interaction follows a defined method. The method constrains the output to be useful, not just fluent.
2. **Artifacts > ephemeral answers.** Every run produces a persistent, inspectable output. Thinking accumulates instead of evaporating.
3. **Observability from day one.** Every run generates metadata: cost, duration, token usage, model. You can't optimise what you can't measure.
4. **Local-first, inspectable.** All data stored as files you own. Markdown and JSON. No cloud dependency, no lock-in, no hidden state.
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

Long-term, this likely becomes a persistent local policy layer: safe roots, denied paths, approved context modes, and auditable context provenance for every run.

## Evolution path

Each phase is independently useful. No phase requires the next to deliver value.

### Phase 1 — Foundation (current)

One workflow (`think`), one artifact format, observability from day one.

*Validation: do I reach for `praxis think` instead of opening a chat window?*

### Phase 2 — Workflow expansion

Additional workflows: `decide` (weighted decision matrix), `plan` (goal decomposition), `review` (critique an artifact), `summarise` (structured extraction).

*Validation: do I have workflows I use weekly? Do the artifacts accumulate into something I reference?*

### Phase 3 — Observability dashboard

Structured index over run metadata (DuckDB or SQLite). CLI queries: `praxis stats`, `praxis compare`. Cost tracking, model comparison, duration trends.

*Validation: can I answer "how much did AI cost me this month?" and "which model gives better results for decision workflows?"*

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
| Storage | Flat files (markdown + JSON) | Inspectable, scriptable, no migrations, composable with Unix tools |
| Config | TOML + env vars | Standard CLI pattern — config file for defaults, env vars for overrides |

## What would kill this project

1. **Not using it.** If the tool sits unused after a week, it's failed. The UX of a single run must justify the command.
2. **Scope creep before validation.** Building RAG and integrations before the core loop works daily.
3. **Prompt mediocrity.** If the workflows produce generic, verbose, hedge-everything output, the structured format is just overhead. The prompts are the product.

## What would validate it

1. **Daily personal use for 30 days.** Reaching for `praxis think` reflexively when facing a decision.
2. **Artifact reference.** Going back to a previous run to inform a current decision — at least once.
3. **Observability insight.** The metadata reveals something non-obvious about model performance, cost, or workflow effectiveness.

## Differentiation

The closest tool in this space is [fabric](https://github.com/danielmiessler/fabric) — structured prompts via CLI. Praxis differs in three ways: built-in observability (every run is measured and tracked), persistent artifacts with structured metadata (not just stdout), and a deliberate evolution toward context-aware reasoning (Phase 4+). The observability layer is the moat — it turns personal AI usage into a dataset you can learn from.
