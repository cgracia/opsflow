# OpsFlow Evolution — AI-Native Operational Control Plane

## TL;DR

> **Quick Summary**: Evolve the existing Rust CLI (praxis) into an AI-native operational control plane by adding a Python investigation engine (FastAPI) that demonstrates one deeply convincing multi-signal convergence investigation flow — support ticket + anomaly alert + blocked device resolved as a single incident with entity-centric reasoning, hybrid retrieval, governed orchestration, and full trace observability.
>
> **Deliverables**:
> - Python investigation engine (`python/`) with FastAPI, SQLAlchemy, Pydantic
> - 10 entity models in Postgres with synthetic data generators
> - Qdrant hybrid retrieval across tickets, docs, telemetry, runbooks
> - Single control-plane orchestrator coordinating investigation phases
> - 2 specialist investigator tools (telemetry + historical incident)
> - Policy/governance layer with severity-aware gating
> - Langfuse-traced investigation flow (the primary demo artifact)
> - Docker Compose stack (Postgres, Qdrant, Langfuse, Grafana, Prometheus, API)
> - Grafana operational dashboards (provisioned, secondary artifact)
> - Updated README + architecture documentation
> - Preserved Rust CLI (untouched)
>
> **Estimated Effort**: Large (XL)
> **Parallel Execution**: YES — 5 waves
> **Critical Path**: T1 (scaffold) → T3 (entities) → T5 (synthetic data) → T7 (Qdrant indexing) → T8 (orchestrator) → T11 (API endpoint) → T13 (compose) → T15 (README) → F1-F4

---

## Context

### Original Request
Evolve the existing opsflow Rust CLI repository into a high-signal public artifact demonstrating AI-native operational investigation for complex technical operations. The system should feel like "a simplified but realistic operational substrate inspired by real-world high-stakes operations."

### Interview Summary
**Key Discussions**:
- **Stack direction**: Hybrid — Rust CLI preserved as local analytics component, Python FastAPI service added for investigation engine, Docker Compose ties them together
- **Investigation scenario**: Multi-signal convergence — support ticket + anomaly alert + blocked device arrive simultaneously, system resolves they're the same incident
- **Entity model**: 10 types generalized from robotics-specific to distributed technical systems (Device not Robot)
- **Specialists**: 2 for v1 — telemetry investigator + historical incident investigator
- **UI approach**: API-only + Langfuse traces (primary) + Grafana dashboards (secondary), no custom frontend
- **Test strategy**: TDD + Agent QA, pytest for Python
- **LLM provider**: OpenAI-compatible API, configurable via env

**Research Findings**:
- Existing Rust CLI has 31 source files, ~80% working with comprehensive tests across 14 modules
- Langfuse Python SDK v4 provides nested traces, evidence metadata, context managers + decorators
- Qdrant supports hybrid search (dense+sparse vectors) with RRF fusion and entity-centric filtering
- Docker Compose stack needs ~8 containers (Postgres, Qdrant, Langfuse web+worker, ClickHouse, Redis, Prometheus, Grafana, FastAPI)
- Market whitespace: no vendor covers full stack (Pylon account context + Datadog reasoning + fleet-native artifacts)

### Metis Review
**Identified Gaps** (addressed):
- **Demo entry point**: Single API call `POST /investigations` with synthetic signal IDs
- **Rust-Python contract**: Pure coexistence in v1 — no Rust calling Python. Rust untouched.
- **Canonical input**: Three signals in one API call; system correlates across entities
- **Governance scope**: Classification + gating + explanation only. Severity determines auto vs HITL threshold.
- **Embeddings**: Deterministic fake vectors for tests, configurable model for real runs
- **Langfuse in tests**: Mocked by default, integration tests verify trace_id presence in response
- **Degraded behavior**: Out of scope for v1 — prototype assumes all services available
- **False convergence / partial evidence**: Edge cases handled in governance layer (low-confidence → uncertain briefing)
- **Seed idempotency**: Seeder clears before inserting; deterministic IDs

---

## Work Objectives

### Core Objective
Build ONE deeply convincing multi-signal convergence investigation flow that demonstrates entity-centric reasoning, hybrid retrieval, governed orchestration, and full trace observability for distributed technical operations.

### Concrete Deliverables
- `python/` package with FastAPI app, SQLAlchemy models, Pydantic schemas
- `docker-compose.yml` at repo root with full stack
- Qdrant collections seeded with synthetic operational data
- Investigation API endpoint returning traceable, evidence-linked results
- Langfuse traces showing every step of the investigation
- Grafana dashboards showing operational metrics
- Updated README.md with positioning and walkthrough
- Architecture documentation

### Definition of Done
- [ ] `docker compose up -d` brings all services healthy
- [ ] `curl -X POST /investigations` with synthetic signal IDs returns a complete investigation with entity resolution, evidence, specialist outputs, hypothesis, governance decision, operator briefing, customer-safe response, and trace_id
- [ ] Langfuse UI shows the full investigation trace with nested spans
- [ ] Grafana shows operational metrics
- [ ] `pytest python/tests/ -q` passes all tests
- [ ] README walkthrough reproducible from clean environment
- [ ] No existing Rust code behavior changed

### Must Have
- Entity-centric reasoning across Account → Site → Fleet → Device → Deployment
- Hybrid retrieval (semantic + keyword + entity filter) from Qdrant
- Single control-plane orchestrator with fixed investigation phases
- 2 specialist investigator tools invoked by orchestrator
- Policy/governance layer with severity classification and HITL gating
- Full Langfuse trace emission (every LLM call, tool call, retrieval, policy decision)
- Deterministic synthetic data that tells one coherent incident story
- Operator briefing output (internal)
- Customer-safe response draft output (external-facing)
- Zero human intervention in QA verification

### Must NOT Have (Guardrails)
- NO custom frontend
- NO third specialist, remediation executor, or deployment investigator in v1
- NO full policy engine / rules DSL — governance is classification + gating + explanation
- NO auth, multi-user, RBAC, SaaS tenancy
- NO Rust code behavior changes
- NO background workers/queues/event bus
- NO streaming API responses
- NO real external integrations (ticketing, paging, messaging)
- NO bidirectional Rust↔Python orchestration in v1
- NO evaluation pipelines or benchmark suites beyond pytest
- NO over-abstracted agent framework — single orchestrator, specialist tools, fixed phases

---

## Verification Strategy (MANDATORY)

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: NO (new Python service)
- **Automated tests**: YES (TDD)
- **Framework**: pytest
- **TDD flow**: Each task follows RED (failing test) → GREEN (minimal impl) → REFACTOR

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **API endpoints**: Use Bash (curl) — Send requests, assert status + response fields
- **Python modules**: Use Bash (pytest) — Run tests, verify pass/fail
- **Docker services**: Use Bash (docker compose) — Start, healthcheck, verify logs
- **Grafana**: Use Bash (curl) — Verify provisioning, datasource, dashboard APIs

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — foundation + scaffolding):
├── Task 1: Python service scaffold + config + health endpoint [quick]
├── Task 2: Postgres foundation (SQLAlchemy base + Alembic + session) [quick]
├── Task 3: Entity models — all 10 types + relationships [unspecified-high]
├── Task 4: Pydantic schemas (request/response models) [quick]
├── Task 5: Synthetic data seeders — one coherent incident narrative [deep]
└── Task 6: Dockerfile + docker-compose.yml skeleton [quick]

Wave 2 (After Wave 1 — retrieval + specialists, MAX PARALLEL):
├── Task 7: Qdrant client + collection setup + hybrid retrieval [unspecified-high]
├── Task 8: Evidence indexer — seed Qdrant from synthetic data [unspecified-high]
├── Task 9: Telemetry investigator specialist tool [deep]
├── Task 10: Historical incident investigator specialist tool [deep]
├── Task 11: LLM client + prompt templates (OpenAI-compatible) [unspecified-high]
└── Task 12: Policy/governance engine — classification + gating [unspecified-high]

Wave 3 (After Wave 2 — orchestration + integration):
├── Task 13: Control-plane orchestrator (investigation manager) [deep]
├── Task 14: Langfuse trace emission wiring [unspecified-high]
├── Task 15: Investigation API endpoint (POST /investigations) [unspecified-high]
└── Task 16: Seed command + end-to-end investigation test [deep]

Wave 4 (After Wave 3 — infrastructure + documentation):
├── Task 17: Full docker-compose.yml with all services [unspecified-high]
├── Task 18: Grafana dashboards + provisioning [visual-engineering]
├── Task 19: README.md rewrite + architecture documentation [writing]
└── Task 20: .env.example + setup guide + demo walkthrough [quick]

Wave FINAL (After ALL tasks — 4 parallel reviews, then user okay):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Real manual QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: T1 → T3 → T5 → T8 → T13 → T16 → T17 → F1-F4
Parallel Speedup: ~65% faster than sequential
Max Concurrent: 6 (Waves 1 & 2)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| T1   | —         | T2-T6, Wave 2+ | 1 |
| T2   | T1        | T3 | 1 |
| T3   | T2        | T4, T5, T7, T9, T10 | 1 |
| T4   | T3        | T11, T15 | 1 |
| T5   | T3        | T7, T8, T9, T10 | 1 |
| T6   | T1        | T17 | 1 |
| T7   | T3        | T8, T13 | 2 |
| T8   | T5, T7    | T13, T16 | 2 |
| T9   | T3, T5, T11 | T13 | 2 |
| T10  | T3, T5, T11 | T13 | 2 |
| T11  | T1, T4    | T9, T10, T12, T13 | 2 |
| T12  | T4, T11   | T13 | 2 |
| T13  | T7-T12    | T14, T15, T16 | 3 |
| T14  | T13       | T15, T16 | 3 |
| T15  | T13, T14  | T16 | 3 |
| T16  | T14, T15  | T17, T19 | 3 |
| T17  | T6, T16   | T18, T20 | 4 |
| T18  | T17       | F1-F4 | 4 |
| T19  | T16       | F1-F4 | 4 |
| T20  | T17       | F1-F4 | 4 |

### Agent Dispatch Summary

