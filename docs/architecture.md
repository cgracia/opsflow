# Architecture

OpsFlow is a structured investigation pipeline that converges signals from tickets, alerts, and events into governed investigations across a hierarchical entity model. This document describes how the system is built, why key decisions were made, and how data flows through the pipeline.

> **Note:** This is an alpha-stage personal project. See the "Current implementation status" section below for what is real vs. stubbed.

## Current implementation status

| Component | Status | Notes |
|-----------|--------|-------|
| 7-phase orchestrator | Working | Runs end-to-end on synthetic data |
| Entity model (10 types) | Working | SQLAlchemy models, seeded via `/seed` endpoint |
| Hybrid retrieval | Architecture real, vectors fake | Qdrant RRF fusion works; dense vectors are SHA-256 hashes, sparse vectors are word-count bags |
| Telemetry specialist | Working (rule-based) | Keyword matching on evidence content; LLM optional |
| Historical specialist | Working (rule-based) | Pattern detection from historical tickets; LLM optional |
| Governance engine | Working | EXECUTE blocked, severity/sensitivity gating functional |
| Hypothesis generation | Working (rule-based fallback) | LLM path supported; defaults to rule-based when no LLM key |
| Output generation | Working | Template-based operator briefing and customer response draft |
| Langfuse tracing | Working | Every phase emits spans with evidence, hypotheses, governance |
| Signal connectors | Stub | Receives signal IDs, does not connect to external systems |
| Entity resolution | Demo-shaped | Hardcoded for the seed scenario; not tested on ambiguous graphs |
| Time-bounded retrieval | Stub | `search_time_window` exists but does not filter by time |

## System Overview

Distributed technical operations produce more signal than any team can process manually. Tickets arrive from customer channels. Alerts fire from fleet monitoring. Events stream from deployment pipelines and device telemetry. Each signal type lives in its own system with its own data model and its own timeline.

OpsFlow treats these as different entry points into the same problem space. A navigation error reported as a ticket, an anomaly alert from fleet telemetry, and a deployment event are not three separate things to triage independently. They are three signals pointing at the same incident, and the system converges them, reasons across the entity graph, retrieves relevant evidence, and produces a bounded, traceable output.

The core constraint: the system investigates and recommends. It does not autonomously execute changes against production. Every output passes through a governance layer. EXECUTE actions are blocked. Human-in-the-loop escalation is mandatory for high-severity customer-facing incidents.

## Component Architecture

### FastAPI API (`python/app/api/`)

The entry point. Exposes a REST API on port 8000 with three primary endpoints:

- `GET /api/v1/healthz` - Service health check
- `POST /api/v1/seed` - Load synthetic entity data and evidence into Postgres and Qdrant
- `POST /api/v1/investigations` - Accept a `SignalIds` payload and return a full `InvestigationResponse`

The API layer is thin. It validates the request, resolves dependencies (Qdrant client, LLM client, database session), and delegates to the `InvestigationManager`.

### Orchestrator (`python/app/orchestrator/`)

The central coordinator. `InvestigationManager` owns the investigation lifecycle and executes seven phases in fixed order:

1. Signal Ingestion
2. Entity Resolution
3. Evidence Retrieval
4. Specialist Investigation
5. Hypothesis Generation
6. Governance Evaluation
7. Output Generation

The orchestrator is not an autonomous agent. It is a structured workflow engine with defined phases, typed inputs and outputs at each stage, and a single control flow. This is intentional. The domain demands predictability and inspectability over flexibility. Every phase emits a trace span so the full reasoning chain is visible after the fact.

### Specialists (`python/app/specialists/`)

Domain-scoped investigator tools, each responsible for a narrow analysis task:

**Telemetry Investigator** (`telemetry.py`). Queries Qdrant for telemetry evidence filtered by device and fleet IDs. Analyzes retrieved snapshots using rule-based keyword detection — looks for terms like `navigation_error_rate`, `sensor_fusion_latency`, and temporal correlation keywords. Returns a structured `TelemetryReport` with findings, anomalies, an event timeline, and a confidence score derived from evidence strength.

**Historical Incident Investigator** (`historical.py`). Searches Qdrant for past tickets, runbooks, and deployment records across three separate queries. Identifies recurring patterns through keyword matching (navigation failures, sensor fusion issues, SLA impacts), detects deployment adjacency from halted deployment metadata, and returns a `HistoricalReport` with similar incidents, recurring patterns, and known issues.

Both specialists can operate with or without an LLM. The current implementation uses the rule-based path by default. With an LLM configured, they can perform deeper reasoning over the retrieved evidence. This fallback design ensures the system degrades gracefully.

### Retrieval Layer (`python/app/retrieval/`)

