# OpsFlow

**AI-Native Operational Investigation and Orchestration Platform**

OpsFlow is a governed operational intelligence layer for distributed technical systems. It converges tickets, alerts, and events into structured investigations that reason across your entity hierarchy, retrieve evidence from multiple sources, and produce bounded outputs with full traceability. The system investigates, explains, and recommends. It does not act on its own.

## What It Does

- **Multi-signal convergence investigation.** A ticket, an alert, and an operational event enter as separate signals. OpsFlow merges them into a single incident, resolves the affected entities, and runs a coordinated investigation.
- **Entity-centric reasoning.** Every investigation operates across the Account, Site, Fleet, Device, Deployment hierarchy. The system understands that a navigation error on three devices in one fleet during a software rollout is not three separate problems.
- **Hybrid evidence retrieval.** Dense semantic search and sparse keyword matching fused via reciprocal rank fusion in Qdrant. Filter by entity, source type, or time window. No single retrieval strategy covers this domain well enough on its own.
- **Policy-bounded governance.** Every investigation output passes through a governance layer that classifies the action, gates sensitive operations, and enforces human-in-the-loop escalation. EXECUTE actions are blocked in v1. The system does not autonomously change production state.
- **Full trace observability.** Every phase of every investigation emits spans to Langfuse: prompts, tool calls, evidence retrieved, hypotheses generated, governance decisions. You can inspect, debug, and evaluate the reasoning chain end to end.

## Architecture

```
  Signal Sources                OpsFlow Engine                         Supporting Services
  ─────────────                 ─────────────                          ────────────────────

  Tickets ──┐
  Alerts ───┤    ┌──────────┐   ┌──────────────┐   ┌──────────────┐
  Events ───┼───>│ FastAPI   │──>│Orchestrator   │──>│ Governance   │──> Operator Briefing
             │   │ API       │   │(7 phases)     │   │ Engine       │──> Customer Draft
  Docs ─────┤    │ :8000     │   │               │   └──────────────┘
  Runbooks ─┤    └──────────┘   │  ┌──────────┐  │
  Telemetry─┤                   │  │Specialist │  │   Postgres ─────── Entities & relations
  Logs ─────┘                   │  │ Tools     │  │   Qdrant ───────── Evidence store (RRF)
                                │  └──────────┘  │   Langfuse ──────── Traces & observability
  ┌──────────────┐              │                │   Grafana ────────── Operational dashboards
  │ Rust CLI     │              │  ┌──────────┐  │   Prometheus ─────── Metrics
  │ (praxis)     │              │  │ Retrieval │  │
  │ analytics    │              │  │ (hybrid)  │  │   Docker Compose brings it all up
  └──────────────┘              │  └──────────┘  │
                                └──────────────┘
```

The Rust CLI (`praxis`) is a standalone analytics tool for AI workflow observability. It ingests logs from OpenCode, Claude Code, and similar tools into DuckDB for cost and usage analysis. It runs independently of the Python investigation engine.

## Quick Start

```bash
# Clone and configure
git clone https://github.com/carlos/opsflow.git && cd opsflow
cp .env.example .env
# Edit .env: set LLM_API_KEY (OpenAI-compatible)
# Edit .env: optionally set LANGFUSE_PUBLIC_KEY and LANGFUSE_SECRET_KEY for investigation tracing

# Start all services
docker compose up -d

# Verify the API is healthy
curl -sf http://localhost:8000/api/v1/healthz

# Seed synthetic entity data and evidence
curl -sf -X POST http://localhost:8000/api/v1/seed

# Run a multi-signal investigation
curl -sf -X POST http://localhost:8000/api/v1/investigations \
  -H 'Content-Type: application/json' \
  -d '{"signal_ids": {"ticket_id": "TCK-1001", "alert_id": "ALT-2001", "event_id": "EVT-3001"}}'
```

Ports: API at `:8000`, Qdrant at `:6333`, Langfuse at `:3000`, Grafana at `:3100`, Postgres at `:5432`.

## Investigation Flow

Every investigation follows seven fixed phases, executed in order:

1. **Signal Ingestion.** Receive ticket, alert, and event identifiers. Validate and normalize into a unified signal set.
2. **Entity Resolution.** Map signals to the affected entity graph: Account, Site, Fleet, Devices, Deployment, Software Revision.
3. **Evidence Retrieval.** Query Qdrant with hybrid search (dense + sparse, fused via RRF), filtered by entity IDs and source types.
4. **Specialist Investigation.** Dispatch to domain-specific investigators. The Telemetry Investigator analyzes device metrics. The Historical Incident Investigator searches past incidents, runbooks, and deployment records.
5. **Hypothesis Generation.** Synthesize evidence, telemetry findings, and historical patterns into ranked hypotheses with confidence scores.
6. **Governance Evaluation.** Classify the action (INVESTIGATE, RECOMMEND, ESCALATE, COMMUNICATE, EXECUTE). Gate sensitive operations. Flag human-in-the-loop escalation for high-severity customer-facing incidents.
7. **Output Generation.** Produce two bounded outputs: an internal operator briefing and a customer-safe response draft.

## Stack

| Component | Technology |
|-----------|-----------|
| API | Python 3.12, FastAPI, Pydantic v2 |
| ORM | SQLAlchemy 2.0 (async) |
| Entity store | PostgreSQL 17 |
| Evidence store | Qdrant (hybrid dense + sparse vectors) |
| LLM | OpenAI-compatible API (configurable) |
| Tracing | Langfuse 3 |
| Metrics | Prometheus |
| Dashboards | Grafana |
| CLI | Rust (praxis) |
| Runtime | Docker Compose |

## Status

Alpha, in active development.

The investigation pipeline is end to end: signal ingestion through output generation, with governance gating and full trace emission. The entity model covers ten types. Hybrid retrieval works. Two specialist tools are operational. What comes next: richer connectors, cross-incident learning, deployment correlation, and evaluation harnesses.

## License

MIT