- **Wave 1**: **6 tasks** — T1 → `quick`, T2 → `quick`, T3 → `unspecified-high`, T4 → `quick`, T5 → `deep`, T6 → `quick`
- **Wave 2**: **6 tasks** — T7 → `unspecified-high`, T8 → `unspecified-high`, T9 → `deep`, T10 → `deep`, T11 → `unspecified-high`, T12 → `unspecified-high`
- **Wave 3**: **4 tasks** — T13 → `deep`, T14 → `unspecified-high`, T15 → `unspecified-high`, T16 → `deep`
- **Wave 4**: **4 tasks** — T17 → `unspecified-high`, T18 → `visual-engineering`, T19 → `writing`, T20 → `quick`
- **FINAL**: **4 tasks** — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [x] 1. Python Service Scaffold + Config + Health Endpoint

  **What to do**:
  - Create `python/` directory structure: `python/app/`, `python/app/__init__.py`, `python/app/main.py`, `python/app/config.py`, `python/tests/`, `python/tests/unit/`, `python/tests/integration/`, `python/tests/conftest.py`
  - Create `pyproject.toml` with dependencies: fastapi, uvicorn, pydantic, pydantic-settings, sqlalchemy, alembic, qdrant-client, langfuse, openai, opentelemetry-api, opentelemetry-sdk, opentelemetry-instrumentation-fastapi, httpx, pytest, pytest-asyncio
  - Implement `config.py` using `pydantic-settings` with `BaseSettings`: database_url, qdrant_url, langfuse_public_key, langfuse_secret_key, langfuse_base_url, llm_api_base, llm_api_key, llm_model, otel_endpoint, env (dev/prod)
  - Implement `main.py` with FastAPI app factory, CORS middleware, `/healthz` endpoint returning `{"status": "ok"}`
  - Create `tests/unit/test_health.py` with failing test for health endpoint, then implement to pass
  - Create `tests/conftest.py` with pytest fixtures for test client, settings override

  **Must NOT do**:
  - Do NOT create any entity models or business logic yet
  - Do NOT add authentication middleware
  - Do NOT modify any Rust code

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Standard FastAPI scaffold with config, well-established patterns
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - `playwright`: No browser interaction needed

  **Parallelization**:
  - **Can Run In Parallel**: NO — foundation for all subsequent tasks
  - **Parallel Group**: Wave 1 (but sequential prerequisite)
  - **Blocks**: T2, T3, T4, T5, T6, all Wave 2+
  - **Blocked By**: None (can start immediately)

  **References**:

  **Pattern References**:
  - `src/config/mod.rs` — Existing Rust config pattern (env > TOML > defaults). Match this precedence philosophy in Python.
  - `src/main.rs` — Existing CLI command registration. Understand the existing module structure to avoid conflicts.
  - `Cargo.toml` — Existing dependency list. Understand what the Rust side already provides.

  **External References**:
  - FastAPI app factory pattern: https://fastapi.tiangolo.com/tutorial/bigger-applications/
  - pydantic-settings: https://docs.pydantic.dev/latest/concepts/pydantic_settings/

  **WHY Each Reference Matters**:
  - `src/config/mod.rs`: Match the configuration precedence pattern so both Rust and Python feel consistent
  - `Cargo.toml`: Know which capabilities already exist in Rust to avoid duplicating in Python

  **Acceptance Criteria**:

  **If TDD (tests enabled):**
  - [ ] Test file created: `python/tests/unit/test_health.py`
  - [ ] `pytest python/tests/unit/test_health.py -q` → PASS (1 test, 0 failures)

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Health endpoint returns 200 with status ok
    Tool: Bash (curl)
    Preconditions: FastAPI app running via uvicorn on port 8000
    Steps:
      1. Start app: `cd python && uvicorn app.main:app --port 8000 &`
      2. Wait 3 seconds for startup
      3. `curl -sf http://localhost:8000/healthz`
      4. Assert response contains `{"status":"ok"}`
      5. Assert HTTP status is 200
      6. Kill uvicorn process
    Expected Result: HTTP 200 with body `{"status":"ok"}`
    Failure Indicators: Connection refused, non-200 status, missing "ok" in response
    Evidence: .sisyphus/evidence/task-1-health-endpoint.txt

  Scenario: Config loads from environment variables
    Tool: Bash (pytest)
    Preconditions: .env file or env vars not set
    Steps:
      1. `cd python && DATABASE_URL=postgresql://test:test@localhost/test pytest tests/unit/test_health.py -q`
      2. Assert test passes and settings object contains overridden value
    Expected Result: Tests pass, config picks up env override
    Failure Indicators: Test failure, config not reading env vars
    Evidence: .sisyphus/evidence/task-1-config-env.txt
  ```

  **Commit**: YES
  - Message: `feat(python): scaffold FastAPI service with config and health endpoint`
  - Files: `python/app/__init__.py`, `python/app/main.py`, `python/app/config.py`, `python/pyproject.toml`, `python/tests/`
  - Pre-commit: `pytest python/tests/unit/test_health.py -q`

- [x] 2. Postgres Foundation (SQLAlchemy Base + Alembic + Session)

  **What to do**:
  - Create `python/app/db/__init__.py`, `python/app/db/session.py`, `python/app/db/base.py`
  - Implement `session.py`: SQLAlchemy `AsyncSession` factory, `get_db()` dependency for FastAPI, engine creation from settings
  - Implement `base.py`: Declarative base with common mixin (id, created_at, updated_at)
  - Set up Alembic: `python/alembic.ini`, `python/alembic/`, `python/alembic/env.py` configured for async
  - Write failing tests: `tests/unit/test_db.py` — test session creation, test base model has id/timestamps
  - Implement to pass tests

  **Must NOT do**:
  - Do NOT create entity models yet (that's T3)
  - Do NOT connect to real Postgres in unit tests (use mocks or SQLite in-memory)
  - Do NOT modify Rust DuckDB code

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Standard SQLAlchemy setup, well-documented patterns
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — depends on T1 for config
  - **Parallel Group**: Wave 1 (sequential after T1)
  - **Blocks**: T3
  - **Blocked By**: T1

  **References**:

  **Pattern References**:
  - `src/db/mod.rs` — Existing DuckDB schema (7 tables: sessions, runs, ingestion_log, workflow_runs, signals, collection_state, triage_runs). Understand the data model philosophy but DO NOT replicate in Postgres.

  **External References**:
  - SQLAlchemy 2.0 async: https://docs.sqlalchemy.org/en/20/orm/extensions/asyncio.html
  - Alembic async: https://alembic.sqlalchemy.org/en/latest/cookbook.html#using-asyncio-with-alembic

  **WHY Each Reference Matters**:
  - `src/db/mod.rs`: Understand the existing data model philosophy for consistency, but Postgres schema is entirely new for operational entities

  **Acceptance Criteria**:

  **If TDD:**
  - [ ] Test file created: `python/tests/unit/test_db.py`
  - [ ] `pytest python/tests/unit/test_db.py -q` → PASS

  **QA Scenarios:**

  ```
  Scenario: SQLAlchemy session factory creates valid session
    Tool: Bash (pytest)
    Preconditions: T1 complete, pytest available
    Steps:
      1. `cd python && pytest tests/unit/test_db.py -q`
      2. Assert test for session creation passes
      3. Assert test for base model fields passes
    Expected Result: All tests pass
    Evidence: .sisyphus/evidence/task-2-db-foundation.txt

  Scenario: Alembic migration environment configured
    Tool: Bash
    Preconditions: T1 complete, Alembic installed
    Steps:
      1. Verify `python/alembic.ini` exists
      2. Verify `python/alembic/env.py` exists and references async engine
      3. `cd python && python -c "from alembic.config import Config; c = Config('alembic.ini'); print(c.get_main_option('sqlalchemy.url'))"`
    Expected Result: Alembic config file exists and is importable
    Evidence: .sisyphus/evidence/task-2-alembic-config.txt
  ```

  **Commit**: YES
  - Message: `feat(python): add SQLAlchemy base, session management, and Alembic`
  - Files: `python/app/db/`, `python/alembic/`, `python/alembic.ini`
  - Pre-commit: `pytest python/tests/unit/test_db.py -q`

- [x] 3. Entity Models — All 10 Types + Relationships

  **What to do**:
  - Create `python/app/models/__init__.py` and individual model files
  - Define 10 entity models with SQLAlchemy ORM:
    - `Account` (id, name, tier, region, created_at)
    - `Site` (id, account_id FK, name, location, timezone, created_at)
    - `Fleet` (id, site_id FK, account_id FK, name, fleet_type, created_at)
    - `Device` (id, fleet_id FK, site_id FK, account_id FK, device_serial, device_type, software_revision_id FK, status, last_seen_at, created_at)
    - `Deployment` (id, fleet_id FK, software_revision_id FK, status, started_at, completed_at, created_at)
    - `Service` (id, name, version, status, created_at)
    - `SoftwareRevision` (id, version, release_notes, created_at, deployed_at)
    - `Incident` (id, account_id FK, severity, status, title, description, detected_at, resolved_at, created_at)
    - `Ticket` (id, account_id FK, site_id FK, device_id FK, incident_id FK (nullable), subject, body, priority, channel, status, created_at)
    - `OperationalEvent` (id, device_id FK, fleet_id FK, site_id FK, account_id FK, event_type, severity, description, metadata JSON, detected_at, created_at)
  - Define relationships: Account→Sites→Fleets→Devices, Deployment→Fleet, Incident→Account, Ticket→Incident (nullable), OperationalEvent→Device/Fleet/Site/Account
  - Generate Alembic migration
  - Write failing tests: model creation, relationship traversal, cascade behavior
  - Implement to pass

  **Must NOT do**:
  - Do NOT add API endpoints for entities yet
  - Do NOT add Pydantic schemas yet (that's T4)
  - Do NOT over-normalize — keep the model practical for the investigation flow

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: 10 interrelated entity models with foreign keys and relationships requires careful design
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — depends on T2
  - **Parallel Group**: Wave 1 (sequential after T2)
  - **Blocks**: T4, T5, T7, T9, T10
  - **Blocked By**: T2

  **References**:

  **Pattern References**:
  - `src/db/mod.rs` — Existing DuckDB tables (sessions, runs, signals, etc.). Understand the ID and timestamp conventions used (UUID ids, UTC timestamps).

  **External References**:
  - SQLAlchemy relationships: https://docs.sqlalchemy.org/en/20/orm/relationships.html
  - SQLAlchemy async session: https://docs.sqlalchemy.org/en/20/orm/extensions/asyncio.html

  **WHY Each Reference Matters**:
  - `src/db/mod.rs`: Match ID format (UUID) and timestamp conventions (UTC, chrono) for consistency across the repo

  **Acceptance Criteria**:

  **If TDD:**
  - [ ] Test file created: `python/tests/unit/test_models.py`
  - [ ] `pytest python/tests/unit/test_models.py -q` → PASS (all entity tests)

  **QA Scenarios:**

  ```
  Scenario: All 10 entity models can be instantiated with required fields
    Tool: Bash (pytest)
    Preconditions: T2 complete
    Steps:
      1. `cd python && pytest tests/unit/test_models.py -q -k "test_create"`
      2. Assert all 10 model creation tests pass
    Expected Result: 10/10 model creation tests pass
    Evidence: .sisyphus/evidence/task-3-model-creation.txt

  Scenario: Entity relationships traverse correctly
    Tool: Bash (pytest)
    Preconditions: Models defined with relationships
    Steps:
      1. `cd python && pytest tests/unit/test_models.py -q -k "test_relationships"`
      2. Assert Account→Sites→Fleets→Devices traversal works
      3. Assert Ticket→Incident nullable relationship works
      4. Assert OperationalEvent→Device/Fleet/Site/Account traversal works
    Expected Result: All relationship tests pass
    Evidence: .sisyphus/evidence/task-3-relationships.txt

  Scenario: Alembic migration generates and applies cleanly
    Tool: Bash
    Preconditions: Models defined, Alembic configured
    Steps:
      1. `cd python && alembic revision --autogenerate -m "add entity models"`
      2. Verify migration file created in `alembic/versions/`
      3. Verify migration contains CREATE TABLE for all 10 entities
    Expected Result: Migration file exists with all 10 CREATE TABLE statements
    Failure Indicators: Missing tables, relationship constraint errors
    Evidence: .sisyphus/evidence/task-3-migration.txt
  ```

  **Commit**: YES
  - Message: `feat(python): define all 10 entity models with relationships`
  - Files: `python/app/models/`, `python/alembic/versions/`
  - Pre-commit: `pytest python/tests/unit/test_models.py -q`

- [x] 4. Pydantic Schemas (Request/Response Models)

  **What to do**:
  - Create `python/app/schemas/__init__.py`, `python/app/schemas/entities.py`, `python/app/schemas/investigation.py`
  - Define Pydantic v2 models for:
    - Entity response schemas: `AccountOut`, `SiteOut`, `FleetOut`, `DeviceOut`, `DeploymentOut`, `ServiceOut`, `SoftwareRevisionOut`, `IncidentOut`, `TicketOut`, `OperationalEventOut`
    - Investigation request: `InvestigationRequest` with `signal_ids` dict (ticket_id, alert_id, event_id)
    - Investigation response: `InvestigationResponse` with `investigation_id`, `trace_id`, `entity_context`, `evidence`, `telemetry_analysis`, `historical_analysis`, `hypotheses`, `governance_decision`, `operator_briefing`, `customer_response_draft`
    - Shared schemas: `Hypothesis`, `EvidenceItem`, `GovernanceDecision`, `PolicyGateResult`
  - Write failing tests for schema validation, then implement
  - All schemas should have `model_config = ConfigDict(from_attributes=True)` for ORM compatibility

  **Must NOT do**:
  - Do NOT couple schemas to API endpoints yet
  - Do NOT add serialization logic beyond Pydantic's built-in

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Pydantic schemas are mechanical, well-defined from entity models
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T5, T6 — after T3)
  - **Parallel Group**: Wave 1
  - **Blocks**: T11, T15
  - **Blocked By**: T3

  **References**:

  **Pattern References**:
  - `python/app/models/` (from T3) — Entity model definitions. Schemas must match entity fields exactly.

  **External References**:
  - Pydantic v2 models: https://docs.pydantic.dev/latest/concepts/models/
  - Pydantic ConfigDict: https://docs.pydantic.dev/latest/api/config/

  **WHY Each Reference Matters**:
  - T3 entity models: Schemas are the API contract layer over the same entity shapes

  **Acceptance Criteria**:

  **If TDD:**
  - [ ] Test file created: `python/tests/unit/test_schemas.py`
  - [ ] `pytest python/tests/unit/test_schemas.py -q` → PASS

  **QA Scenarios:**

  ```
  Scenario: Investigation request schema validates correct input
    Tool: Bash (pytest)
    Preconditions: T3 complete
    Steps:
      1. `cd python && pytest tests/unit/test_schemas.py -q -k "test_investigation_request"`
      2. Assert valid signal_ids passes validation
      3. Assert missing required fields raises ValidationError
    Expected Result: Validation tests pass
    Evidence: .sisyphus/evidence/task-4-schemas.txt

  Scenario: Entity schemas serialize from ORM model instances
    Tool: Bash (pytest)
    Preconditions: T3 models exist
    Steps:
      1. `cd python && pytest tests/unit/test_schemas.py -q -k "test_entity_serialization"`
      2. Assert all 10 entity schemas can be created from mock ORM objects
    Expected Result: All serialization tests pass
    Evidence: .sisyphus/evidence/task-4-entity-schemas.txt
  ```

  **Commit**: YES
  - Message: `feat(python): add Pydantic request/response schemas`
  - Files: `python/app/schemas/`
  - Pre-commit: `pytest python/tests/unit/test_schemas.py -q`

- [x] 5. Synthetic Data Seeders — One Coherent Incident Narrative

  **What to do**:
  - Create `python/app/seed/__init__.py`, `python/app/seed/main.py`, `python/app/seed/entities.py`, `python/app/seed/evidence.py`
  - Design one coherent incident narrative:
    - **Account**: "Meridian Logistics" (tier: enterprise, region: us-west)
    - **Site**: "Portland Distribution Center" (location: Portland OR)
    - **Fleet**: "Warehouse Alpha Fleet" (fleet_type: autonomous_mobile_devices)
    - **Devices**: 8 devices, 3 affected (DEV-401, DEV-402, DEV-403), 5 healthy
    - **SoftwareRevision**: v3.2.1 (stable) → v3.3.0 (introduced regression)
    - **Deployment**: Fleet-wide rollout of v3.3.0 started 2 hours before incident
    - **OperationalEvent**: Device DEV-401 blocked (event_type: device_blocked, severity: high)
    - **Alert**: Anomaly alert ALT-2001 (metric: navigation_error_rate spike)
    - **Ticket**: TCK-1001 from Meridian site lead ("3 devices stopped navigating")
    - **Incident**: INC-5001 — the convergence target
    - **Supporting evidence**: 5 historical tickets for similar issues, 3 runbook docs, 2 telemetry snapshots, 1 deployment manifest
  - All IDs must be deterministic (not random UUIDs) for test reproducibility
  - Write failing tests: seeder creates correct entity counts, relationships are consistent, IDs are deterministic
  - Implement `seed/entities.py` to populate Postgres, `seed/evidence.py` to prepare data for Qdrant (actual indexing in T8)
  - Implement `seed/main.py` with `seed_all()` function that clears and reseeds

  **Must NOT do**:
  - Do NOT create random/generated data — one fixed narrative only
  - Do NOT connect to Qdrant yet (that's T8)
  - Do NOT create more than one incident scenario

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Requires designing a coherent, internally consistent operational narrative across 10 entity types with realistic relationships
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T4, T6 — after T3)
  - **Parallel Group**: Wave 1
  - **Blocks**: T7, T8, T9, T10
  - **Blocked By**: T3

  **References**:

  **Pattern References**:
  - `python/app/models/` (from T3) — Entity models and relationships. Seeder must create valid instances.
  - `src/db/mod.rs:insert_run()` — Existing idempotent upsert pattern. Seeder should be idempotent too.
  - `docs/deep-research-report.md` — Market context. Data should feel operationally real, matching the "governed operational intelligence" positioning.

  **WHY Each Reference Matters**:
  - T3 models: Seeder must respect all foreign key constraints and relationship cardinality
  - `docs/deep-research-report.md`: Data realism is critical for the artifact's credibility. The narrative should feel like a real operational incident.

  **Acceptance Criteria**:

  **If TDD:**
  - [ ] Test file created: `python/tests/unit/test_seeders.py`
  - [ ] `pytest python/tests/unit/test_seeders.py -q` → PASS

  **QA Scenarios:**

  ```
  Scenario: Seeder creates all entities with correct counts
    Tool: Bash (pytest)
    Preconditions: T3 complete, test database available (SQLite in-memory)
    Steps:
      1. `cd python && pytest tests/unit/test_seeders.py -q -k "test_entity_counts"`
      2. Assert exactly 1 Account, 1 Site, 1 Fleet, 8 Devices, etc.
    Expected Result: All entity count assertions pass
    Evidence: .sisyphus/evidence/task-5-seeder-counts.txt

  Scenario: Seeder is idempotent — running twice produces same data
    Tool: Bash (pytest)
    Preconditions: Seeder implemented
    Steps:
      1. `cd python && pytest tests/unit/test_seeders.py -q -k "test_idempotency"`
      2. Seed twice, assert entity counts unchanged
    Expected Result: Idempotency test passes
    Evidence: .sisyphus/evidence/task-5-seeder-idempotency.txt

  Scenario: Narrative is internally consistent
    Tool: Bash (pytest)
    Preconditions: All entities seeded
    Steps:
      1. `cd python && pytest tests/unit/test_seeders.py -q -k "test_narrative"`
      2. Assert DEV-401, DEV-402, DEV-403 belong to same fleet and site
      3. Assert affected devices have software_revision v3.3.0
      4. Assert deployment of v3.3.0 started before operational event
      5. Assert ticket references affected device DEV-401
      6. Assert alert references same fleet as affected devices
    Expected Result: All consistency assertions pass
    Evidence: .sisyphus/evidence/task-5-narrative-consistency.txt
  ```

  **Commit**: YES
  - Message: `feat(python): add synthetic data seeders with coherent incident narrative`
  - Files: `python/app/seed/`, `python/tests/unit/test_seeders.py`
  - Pre-commit: `pytest python/tests/unit/test_seeders.py -q`

- [x] 6. Dockerfile + Docker Compose Skeleton

  **What to do**:
  - Create `python/Dockerfile` — Python 3.12-slim, install deps, copy app, expose 8000, run uvicorn
  - Create `docker-compose.yml` at repo root with skeleton services:
    - `api` (FastAPI, build from python/)
    - `postgres` (postgres:17-alpine)
    - `qdrant` (qdrant/qdrant:latest)
  - Each service with health checks and basic environment variables
  - Create `.env.example` with placeholder values
  - Write test: `docker compose build api` succeeds
  - This is a SKELETON — full compose with Langfuse/Grafana is T17

  **Must NOT do**:
  - Do NOT add Langfuse, Grafana, ClickHouse, Redis, Prometheus yet (T17)
  - Do NOT configure production-grade settings
  - Do NOT expose ports to 0.0.0.0 — bind to 127.0.0.1

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Standard Dockerfile and minimal compose, well-documented
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T4, T5 — after T1)
  - **Parallel Group**: Wave 1
  - **Blocks**: T17
  - **Blocked By**: T1

  **References**:

  **Pattern References**:
  - `Cargo.toml` — Existing project name is "praxis". Docker service name should be "api" or "opsflow-api".
  - `.gitignore` — Check for existing Docker/infra patterns to preserve

  **External References**:
  - Docker Compose healthcheck: https://docs.docker.com/compose/compose-file/05-services/#healthcheck

  **WHY Each Reference Matters**:
  - `.gitignore`: Ensure Docker volumes, .env files are properly ignored

  **Acceptance Criteria**:

  **If TDD:**
  - [ ] No formal test file — verification via docker compose build

  **QA Scenarios:**

  ```
  Scenario: Docker image builds successfully
    Tool: Bash
    Preconditions: Dockerfile created, pyproject.toml exists
    Steps:
      1. `docker compose build api`
      2. Assert exit code 0
      3. Assert image tagged correctly
    Expected Result: Build succeeds without errors
    Failure Indicators: Build failures, missing dependencies
    Evidence: .sisyphus/evidence/task-6-docker-build.txt

  Scenario: Compose file is valid
    Tool: Bash
    Preconditions: docker-compose.yml exists
    Steps:
      1. `docker compose config`
      2. Assert exit code 0 (valid YAML and compose syntax)
    Expected Result: Valid compose configuration
    Evidence: .sisyphus/evidence/task-6-compose-validate.txt
  ```

  **Commit**: YES
  - Message: `infra: add Python Dockerfile and docker-compose skeleton`
  - Files: `python/Dockerfile`, `docker-compose.yml`, `.env.example`
  - Pre-commit: `docker compose build api`

- [x] 7. Qdrant Client + Collection Setup + Hybrid Retrieval

  **What to do**:
  - Create `python/app/retrieval/__init__.py`, `python/app/retrieval/client.py`, `python/app/retrieval/search.py`
  - Implement `client.py`: Qdrant client wrapper, collection management:
    - Collection: `operational_evidence` with dense vectors (384-dim, all-MiniLM-L6-v2) + sparse vectors (BM25)
    - Payload schema: entity_id, entity_type, source_type (ticket/doc/telemetry/runbook/log), content, timestamp, metadata
    - Payload indexes on entity_type, source_type, entity_id for filtering
  - Implement `search.py`: Hybrid retrieval functions:
    - `search_evidence(query, entity_ids=None, source_types=None, limit=10)` — RRF fusion of dense+sparse
    - `search_by_entity(entity_id, limit=5)` — Filter by entity, semantic search
    - `search_time_window(query, start_time, end_time, limit=10)` — Time-bounded search
  - Write failing tests: collection creation, hybrid search returns results, filtering works
  - Use deterministic fake vectors in tests (no model dependency for unit tests)

  **Must NOT do**:
  - Do NOT index actual data yet (that's T8)
  - Do NOT create multiple collections — one `operational_evidence` collection is sufficient
  - Do NOT implement graph retrieval (too complex for v1, hybrid is enough)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Qdrant hybrid search setup with dense+sparse vectors and RRF fusion requires careful implementation
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T8-T12 — after T3+T5)
  - **Parallel Group**: Wave 2
  - **Blocks**: T8, T13
  - **Blocked By**: T3

  **References**:

  **Pattern References**:
  - `python/app/models/` (T3) — Entity types inform the payload schema (entity_id, entity_type fields)
  - `python/app/seed/evidence.py` (T5) — Evidence data prepared for indexing, schema must match

  **External References**:
  - Qdrant hybrid search with RRF: https://qdrant.tech/documentation/concepts/hybrid-queries/
  - Qdrant sparse vectors + BM25: https://qdrant.tech/documentation/concepts/indexing/#sparse-vector-index
  - Qdrant payload filtering: https://qdrant.tech/documentation/concepts/filtering/

  **WHY Each Reference Matters**:
  - T5 evidence data: Retrieval payload schema must match the data the seeder prepares
  - Qdrant docs: RRF fusion and sparse vector setup require specific configuration

  **Acceptance Criteria**:

  **If TDD:**
  - [ ] Test file created: `python/tests/unit/test_retrieval.py`
  - [ ] `pytest python/tests/unit/test_retrieval.py -q` → PASS

  **QA Scenarios:**

  ```
  Scenario: Qdrant collection created with dense + sparse vectors
    Tool: Bash (pytest)
    Preconditions: Qdrant service running (or mocked in tests)
    Steps:
      1. `cd python && pytest tests/unit/test_retrieval.py -q -k "test_collection_creation"`
      2. Assert collection has dense vector config (384 dim, cosine)
      3. Assert collection has sparse vector config
    Expected Result: Collection configured for hybrid search
    Evidence: .sisyphus/evidence/task-7-collection-setup.txt

  Scenario: Hybrid search returns ranked results with RRF fusion
    Tool: Bash (pytest)
    Preconditions: Test points indexed with known content
    Steps:
      1. `cd python && pytest tests/unit/test_retrieval.py -q -k "test_hybrid_search"`
      2. Index 5 test documents with known content
      3. Search for "navigation error device blocked"
      4. Assert results are ranked and contain relevant documents
      5. Assert results have payload with entity_id, entity_type, source_type
    Expected Result: Search returns ranked, filtered results
    Evidence: .sisyphus/evidence/task-7-hybrid-search.txt

  Scenario: Entity filtering narrows results correctly
    Tool: Bash (pytest)
    Preconditions: Test points indexed for multiple entities
    Steps:
      1. Index documents for DEV-401 and DEV-402
      2. Search with entity_id filter for DEV-401 only
      3. Assert only DEV-401 documents returned
    Expected Result: Filtered results match entity_id
    Evidence: .sisyphus/evidence/task-7-entity-filter.txt
  ```

  **Commit**: YES
  - Message: `feat(python): add Qdrant client with hybrid retrieval`
  - Files: `python/app/retrieval/`, `python/tests/unit/test_retrieval.py`
  - Pre-commit: `pytest python/tests/unit/test_retrieval.py -q`

- [x] 8. Evidence Indexer — Seed Qdrant from Synthetic Data

  **What to do**:
  - Create `python/app/retrieval/indexer.py`
  - Implement `index_all_evidence(db_session, qdrant_client)`:
    - Index tickets: subject + body as content, entity_id=ticket_id, entity_type="ticket", source_type="ticket"
    - Index runbook docs: title + content, entity_type="runbook", source_type="doc"
    - Index telemetry snapshots: metric descriptions + annotations, entity_type="device", source_type="telemetry"
    - Index deployment manifests: version + release notes, entity_type="deployment", source_type="deployment"
    - Index historical tickets: past tickets for known issues, entity_type="ticket", source_type="historical_ticket"
  - Each document gets: dense embedding (real or fake depending on config), sparse BM25 vector, full payload
  - Use the narrative from T5 — ensure the evidence supports the multi-signal convergence story
  - Write integration tests: index all evidence, verify counts, search returns relevant results

  **Must NOT do**:
  - Do NOT generate embeddings from a real model in tests — use deterministic fake vectors
  - Do NOT index every entity — only evidence documents (tickets, docs, telemetry, deployment manifests)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Requires mapping seeder data to Qdrant payloads, ensuring consistency with the narrative
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — depends on T5 (data) and T7 (Qdrant client)
  - **Parallel Group**: Wave 2 (sequential after T5, T7)
  - **Blocks**: T13, T16
  - **Blocked By**: T5, T7

  **References**:

  **Pattern References**:
  - `python/app/seed/evidence.py` (T5) — Evidence data prepared for indexing. Indexer must consume this format.
  - `python/app/retrieval/client.py` (T7) — Qdrant collection schema. Indexer must produce matching points.

  **WHY Each Reference Matters**:
  - T5 evidence data: This is the source the indexer consumes
  - T7 Qdrant client: This is the target the indexer writes to

  **Acceptance Criteria**:

  **If TDD:**
  - [ ] Test file created: `python/tests/integration/test_indexing.py`
  - [ ] `pytest python/tests/integration/test_indexing.py -q` → PASS

  **QA Scenarios:**

  ```
  Scenario: All evidence documents indexed with correct counts
    Tool: Bash (pytest)
    Preconditions: T5 seeder data available, Qdrant client from T7
    Steps:
      1. Seed Postgres entities
      2. Run indexer
      3. Assert Qdrant point count matches expected: 5 historical tickets + 3 runbooks + 2 telemetry snapshots + 1 deployment manifest + 1 current ticket = 12 documents
    Expected Result: Correct document count in Qdrant
    Evidence: .sisyphus/evidence/task-8-index-counts.txt

  Scenario: Search for "navigation error" returns relevant historical tickets
    Tool: Bash (pytest)
    Preconditions: Evidence indexed
    Steps:
      1. Search for "navigation error device blocked"
      2. Assert results include historical ticket about similar issue
      3. Assert results include runbook about navigation troubleshooting
    Expected Result: Relevant evidence returned in top results
    Evidence: .sisyphus/evidence/task-8-search-relevance.txt
  ```

  **Commit**: YES
  - Message: `feat(python): add evidence indexer for Qdrant seeding`
  - Files: `python/app/retrieval/indexer.py`, `python/tests/integration/test_indexing.py`
  - Pre-commit: `pytest python/tests/integration/test_indexing.py -q`

- [x] 9. Telemetry Investigator Specialist Tool

  **What to do**:
  - Create `python/app/specialists/__init__.py`, `python/app/specialists/telemetry.py`
  - Implement `TelemetryInvestigator` as a callable tool:
    - `investigate(device_id, fleet_id, time_window) -> TelemetryReport`
    - Retrieves telemetry snapshots from Qdrant (filtered by device/fleet + time window)
    - Analyzes metric patterns (error rates, latency, throughput)
    - Extracts event timeline (what happened when)
    - Identifies anomalies relative to baseline
    - Returns structured `TelemetryReport` with findings, confidence, and evidence references
  - Uses LLM for interpretation (via T11 client) but falls back to rule-based analysis if LLM unavailable
  - Tool interface: async function that accepts typed inputs and returns typed outputs
  - Write failing tests: investigate with known telemetry returns expected analysis
  - The telemetry report for the narrative should identify: navigation_error_rate spike coinciding with v3.3.0 deployment

  **Must NOT do**:
  - Do NOT make this an autonomous agent — it's a tool with typed inputs/outputs
  - Do NOT connect to real monitoring systems
  - Do NOT implement real-time streaming telemetry

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Requires designing realistic telemetry analysis logic that feels production-quality
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T10, T11, T12 — after T3+T5)
  - **Parallel Group**: Wave 2
  - **Blocks**: T13
  - **Blocked By**: T3, T5, T11 (LLM client for interpretation)

  **References**:

  **Pattern References**:
  - `python/app/retrieval/search.py` (T7) — Evidence retrieval functions. Telemetry investigator uses these.
  - `python/app/seed/evidence.py` (T5) — Telemetry snapshot data. The investigator must analyze this specific data.

  **External References**:
  - Formant AI agent triage pattern (from deep-research-report.md): "triage alarms, investigate root causes, execute through existing tools, escalate with full triage packets"

  **WHY Each Reference Matters**:
  - T7 retrieval: The specialist retrieves telemetry evidence through the retrieval layer
  - T5 data: The analysis must match the specific narrative (navigation_error_rate spike after v3.3.0)

  **Acceptance Criteria**:

  **If TDD:**
  - [ ] Test file created: `python/tests/unit/test_telemetry_specialist.py`
  - [ ] `pytest python/tests/unit/test_telemetry_specialist.py -q` → PASS

  **QA Scenarios:**

  ```
  Scenario: Telemetry investigator identifies navigation error spike
    Tool: Bash (pytest)
    Preconditions: T5 seeded data, T7 retrieval available, LLM mocked
    Steps:
      1. Call investigate(device_id="DEV-401", fleet_id="FLT-101", time_window=(T-4h, T))
      2. Assert report identifies navigation_error_rate anomaly
      3. Assert report links anomaly to deployment time window
      4. Assert confidence score is set (0.0-1.0)
      5. Assert evidence references include telemetry snapshot IDs
    Expected Result: Structured TelemetryReport with navigation error findings
    Evidence: .sisyphus/evidence/task-9-telemetry-report.txt

  Scenario: Telemetry investigator returns graceful empty report for healthy device
    Tool: Bash (pytest)
    Preconditions: Data for healthy device DEV-404
    Steps:
      1. Call investigate(device_id="DEV-404", fleet_id="FLT-101", time_window=(T-4h, T))
      2. Assert report indicates no significant anomalies
      3. Assert confidence is appropriate for no-findings
    Expected Result: Clean no-findings report without errors
    Evidence: .sisyphus/evidence/task-9-telemetry-healthy.txt
  ```

  **Commit**: YES
  - Message: `feat(python): add telemetry investigator specialist tool`
  - Files: `python/app/specialists/telemetry.py`, `python/tests/unit/test_telemetry_specialist.py`
  - Pre-commit: `pytest python/tests/unit/test_telemetry_specialist.py -q`

- [x] 10. Historical Incident Investigator Specialist Tool

  **What to do**:
  - Create `python/app/specialists/historical.py`
  - Implement `HistoricalInvestigator` as a callable tool:
    - `investigate(entity_ids, entity_types, time_window) -> HistoricalReport`
    - Retrieves historical tickets from Qdrant filtered by entity context
    - Identifies recurring failure patterns across devices/fleets
    - Checks deployment adjacency (did issues correlate with software changes?)
    - Retrieves known issues / runbooks matching current symptoms
    - Builds account/fleet operational memory (what's happened before?)
    - Returns structured `HistoricalReport` with findings, patterns, confidence, evidence references
  - Uses LLM for pattern recognition (via T11 client) with fallback
  - The historical report for the narrative should identify: similar navigation issue 3 months ago (v3.1.2), runbook for navigation troubleshooting, deployment adjacency with v3.3.0 rollout

  **Must NOT do**:
  - Do NOT build a knowledge graph — relational queries + Qdrant search is sufficient
  - Do NOT implement cross-account pattern matching (same-account only in v1)
  - Do NOT make this an autonomous agent

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Requires designing pattern recognition logic that feels like real operational memory
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T9, T11, T12 — after T3+T5)
  - **Parallel Group**: Wave 2
  - **Blocks**: T13
  - **Blocked By**: T3, T5, T11

  **References**:

  **Pattern References**:
  - `python/app/retrieval/search.py` (T7) — Evidence retrieval. Historical investigator uses search_by_entity and time-window search.
  - `python/app/seed/evidence.py` (T5) — Historical tickets and runbooks. The investigator must surface these.

  **External References**:
  - Pylon account intelligence pattern (from deep-research-report.md): "account-level intelligence, routing, knowledge automation"

  **WHY Each Reference Matters**:
  - T7 retrieval: Historical evidence retrieved through the same retrieval layer
  - T5 data: Historical data includes 5 past tickets and 3 runbooks that must be discoverable

  **Acceptance Criteria**:

  **If TDD:**
  - [ ] Test file created: `python/tests/unit/test_historical_specialist.py`
  - [ ] `pytest python/tests/unit/test_historical_specialist.py -q` → PASS

  **QA Scenarios:**

  ```
  Scenario: Historical investigator finds past navigation issue
    Tool: Bash (pytest)
    Preconditions: T5 seeded data, LLM mocked
    Steps:
      1. Call investigate(entity_ids=["DEV-401"], entity_types=["device"])
      2. Assert report identifies historical ticket about navigation issue (v3.1.2)
      3. Assert report identifies runbook for navigation troubleshooting
      4. Assert report identifies deployment adjacency (v3.3.0 rollout timing)
      5. Assert confidence score is reasonable
    Expected Result: HistoricalReport with pattern findings
    Evidence: .sisyphus/evidence/task-10-historical-report.txt

  Scenario: Historical investigator handles entity with no history
    Tool: Bash (pytest)
    Preconditions: Entity with no past tickets
    Steps:
      1. Call investigate(entity_ids=["DEV-900"], entity_types=["device"])
      2. Assert report returns empty/minimal findings
      3. Assert no errors or crashes
    Expected Result: Clean minimal report
    Evidence: .sisyphus/evidence/task-10-historical-empty.txt
  ```

  **Commit**: YES
  - Message: `feat(python): add historical incident investigator specialist tool`
  - Files: `python/app/specialists/historical.py`, `python/tests/unit/test_historical_specialist.py`
  - Pre-commit: `pytest python/tests/unit/test_historical_specialist.py -q`

- [x] 11. LLM Client + Prompt Templates (OpenAI-Compatible)

  **What to do**:
  - Create `python/app/llm/__init__.py`, `python/app/llm/client.py`, `python/app/llm/prompts.py`
  - Implement `client.py`: Async OpenAI-compatible client wrapper:
    - `generate(system_prompt, user_prompt, **kwargs) -> LLMResponse`
    - `generate_structured(system_prompt, user_prompt, response_schema, **kwargs) -> dict`
    - Handles streaming internally but returns complete responses
    - Token counting, cost estimation (optional)
    - Configurable model, temperature, max_tokens via settings
    - Proper error handling with retries for transient failures
  - Implement `prompts.py`: Named prompt templates:
    - `TELEMETRY_ANALYSIS` — System prompt for telemetry interpretation
    - `HISTORICAL_PATTERN_RECOGNITION` — System prompt for pattern matching
    - `HYPOTHESIS_GENERATION` — System prompt for generating ranked hypotheses from evidence
    - `GOVERNANCE_CLASSIFICATION` — System prompt for action classification
    - `OPERATOR_BRIEFING` — System prompt for internal briefing generation
    - `CUSTOMER_RESPONSE` — System prompt for customer-safe response drafting
  - Write failing tests: client calls with mocked responses, prompt templates are valid

  **Must NOT do**:
  - Do NOT implement streaming to callers — internal only
  - Do NOT hardcode model names — use config
  - Do NOT implement prompt versioning/management systems

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: LLM client with structured output, retry logic, and multiple specialized prompt templates
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T9, T10, T12)
  - **Parallel Group**: Wave 2
  - **Blocks**: T9, T10, T12, T13
  - **Blocked By**: T1, T4

  **References**:

  **Pattern References**:
  - `src/llm/mod.rs` — Existing Rust LLM client with streaming, SSE parsing, reasoning fields. Match the general approach but Python implementation can be simpler.
  - `src/signals/triage.rs` — Existing triage prompt structure. Match the pattern of system prompt + user prompt with structured output.

  **External References**:
  - OpenAI Python SDK: https://github.com/openai/openai-python
  - Structured outputs: https://platform.openai.com/docs/guides/structured-outputs

  **WHY Each Reference Matters**:
  - `src/llm/mod.rs`: Match the OpenAI-compatible API approach for consistency
  - `src/signals/triage.rs`: The existing triage prompts show the pattern of structured LLM output

  **Acceptance Criteria**:

  **If TDD:**
  - [ ] Test file created: `python/tests/unit/test_llm_client.py`
  - [ ] `pytest python/tests/unit/test_llm_client.py -q` → PASS

  **QA Scenarios:**

  ```
  Scenario: LLM client generates response with mocked API
    Tool: Bash (pytest)
    Preconditions: OpenAI client mocked in tests
    Steps:
      1. `cd python && pytest tests/unit/test_llm_client.py -q -k "test_generate"`
      2. Assert mocked response returned correctly
      3. Assert token counts extracted
    Expected Result: Client returns LLMResponse with text and metadata
    Evidence: .sisyphus/evidence/task-11-llm-client.txt

  Scenario: All prompt templates are valid and render correctly
    Tool: Bash (pytest)
    Preconditions: prompts.py created
    Steps:
      1. `cd python && pytest tests/unit/test_llm_client.py -q -k "test_prompts"`
      2. Assert each template has system and user components
      3. Assert templates can be formatted with sample data without errors
    Expected Result: All 6 prompt templates render without errors
    Evidence: .sisyphus/evidence/task-11-prompt-templates.txt
  ```

  **Commit**: YES
  - Message: `feat(python): add LLM client with prompt templates`
  - Files: `python/app/llm/`, `python/tests/unit/test_llm_client.py`
  - Pre-commit: `pytest python/tests/unit/test_llm_client.py -q`

- [x] 12. Policy/Governance Engine — Classification + Gating

  **What to do**:
  - Create `python/app/governance/__init__.py`, `python/app/governance/engine.py`, `python/app/governance/classification.py`
  - Implement `classification.py`: Action classifier:
    - `classify_action(hypothesis, evidence, entity_context) -> ActionClassification`
    - Categories: `INVESTIGATE`, `RECOMMEND`, `ESCALATE`, `COMMUNICATE`, `EXECUTE` (EXECUTE always blocked in v1)
    - Severity levels: `LOW`, `MEDIUM`, `HIGH`, `CRITICAL`
    - Customer sensitivity: `INTERNAL_ONLY`, `CUSTOMER_FACING`, `VIP_CUSTOMER`
  - Implement `engine.py`: Governance engine:
    - `evaluate(hypothesis, evidence, entity_context, action_classification) -> GovernanceDecision`
    - HITL gating: HIGH/CRITICAL + CUSTOMER_FACING requires explicit escalation flag
    - Bounded tool execution: specialists can investigate but cannot take actions
    - Severity-aware behavior: CRITICAL triggers additional validation
    - Safe output boundary: customer-facing output must pass sensitivity filter
    - Returns: `GovernanceDecision` with approved_actions, blocked_actions, escalation_required, confidence_threshold, reasoning
  - Write failing tests: classification accuracy, gating behavior, edge cases

  **Must NOT do**:
  - Do NOT build a full policy DSL or rules engine
  - Do NOT implement real action execution
  - Do NOT add audit trail persistence (governance decisions are part of the investigation trace in Langfuse)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Requires careful design of classification logic and policy boundaries that feel production-realistic
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T9, T10, T11)
  - **Parallel Group**: Wave 2
  - **Blocks**: T13
  - **Blocked By**: T4, T11

  **References**:

  **Pattern References**:
  - `src/signals/triage.rs` — Existing signal triage with 5 priority buckets (ACT_NOW, REVIEW_TODAY, REVIEW_WEEK, BACKGROUND, IGNORE). The governance classification should be similarly structured.
  - `docs/deep-research-report.md` — "progressive, not absolute" autonomy model. Governance should visibly distinguish investigation/recommendation/escalation/communication/execution.

  **External References**:
  - OpenAI guardrails: https://platform.openai.com/docs/guides/guardrails
  - Anthropic guardrails: https://docs.anthropic.com/en/docs/build-with-claude/guardrails

  **WHY Each Reference Matters**:
  - Triage system: The governance layer extends the existing priority classification into action-level policy
  - Research report: The "progressive autonomy" model is the core governance philosophy

  **Acceptance Criteria**:

  **If TDD:**
  - [ ] Test file created: `python/tests/unit/test_governance.py`
  - [ ] `pytest python/tests/unit/test_governance.py -q` → PASS

  **QA Scenarios:**

  ```
  Scenario: Governance classifies high-severity customer-facing issue correctly
    Tool: Bash (pytest)
    Preconditions: T11 complete, schemas defined
    Steps:
      1. Create mock hypothesis with HIGH severity and VIP_CUSTOMER sensitivity
      2. Call classify_action() → assert ESCALATE classification
      3. Call evaluate() → assert escalation_required=True
      4. Assert EXECUTE is always in blocked_actions
    Expected Result: Governance decision requires escalation
    Evidence: .sisyphus/evidence/task-12-governance-high.txt

  Scenario: Governance allows investigation for low-severity internal issue
    Tool: Bash (pytest)
    Preconditions: Mock hypothesis with LOW severity, INTERNAL_ONLY
    Steps:
      1. Call classify_action() → assert INVESTIGATE
      2. Call evaluate() → assert escalation_required=False
      3. Assert INVESTIGATE in approved_actions
    Expected Result: Governance allows investigation without escalation
    Evidence: .sisyphus/evidence/task-12-governance-low.txt

  Scenario: EXECUTE action always blocked in v1
    Tool: Bash (pytest)
    Preconditions: Any hypothesis
    Steps:
      1. Call evaluate() with any classification
      2. Assert EXECUTE is always in blocked_actions regardless of severity
    Expected Result: EXECUTE never approved
    Evidence: .sisyphus/evidence/task-12-governance-execute-blocked.txt
  ```

  **Commit**: YES
  - Message: `feat(python): add policy/governance engine`
  - Files: `python/app/governance/`, `python/tests/unit/test_governance.py`
  - Pre-commit: `pytest python/tests/unit/test_governance.py -q`

- [x] 13. Control-Plane Investigation Orchestrator

  **What to do**:
  - Create `python/app/orchestrator/__init__.py`, `python/app/orchestrator/investigation.py`, `python/app/orchestrator/phases.py`
  - Implement `phases.py`: Investigation phase definitions (enum + descriptions):
    1. **Signal Ingestion**: Receive and normalize signals (ticket, alert, event)
    2. **Entity Resolution**: Resolve signals to entities (account, site, fleet, device, deployment)
    3. **Evidence Retrieval**: Gather evidence from Qdrant across all signal contexts
    4. **Specialist Investigation**: Invoke telemetry + historical specialists
    5. **Hypothesis Generation**: LLM generates ranked hypotheses from all evidence
    6. **Governance Evaluation**: Policy engine classifies and gates actions
    7. **Output Generation**: Produce operator briefing + customer-safe response
  - Implement `investigation.py`: `InvestigationManager`:
    - `run_investigation(signal_ids) -> InvestigationResponse`
    - Coordinates all 7 phases in sequence
    - Each phase emits a Langfuse span (via T14 tracing)
    - Collects evidence from retrieval layer
    - Invokes specialists with appropriate context
    - Feeds specialist outputs to hypothesis generator
    - Passes hypothesis through governance
    - Generates final outputs
    - Returns complete `InvestigationResponse` with trace_id
  - Write failing tests: mock all dependencies, verify phase execution order, verify output shape

  **Must NOT do**:
  - Do NOT implement parallel specialist invocation (sequential is fine for v1)
  - Do NOT add workflow engine or planner/executor framework
  - Do NOT make the orchestrator an autonomous agent — it follows a fixed phase progression

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Central orchestration logic coordinating 7 phases across 4 subsystems (retrieval, specialists, governance, LLM)
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — depends on all Wave 2 tasks
  - **Parallel Group**: Wave 3 (sequential after Wave 2)
  - **Blocks**: T14, T15, T16
  - **Blocked By**: T7, T8, T9, T10, T11, T12

  **References**:

  **Pattern References**:
  - `python/app/retrieval/search.py` (T7) — Evidence retrieval functions for phase 3
  - `python/app/specialists/telemetry.py` (T9) — Telemetry specialist for phase 4
  - `python/app/specialists/historical.py` (T10) — Historical specialist for phase 4
  - `python/app/llm/prompts.py` (T11) — HYPOTHESIS_GENERATION, OPERATOR_BRIEFING, CUSTOMER_RESPONSE prompts
  - `python/app/governance/engine.py` (T12) — Governance evaluation for phase 6

  **WHY Each Reference Matters**:
  - T7-T12: The orchestrator is the glue that calls all these subsystems in the correct order

  **Acceptance Criteria**:

  **If TDD:**
  - [ ] Test file created: `python/tests/unit/test_orchestrator.py`
  - [ ] `pytest python/tests/unit/test_orchestrator.py -q` → PASS

  **QA Scenarios:**

  ```
  Scenario: Orchestrator executes all 7 phases in order
    Tool: Bash (pytest)
    Preconditions: All dependencies mocked
    Steps:
      1. Call run_investigation with signal_ids={ticket_id: "TCK-1001", alert_id: "ALT-2001", event_id: "EVT-3001"}
      2. Assert all 7 phases executed via mock call verification
      3. Assert phase execution order: ingestion → resolution → retrieval → specialists → hypothesis → governance → output
    Expected Result: All phases called in correct sequence
    Evidence: .sisyphus/evidence/task-13-phase-order.txt

  Scenario: Orchestrator returns complete InvestigationResponse
    Tool: Bash (pytest)
    Preconditions: All dependencies mocked with realistic returns
    Steps:
      1. Call run_investigation with synthetic signal IDs
      2. Assert response has: investigation_id, entity_context, evidence list, telemetry_analysis, historical_analysis, hypotheses (ranked), governance_decision, operator_briefing, customer_response_draft
      3. Assert hypotheses are ranked by confidence
    Expected Result: Complete InvestigationResponse with all fields populated
    Evidence: .sisyphus/evidence/task-13-complete-response.txt

  Scenario: Orchestrator handles partial evidence gracefully
    Tool: Bash (pytest)
    Preconditions: Retrieval returns limited evidence
    Steps:
      1. Mock retrieval to return only 1 document
      2. Call run_investigation
      3. Assert response still completes all phases
      4. Assert hypothesis reflects limited evidence (lower confidence)
    Expected Result: Investigation completes with low-confidence output
    Evidence: .sisyphus/evidence/task-13-partial-evidence.txt
  ```

  **Commit**: YES
  - Message: `feat(python): add control-plane investigation orchestrator`
  - Files: `python/app/orchestrator/`, `python/tests/unit/test_orchestrator.py`
  - Pre-commit: `pytest python/tests/unit/test_orchestrator.py -q`

- [x] 14. Langfuse Trace Emission Wiring

  **What to do**:
  - Create `python/app/tracing/__init__.py`, `python/app/tracing/langfuse.py`, `python/app/tracing/spans.py`
  - Implement `langfuse.py`: Langfuse client wrapper:
    - Initialize with settings (public_key, secret_key, base_url)
    - Helper to create root trace for investigation
    - Helper to create nested spans for each phase
    - Helper to create generation spans for LLM calls
    - Helper to create tool spans for specialist invocations
    - Helper to attach evidence metadata to spans
  - Implement `spans.py`: Named span definitions matching the 7 investigation phases:
    - `signal_ingestion`, `entity_resolution`, `evidence_retrieval`, `specialist_investigation`, `hypothesis_generation`, `governance_evaluation`, `output_generation`
    - Plus sub-spans: `telemetry_specialist`, `historical_specialist`, `retrieval_hybrid`, `retrieval_entity`
  - Wire tracing into orchestrator — each phase creates a span, specialists create sub-spans, LLM calls create generation spans
  - Mock Langfuse in unit tests, verify trace structure in integration tests
  - Investigation response must include `trace_id` field

  **Must NOT do**:
  - Do NOT set up OpenTelemetry export to Langfuse in v1 — use Langfuse SDK directly (simpler)
  - Do NOT persist traces to our own database — Langfuse is the store
  - Do NOT add custom Langfuse UI or dashboards

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Requires careful instrumentation of every investigation phase with proper span nesting and metadata
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — depends on T13 orchestrator
  - **Parallel Group**: Wave 3 (sequential after T13)
  - **Blocks**: T15, T16
  - **Blocked By**: T13

  **References**:

  **External References**:
  - Langfuse Python SDK v4: https://langfuse.com/docs/sdk/python/decorators
  - Langfuse trace structure: https://langfuse.com/docs/tracing/sessions

  **WHY Each Reference Matters**:
  - Langfuse docs: Must follow the correct API for nested spans with metadata

  **Acceptance Criteria**:

  **If TDD:**
  - [ ] Test file created: `python/tests/unit/test_tracing.py`
  - [ ] `pytest python/tests/unit/test_tracing.py -q` → PASS

  **QA Scenarios:**

  ```
  Scenario: Investigation creates root trace with nested phase spans
    Tool: Bash (pytest)
    Preconditions: Langfuse mocked
    Steps:
      1. Run investigation (mocked dependencies)
      2. Assert root trace created with investigation_id
      3. Assert 7 phase spans created under root trace
      4. Assert specialist sub-spans created under specialist_investigation span
    Expected Result: Proper trace hierarchy created
    Evidence: .sisyphus/evidence/task-14-trace-structure.txt

  Scenario: LLM calls emit generation spans with token usage
    Tool: Bash (pytest)
    Preconditions: Langfuse mocked, LLM client mocked
    Steps:
      1. Run investigation
      2. Assert each LLM call creates a generation span
      3. Assert generation spans include input, output, usage_details
    Expected Result: Generation spans with metadata
    Evidence: .sisyphus/evidence/task-14-generation-spans.txt

  Scenario: Investigation response includes trace_id
    Tool: Bash (pytest)
    Preconditions: Full orchestrator with tracing wired
    Steps:
      1. Run investigation
      2. Assert response.trace_id is non-empty string
      3. Assert trace_id corresponds to Langfuse trace
    Expected Result: trace_id present in response
    Evidence: .sisyphus/evidence/task-14-trace-id.txt
  ```

  **Commit**: YES
  - Message: `feat(python): add Langfuse trace emission wiring`
  - Files: `python/app/tracing/`, `python/tests/unit/test_tracing.py`
  - Pre-commit: `pytest python/tests/unit/test_tracing.py -q`

- [x] 15. Investigation API Endpoint (POST /investigations)

  **What to do**:
  - Create `python/app/api/__init__.py`, `python/app/api/router.py`, `python/app/api/investigations.py`
  - Implement router with FastAPI APIRouter:
    - `POST /api/v1/investigations` — Accept `InvestigationRequest` (signal_ids), return `InvestigationResponse`
    - `GET /api/v1/investigations/{investigation_id}` — Retrieve stored investigation
    - `POST /api/v1/seed` — Trigger data seeding (clear and reseed)
    - `GET /healthz` — Health check (already exists from T1, move to api module)
  - Request validation: signal_ids must contain at least one of ticket_id, alert_id, event_id
  - Response includes: investigation_id, trace_id, all phase outputs
  - Wire FastAPI dependency injection for DB session, Qdrant client, orchestrator
  - Write integration tests: POST returns 200 with complete response, validation rejects bad input, seed endpoint works

  **Must NOT do**:
  - Do NOT add authentication
  - Do NOT implement streaming responses
  - Do NOT add pagination or listing endpoints

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: API endpoint with validation, dependency injection, and integration with orchestrator
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — depends on T13, T14
  - **Parallel Group**: Wave 3 (sequential after T13, T14)
  - **Blocks**: T16
  - **Blocked By**: T13, T14

  **References**:

  **Pattern References**:
  - `python/app/schemas/investigation.py` (T4) — Request/response schemas. API must use these exactly.
  - `python/app/orchestrator/investigation.py` (T13) — InvestigationManager. API calls this.

  **External References**:
  - FastAPI dependency injection: https://fastapi.tiangolo.com/tutorial/dependencies/

  **WHY Each Reference Matters**:
  - T4 schemas: API contract defined by schemas
  - T13 orchestrator: API delegates to orchestrator

  **Acceptance Criteria**:

  **If TDD:**
  - [ ] Test file created: `python/tests/integration/test_investigation_api.py`
  - [ ] `pytest python/tests/integration/test_investigation_api.py -q` → PASS

  **QA Scenarios:**

  ```
  Scenario: POST /investigations returns complete investigation
    Tool: Bash (curl)
    Preconditions: All services running, data seeded
    Steps:
      1. `curl -sf -X POST http://localhost:8000/api/v1/investigations -H 'Content-Type: application/json' -d '{"signal_ids": {"ticket_id": "TCK-1001", "alert_id": "ALT-2001", "event_id": "EVT-3001"}}'`
      2. Assert HTTP 200
      3. Assert response has investigation_id, trace_id
      4. Assert entity_context contains Account, Site, Fleet, Device references
      5. Assert hypotheses array is non-empty with ranked entries
      6. Assert governance_decision is present with approved/blocked actions
      7. Assert operator_briefing is non-empty string
      8. Assert customer_response_draft is non-empty string
    Expected Result: HTTP 200 with complete investigation response
    Failure Indicators: Missing fields, empty arrays, HTTP 4xx/5xx
    Evidence: .sisyphus/evidence/task-15-api-investigation.txt

  Scenario: POST /investigations rejects empty signal_ids
    Tool: Bash (curl)
    Preconditions: API running
    Steps:
      1. `curl -sf -X POST http://localhost:8000/api/v1/investigations -H 'Content-Type: application/json' -d '{"signal_ids": {}}'`
    Expected Result: HTTP 422 with validation error
    Evidence: .sisyphus/evidence/task-15-api-validation.txt

  Scenario: POST /seed creates deterministic data
    Tool: Bash (curl)
    Preconditions: API running
    Steps:
      1. `curl -sf -X POST http://localhost:8000/api/v1/seed`
      2. Assert response contains entity counts matching narrative
    Expected Result: Deterministic counts for all entity types
    Evidence: .sisyphus/evidence/task-15-api-seed.txt
  ```

  **Commit**: YES
  - Message: `feat(python): add investigation API endpoint`
  - Files: `python/app/api/`, `python/tests/integration/test_investigation_api.py`
  - Pre-commit: `pytest python/tests/integration/test_investigation_api.py -q`

- [x] 16. Seed Command + End-to-End Investigation Test

  **What to do**:
  - Create `python/tests/e2e/test_full_investigation.py`
  - Implement end-to-end test that:
    1. Seeds the database with synthetic data
    2. Indexes evidence into Qdrant
    3. Triggers investigation via orchestrator (not API — test the logic directly)
    4. Verifies complete output: entity resolution matches narrative, telemetry identifies navigation error, historical finds past ticket, hypothesis links v3.3.0 deployment, governance requires escalation (enterprise customer + HIGH severity), operator briefing is substantive, customer response is appropriate
  - This is the golden-path test — it validates the entire system works together
  - All external dependencies (LLM, Qdrant) should be mockable but test with real Qdrant if available
  - Add CLI seed command: `python -m app.seed` that runs the seeder outside of API context

  **Must NOT do**:
  - Do NOT make this test require Docker Compose — it should work with mocked services
  - Do NOT test edge cases here — those belong in specialist/governance unit tests

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Integration test validating the entire investigation pipeline end-to-end requires understanding all subsystems
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — depends on all Wave 3 tasks
  - **Parallel Group**: Wave 3 (sequential after T14, T15)
  - **Blocks**: T17, T19
  - **Blocked By**: T14, T15

  **References**:

  **Pattern References**:
  - `python/app/orchestrator/investigation.py` (T13) — The main system under test
  - `python/app/seed/main.py` (T5) — Seeder that provides test data
  - `python/app/retrieval/indexer.py` (T8) — Evidence indexer for test data

  **WHY Each Reference Matters**:
  - T13 orchestrator: This is the integration point being tested
  - T5 seeder: Provides the coherent narrative that the test validates against

  **Acceptance Criteria**:

  **If TDD:**
  - [ ] Test file created: `python/tests/e2e/test_full_investigation.py`
  - [ ] `pytest python/tests/e2e/test_full_investigation.py -q` → PASS

  **QA Scenarios:**

  ```
  Scenario: Golden-path investigation resolves multi-signal convergence
    Tool: Bash (pytest)
    Preconditions: All modules implemented, data seeded
    Steps:
      1. Seed data: entities + evidence indexed
      2. Call run_investigation(signal_ids={TCK-1001, ALT-2001, EVT-3001})
      3. Assert entity resolution links all 3 signals to Account "Meridian Logistics"
      4. Assert telemetry report identifies navigation_error_rate anomaly
      5. Assert historical report finds past v3.1.2 navigation issue
      6. Assert top hypothesis mentions v3.3.0 deployment as likely cause
      7. Assert governance requires escalation (enterprise account, HIGH severity)
      8. Assert operator briefing mentions v3.3.0, DEV-401/402/403, navigation errors
      9. Assert customer response is professional and does not mention internal tooling
      10. Assert trace_id is present
    Expected Result: Complete investigation matching the Meridian Logistics narrative
    Evidence: .sisyphus/evidence/task-16-golden-path.txt

  Scenario: CLI seed command works outside API context
    Tool: Bash
    Preconditions: Python environment set up
    Steps:
      1. `cd python && python -m app.seed --db-url sqlite:///test.db`
      2. Assert exit code 0
      3. Assert test.db contains expected entity counts
    Expected Result: Seed command completes successfully
    Evidence: .sisyphus/evidence/task-16-cli-seed.txt
  ```

  **Commit**: YES
  - Message: `feat(python): add seed command and end-to-end investigation test`
  - Files: `python/tests/e2e/`, `python/app/seed/__main__.py`
  - Pre-commit: `pytest python/tests/e2e/ -q`

- [x] 17. Full Docker Compose with All Services

  **What to do**:
  - Expand `docker-compose.yml` from skeleton (T6) to full stack:
    - `api` (FastAPI) — build from `python/`, depends on postgres, qdrant
    - `postgres` (postgres:17-alpine) — with health check, volume, env vars
    - `qdrant` (qdrant/qdrant:latest) — with health check, volume
    - `langfuse-web` (langfuse:3) — web UI, depends on postgres, clickhouse, redis
    - `langfuse-worker` (langfuse-worker:3) — background processor
    - `clickhouse` (clickhouse-server) — Langfuse dependency
    - `redis` (redis:7-alpine) — Langfuse dependency
    - `prometheus` (prom/prometheus) — metrics collection
    - `grafana` (grafana/grafana) — dashboards, depends on prometheus
  - Add proper health checks for all services
  - Add startup dependency ordering (depends_on with condition)
  - Configure networking (opsflow-network)
  - Bind all ports to 127.0.0.1
  - Create `infra/prometheus/prometheus.yml` with scrape targets
  - Test: `docker compose up -d` brings all services healthy

  **Must NOT do**:
  - Do NOT configure production-grade security (this is a demo artifact)
  - Do NOT add CI/CD pipeline
  - Do NOT optimize for resource usage beyond "works on a developer laptop"

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: 9-service Docker Compose with health checks, dependency ordering, and cross-service networking
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T18, T19, T20 — after T16)
  - **Parallel Group**: Wave 4
  - **Blocks**: T18, T20
  - **Blocked By**: T6, T16

  **References**:

  **Pattern References**:
  - `docker-compose.yml` (T6) — Existing skeleton to expand
  - `python/Dockerfile` (T6) — API service Dockerfile

  **External References**:
  - Langfuse self-hosting: https://langfuse.com/self-hosting
  - Docker Compose healthcheck: https://docs.docker.com/compose/compose-file/05-services/#healthcheck

  **WHY Each Reference Matters**:
  - T6 skeleton: This is the base to expand, not rewrite
  - Langfuse docs: Langfuse requires ClickHouse + Redis + Postgres — must get the env vars right

  **Acceptance Criteria**:

  **QA Scenarios:**

  ```
  Scenario: All 9 services become healthy
    Tool: Bash
    Preconditions: Docker available, images pulled
    Steps:
      1. `docker compose up -d`
      2. Wait 60 seconds for all health checks
      3. `docker compose ps`
      4. Assert all services show "healthy" or "running" status
    Expected Result: All 9 services running and healthy
    Failure Indicators: Service exits, health check fails, dependency error
    Evidence: .sisyphus/evidence/task-17-compose-up.txt

  Scenario: API service connects to Postgres and Qdrant
    Tool: Bash (curl)
    Preconditions: All services up
    Steps:
      1. `curl -sf http://localhost:8000/healthz` → 200
      2. `curl -sf http://localhost:8000/api/v1/seed` → 200
      3. `curl -sf -X POST http://localhost:8000/api/v1/investigations -H 'Content-Type: application/json' -d '{"signal_ids":{"ticket_id":"TCK-1001","alert_id":"ALT-2001","event_id":"EVT-3001"}}'` → 200 with investigation
    Expected Result: Full investigation flow works in Docker
    Evidence: .sisyphus/evidence/task-17-docker-investigation.txt

  Scenario: Langfuse UI accessible and receiving traces
    Tool: Bash (curl)
    Preconditions: All services up, investigation triggered
    Steps:
      1. `curl -sf http://localhost:3000/api/health` → 200
      2. Trigger an investigation via API
      3. Check Langfuse API for traces: `curl -sf http://localhost:3000/api/public/traces -H "Authorization: Bearer $LANGFUSE_PUBLIC_KEY"`
      4. Assert at least one trace exists
    Expected Result: Langfuse receives and stores investigation traces
    Evidence: .sisyphus/evidence/task-17-langfuse-traces.txt
  ```

  **Commit**: YES
  - Message: `infra: complete docker-compose with all services and health checks`
  - Files: `docker-compose.yml`, `infra/prometheus/prometheus.yml`
  - Pre-commit: `docker compose config`

- [x] 18. Grafana Dashboards + Provisioning

  **What to do**:
  - Create Grafana provisioning files:
    - `infra/grafana/provisioning/datasources/datasources.yml` — Prometheus datasource
    - `infra/grafana/provisioning/dashboards/dashboards.yml` — Dashboard provider
  - Create operational dashboards:
    - `infra/grafana/dashboards/opsflow-overview.json` — Main operational dashboard with panels:
      - Investigation count (counter)
      - Investigation latency (histogram/average)
      - Policy gate triggers by action type (pie/bar chart)
      - Anomaly categories distribution (bar chart)
      - Tool usage by specialist type (bar chart)
      - Retrieval hit/miss rate (gauge)
      - Escalation decisions (counter by severity)
      - Simulated fleet health status (stat panel)
  - Configure Prometheus to scrape:
    - FastAPI metrics endpoint (add `prometheus_fastapi_instrumentator` to API)
    - Qdrant metrics (`/metrics` endpoint)
  - Dashboard should feel like a real operational NOC view, not a demo toy

  **Must NOT do**:
  - Do NOT make Grafana the primary UI — it's secondary evidence
  - Do NOT build 20+ dashboards — one solid overview dashboard is enough
  - Do NOT add alerting rules in v1

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Dashboard design requires visual layout decisions and metric presentation
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T19, T20 — after T17)
  - **Parallel Group**: Wave 4
  - **Blocks**: F1-F4
  - **Blocked By**: T17

  **References**:

  **External References**:
  - Grafana provisioning: https://grafana.com/docs/grafana/latest/administration/provisioning/
  - prometheus-fastapi-instrumentator: https://github.com/trallnag/prometheus-fastapi-instrumentator

  **WHY Each Reference Matters**:
  - Grafana provisioning: Must follow the correct file structure for automatic dashboard loading

  **Acceptance Criteria**:

  **QA Scenarios:**

  ```
  Scenario: Grafana loads provisioned datasource and dashboard
    Tool: Bash (curl)
    Preconditions: Docker Compose running, Grafana healthy
    Steps:
      1. `curl -sf http://localhost:3100/api/health` → {"database":"ok",...}
      2. `curl -sf -u admin:admin http://localhost:3100/api/datasources` → contains Prometheus datasource
      3. `curl -sf -u admin:admin http://localhost:3100/api/search?type=dash-db` → contains opsflow-overview dashboard
    Expected Result: Grafana has datasource and dashboard provisioned
    Evidence: .sisyphus/evidence/task-18-grafana-provisioned.txt

  Scenario: Prometheus scrapes FastAPI metrics
    Tool: Bash (curl)
    Preconditions: API running with prometheus instrumentator
    Steps:
      1. `curl -sf http://localhost:8000/metrics` → contains http_request_duration_seconds
      2. `curl -sf http://localhost:9090/api/v1/query?query=http_request_duration_seconds_count` → returns data
    Expected Result: Prometheus collecting metrics from API
    Evidence: .sisyphus/evidence/task-18-prometheus-metrics.txt
  ```

  **Commit**: YES
  - Message: `infra: add Grafana dashboards and provisioning`
  - Files: `infra/grafana/`
  - Pre-commit: Verify JSON files are valid

- [x] 19. README.md Rewrite + Architecture Documentation

  **What to do**:
  - Rewrite `README.md` at repo root:
    - Title: "OpsFlow — AI-Native Operational Investigation and Orchestration Platform"
    - Positioning: "governed operational intelligence for distributed technical systems"
    - Key themes: entity-centric investigations, operational memory, policy-bounded orchestration, traceable reasoning, hybrid retrieval, high-stakes operational workflows
    - Tone: calm, technical, operational, senior — NOT AI hype, NOT "autonomous AGI ops", NOT generic agent buzzwords
    - Architecture overview with ASCII diagram showing the investigation flow
    - Quick start: `docker compose up -d`, seed, trigger investigation, view trace
    - Screenshot placeholders (or actual screenshots if running)
    - Roadmap section showing strategic evolution without requiring implementation
  - Create `docs/architecture.md`:
    - System architecture with component diagram
    - Entity model description
    - Investigation flow phases
    - Specialist tool descriptions
    - Governance model explanation
    - Observability approach (Langfuse + Grafana)
    - Design decisions and rationale
  - Create `docs/roadmap.md`:
    - Phase 2 evolution: operational memory graph, cross-incident learning, anomaly clustering
    - Phase 3: reliability state, policy engines, customer/account intelligence
    - Phase 4: bounded automations, operational evaluation harnesses
    - Phase 5: runtime governance, multi-environment operations

  **Must NOT do**:
  - Do NOT use emoji in docs (user did not request)
  - Do NOT add AI hype language ("revolutionary", "game-changing", "autonomous AGI")
  - Do NOT create excessive documentation — README + architecture + roadmap is enough

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Strategic positioning documentation requiring precise technical writing
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T17, T18, T20)
  - **Parallel Group**: Wave 4
  - **Blocks**: F1-F4
  - **Blocked By**: T16

  **References**:

  **Pattern References**:
  - `README.md` — Existing README. Rewrite it, preserving what's valuable but updating positioning.
  - `VISION.md` — Existing vision doc. Much of this can be evolved into architecture.md.
  - `docs/deep-research-report.md` — Market context for positioning language.

  **WHY Each Reference Matters**:
  - Existing README: Has good structural elements (problem, what it does, architecture) that should be evolved
  - VISION.md: Contains design decisions and evolution path that inform the roadmap
  - Research report: Contains market positioning language and competitive landscape for README

  **Acceptance Criteria**:

  **QA Scenarios:**

  ```
  Scenario: README contains all required sections
    Tool: Bash (grep)
    Preconditions: README.md exists
    Steps:
      1. `grep -c "AI-Native Operational Investigation" README.md` → > 0
      2. `grep -c "Quick Start" README.md` → > 0
      3. `grep -c "docker compose" README.md` → > 0
      4. `grep -c "Architecture" README.md` → > 0
      5. `grep -c "Roadmap" README.md` → > 0
    Expected Result: README has all required sections
    Evidence: .sisyphus/evidence/task-19-readme-structure.txt

  Scenario: README does NOT contain AI hype language
    Tool: Bash (grep)
    Preconditions: README.md exists
    Steps:
      1. `grep -ic "revolutionary\|game-changing\|AGI\|autonomous AGI\|magic" README.md` → 0
    Expected Result: No hype language found
    Evidence: .sisyphus/evidence/task-19-no-hype.txt

  Scenario: Architecture doc describes all subsystems
    Tool: Bash
    Preconditions: docs/architecture.md exists
    Steps:
      1. Verify docs/architecture.md exists and is > 100 lines
      2. Verify it mentions: entity model, investigation flow, specialists, governance, observability
    Expected Result: Complete architecture documentation
    Evidence: .sisyphus/evidence/task-19-architecture.txt
  ```

  **Commit**: YES
  - Message: `docs: rewrite README with architecture and demo walkthrough`
  - Files: `README.md`, `docs/architecture.md`, `docs/roadmap.md`
  - Pre-commit: —

- [x] 20. .env.example + Setup Guide + Demo Walkthrough

  **What to do**:
  - Update `.env.example` with all required variables and explanatory comments:
    - Postgres: POSTGRES_USER, POSTGRES_PASSWORD, POSTGRES_DB
    - Langfuse: LANGFUSE_PUBLIC_KEY, LANGFUSE_SECRET_KEY, LANGFUSE_SALT, etc.
    - LLM: LLM_API_BASE, LLM_API_KEY, LLM_MODEL
    - Grafana: GF_SECURITY_ADMIN_PASSWORD
  - Create `docs/setup.md` with step-by-step:
    1. Prerequisites (Docker, Docker Compose)
    2. Clone and configure (`cp .env.example .env`, edit values)
    3. Start stack (`docker compose up -d`)
    4. Wait for health checks
    5. Seed data
    6. Run investigation
    7. View Langfuse trace
    8. View Grafana dashboard
  - Create `docs/demo-walkthrough.md` with the narrative walkthrough:
    - "Meridian Logistics Portland Distribution Center" incident story
    - Step-by-step curl commands with expected output snippets
    - What to look for in the Langfuse trace
    - What the Grafana dashboard shows

  **Must NOT do**:
  - Do NOT require real API keys for demo (LLM can be mocked or use free tier)
  - Do NOT write a full deployment guide — this is for local demo only

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Documentation and configuration templates
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T18, T19 — after T17)
  - **Parallel Group**: Wave 4
  - **Blocks**: F1-F4
  - **Blocked By**: T17

  **References**:

  **Pattern References**:
  - `.env.example` (T6) — Existing skeleton to expand
  - `docker-compose.yml` (T17) — All env vars must match compose service configs

  **WHY Each Reference Matters**:
  - T17 compose: Every env var in .env.example must correspond to a variable in docker-compose.yml

  **Acceptance Criteria**:

  **QA Scenarios:**

  ```
  Scenario: Setup guide is reproducible from clean environment
    Tool: Bash
    Preconditions: Docker available, repo cloned
    Steps:
      1. `cp .env.example .env`
      2. Edit .env with test values
      3. `docker compose up -d`
      4. Wait for health checks
      5. `curl -sf http://localhost:8000/healthz`
      6. `curl -sf -X POST http://localhost:8000/api/v1/seed`
      7. `curl -sf -X POST http://localhost:8000/api/v1/investigations -H 'Content-Type: application/json' -d '{"signal_ids":{"ticket_id":"TCK-1001","alert_id":"ALT-2001","event_id":"EVT-3001"}}'`
    Expected Result: All steps complete without errors
    Evidence: .sisyphus/evidence/task-20-setup-reproducible.txt

  Scenario: Demo walkthrough mentions Meridian Logistics narrative
    Tool: Bash (grep)
    Preconditions: docs/demo-walkthrough.md exists
    Steps:
      1. `grep -c "Meridian" docs/demo-walkthrough.md` → > 0
      2. `grep -c "TCK-1001" docs/demo-walkthrough.md` → > 0
      3. `grep -c "Langfuse" docs/demo-walkthrough.md` → > 0
    Expected Result: Walkthrough references the narrative
    Evidence: .sisyphus/evidence/task-20-demo-walkthrough.txt
  ```

  **Commit**: YES
  - Message: `docs: add .env.example and setup guide`
  - Files: `.env.example`, `docs/setup.md`, `docs/demo-walkthrough.md`
  - Pre-commit: —

## Success Criteria

### Verification Commands
```bash
# All services healthy
docker compose up -d && docker compose ps  # Expected: all services "healthy" or "running"

# Python tests pass
cd python && pytest tests/ -q  # Expected: all pass

# Health endpoint
curl -sf http://localhost:8000/healthz  # Expected: {"status":"ok"}

# Seed data
curl -sf -X POST http://localhost:8000/api/v1/seed  # Expected: deterministic entity counts

# Investigation flow
curl -sf -X POST http://localhost:8000/api/v1/investigations \
  -H 'Content-Type: application/json' \
  -d '{"signal_ids": {"ticket_id": "TCK-1001", "alert_id": "ALT-2001", "event_id": "EVT-3001"}}'
# Expected: complete investigation with trace_id

# Langfuse trace
curl -sf http://localhost:3000  # Expected: Langfuse UI accessible

# Grafana dashboards
curl -sf http://localhost:3000/api/health  # Expected: {"database": "ok", "commit": "..."}
# Note: Grafana runs on port 3000 only if Langfuse uses different port. Adjust in compose.
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All pytest tests pass
- [ ] Docker Compose stack healthy
- [ ] Investigation flow returns complete response
- [ ] Langfuse shows full trace
- [ ] Grafana shows operational dashboards
- [ ] Rust CLI behavior unchanged
- [ ] README walkthrough reproducible