Hybrid search over Qdrant combining dense vectors and sparse vectors, fused via reciprocal rank fusion (RRF).

The `search_evidence` function sends two parallel prefetch requests to Qdrant: a dense vector search for semantic similarity and a sparse vector search for keyword relevance. Qdrant merges the results using RRF, which balances recall from dense search with precision from keyword search.

**Current implementation details:**

- Dense vectors are 384-dimensional, generated deterministically from document IDs via SHA-256 hash. They are not semantically meaningful — the architecture supports real embeddings, but no embedding provider is wired yet.
- Sparse vectors are word-count bags derived by splitting the query on whitespace. Not BM25 — functional for the demo scenario, but not production retrieval quality.
- Filtering is first-class. Every search accepts `entity_ids` and `source_types` parameters that become Qdrant `Filter` conditions on payload metadata.

The hybrid approach exists because this domain has retrieval needs that no single strategy covers:

- Exact keyword retrieval for device IDs, version numbers, error codes
- Semantic retrieval for runbooks, historical incident descriptions, and natural-language documentation
- Entity-filtered retrieval for scoped investigation within the entity graph
- Time-bounded retrieval for incident windows (planned, not yet implemented)

### Governance Engine (`python/app/governance/`)

The policy layer that bounds what the system can output and when it must escalate.

The engine classifies every investigation into one of five action categories:

| Category | Meaning | Allowed Tools |
|----------|---------|---------------|
| INVESTIGATE | Still gathering evidence | retrieve_evidence, query_telemetry, query_history |
| RECOMMEND | Enough evidence to suggest next steps | All investigate tools + draft_recommendation |
| ESCALATE | Requires human attention | All investigate tools + notify_operator, draft_escalation |
| COMMUNICATE | Safe to generate customer-facing output | All investigate tools + draft_customer_response |
| EXECUTE | Would take autonomous action | Always blocked in v1 |

Classification depends on severity, customer sensitivity, and evidence confidence. Low-confidence investigations get restricted outputs. High-severity customer-facing incidents force escalation. Enterprise-tier accounts are treated as VIP sensitivity.

Additional rules:
- Evidence confidence below 0.3 blocks recommendation and customer response generation
- High severity plus customer-facing or VIP sensitivity triggers mandatory escalation
- EXECUTE is hardcoded as always-blocked in v1, regardless of classification result

### Tracing (`python/app/tracing/`)

Every investigation phase emits a span to Langfuse. The trace captures:
- Phase name and status (start, complete, error)
- Evidence retrieved (source type, entity, relevance score)
- Hypotheses generated (description, confidence, evidence references)
- Governance decision (classification, approved/blocked actions, escalation status)
- LLM prompts and responses (when LLM is configured)
- Token usage and latency

Langfuse runs as a local service (port 3000) backed by Postgres, ClickHouse for analytics, and Redis for job processing. This gives you a full local observability stack without sending data to external services.

## Entity Model

The system operates over ten entity types that model a distributed technical operation:

```
Account
  └── Site
        ├── Fleet
        │     └── Device
        │           └── Service
        ├── Deployment
        │     └── SoftwareRevision
        └── Incident
              ├── Ticket
              └── OperationalEvent
```

**Operational hierarchy (Account through Device):** Models the physical and organizational structure. An account (customer) has sites. Sites have fleets. Fleets contain devices. Devices run services. This hierarchy is the primary scoping mechanism for investigations. When a device reports a navigation error, the system resolves the full chain: which fleet, which site, which account.

**Software and deployments:** A Deployment tracks a software rollout to a fleet or site. It links to a SoftwareRevision that carries the version number. Deployment adjacency (did a software rollout coincide with the incident?) is one of the strongest signals in the historical investigator.

**Incidents and signals:** An Incident represents a converged event. Tickets, alerts, and operational events are signal types that attach to the entity graph. Multiple signals pointing at overlapping entities get merged into a single investigation.

The entity model is stored in PostgreSQL via SQLAlchemy 2.0 models (async). Each model has typed columns, relationships, and Pydantic schemas for API serialization.

## Investigation Flow in Detail

A concrete walkthrough of what happens when you `POST /api/v1/investigations` with three signal IDs:

**Phase 1: Signal Ingestion.** The orchestrator receives a `SignalIds` object containing `ticket_id: "TCK-1001"`, `alert_id: "ALT-2001"`, and `event_id: "EVT-3001"`. These are validated and stored as the signal set for this investigation.

**Phase 2: Entity Resolution.** The system resolves which entities these signals relate to. For the seeded demo data, this produces an `EntityContext` containing: Account "Meridian Logistics" (enterprise tier), Site "Portland Distribution Center", Fleet "Warehouse Alpha Fleet" (FLT-101), three Devices (DEV-401 in error state, DEV-402 and DEV-403 in degraded state, all on software v3.3.0), an in-progress Deployment (DEPL-501, v3.3.0), and SoftwareRevision SWREV-302.

**Phase 3: Evidence Retrieval.** The system queries Qdrant with a hybrid search. The query combines the signal context ("navigation error device blocked anomaly alert") with entity ID filters (ACC-1001, FLT-101, DEV-401, DEV-402, DEV-403). Up to 15 evidence items are returned, each with source type, entity reference, content, and relevance score.

**Phase 4: Specialist Investigation.** Two specialists run against the resolved entities. The Telemetry Investigator searches for telemetry snapshots for DEV-401 and FLT-101, looking for navigation error rates, sensor fusion latency, and temporal correlations with deployment changes. The Historical Investigator searches past tickets, runbooks, and deployment records for similar incidents and recurring patterns, checking specifically for deployment adjacency.

**Phase 5: Hypothesis Generation.** Evidence, telemetry findings, and historical patterns are synthesized into ranked hypotheses. If an LLM is available, structured generation produces hypotheses with descriptions, confidence scores, and evidence references. Without an LLM, rule-based fallback generates hypotheses from the pattern data (e.g., "Software version v3.3.0 introduced a regression in the navigation engine" at 0.85 confidence if telemetry anomalies and deployment adjacency are both detected).

**Phase 6: Governance Evaluation.** The governance engine classifies the action based on severity, customer sensitivity, and evidence confidence. For an enterprise account with high severity, this typically results in ESCALATE classification with mandatory human-in-the-loop review. EXECUTE is always blocked.

**Phase 7: Output Generation.** Two bounded outputs are produced. The operator briefing contains the full investigation context, primary hypothesis, telemetry findings, historical patterns, and governance decision. The customer response draft is a safe, non-technical summary suitable for external communication.

## Observability

### Langfuse Traces

Every investigation creates a top-level trace with nested spans for each phase. The trace captures the full reasoning chain: what evidence was retrieved, what the specialists found, what hypotheses were generated, what governance decided, and what outputs were produced. Traces are queryable in the local Langfuse UI at `localhost:3000`.

### Prometheus Metrics

The API exposes a `/metrics` endpoint for Prometheus scraping. Service-level metrics (request latency, error rates, investigation duration) are collected automatically.

### Grafana Dashboards

Pre-provisioned Grafana dashboards at `localhost:3100` visualize operational metrics. Dashboard configuration lives in `infra/grafana/dashboards/` and provisioning in `infra/grafana/provisioning/`.

## Design Decisions

### Why hybrid retrieval, not semantic-only?

Pure semantic search misses exact matches on device IDs, version numbers, and error codes. Pure keyword search misses conceptually related runbooks and historical incidents described in different terms. Reciprocal rank fusion over dense + sparse vectors handles both. Qdrant supports this natively with prefetch and fusion, so the implementation is clean. The current limitation is that dense vectors are deterministic hashes rather than real embeddings — the architecture is ready for an embedding provider, but one is not wired yet.

### Why fixed phases, not dynamic agent planning?

Operational investigations in this domain follow a predictable pattern: figure out what happened, figure out what it affects, gather evidence, consult specialists, form a theory, check policy, produce output. Making this dynamic adds complexity without adding value. Fixed phases mean every investigation is structurally comparable, every trace has the same shape, and debugging is straightforward. If a phase produces unexpected output, you know exactly where to look.

### Why no autonomous agents?

The domain constrains this. A wrong recommendation wastes time. A wrong autonomous action in a fleet of physical devices can cause real damage. The system is designed as a structured workflow that produces bounded, inspectable outputs. Human operators review and decide. The governance layer enforces this: EXECUTE is blocked, escalation is mandatory for high-severity customer-facing incidents, and low-confidence investigations get restricted outputs.

### Why a single orchestrator, not multi-agent?

Following the guidance from Anthropic and OpenAI on production agent patterns: maximize a single agent's capabilities first, use specialist tools rather than specialist agents, and only introduce multi-agent patterns when the complexity justifies the orchestration overhead. For the current scope, one orchestrator calling two specialist tools is the right granularity. Each specialist has a clear boundary, typed interface, and can be tested independently.

### Why entity-centric, not ticket-centric?

Tickets are one signal type among many. An incident might surface first as a telemetry anomaly, then as a customer ticket, then as a deployment event. Centering on entities (Account, Site, Fleet, Device) rather than tickets means the system can reason about what is affected regardless of which signal triggered the investigation. This is the same direction the market is moving: entity graphs, not ticket queues.
