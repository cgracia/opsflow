# Repository Guidelines

**This file is the canonical agent reference for this repository.** `CLAUDE.md`
is a pointer to it. Keep guidance here; do not fork it into a second file.

## What this repo is

OpsFlow is an incident investigation layer: it converges signals from multiple
sources (alerts, uptime outages, notifications) into a single governed
investigation over an entity graph, retrieves evidence, consults specialists,
forms ranked hypotheses, and produces bounded, traceable outputs. A human
decides and executes; the system investigates and recommends.

**Public repository.** See "Disclosure" below before writing anything into it.

## Current state, stated honestly

The v1 pipeline is architecturally complete and empirically hollow. It runs
end-to-end, but over synthetic seed data describing a fictional company
("Meridian Logistics"), with several components simulated:

| Component | Reality |
|---|---|
| Dense vectors | SHA-256 hashes of document IDs, not embeddings. Not semantically meaningful |
| Sparse vectors | Whitespace-split word-count bags, not BM25 |
| Entity resolution | Hardcoded for the one seeded scenario |
| Both specialists | Keyword matchers. LLM path exists but is optional and off by default |
| All evidence | 11 invented documents |
| Signal connectors | Stubs. Accept IDs, connect to nothing |

**Do not describe any of these as working.** The README's "What is simulated or
stubbed" section is deliberate and must stay accurate: if you make one of them
real, delete its entry; if you add a new simulation, add an entry.

## Where this is going

Active plan: **`.sisyphus/plans/opsflow-homelab.md`**. Read it before starting
work. Summary: point the same architecture at a real self-hosted service estate
(~20 Docker Compose stacks on a NixOS host, in a separate private infra repo),
migrate the workload onto k3s provisioned with Terraform, and replace the
simulated components with real ones. Every simulation above is a consequence of
having no real signal source, not a design flaw.

In-flight structural decisions from that plan:

- The Rust CLI was **extracted to the `praxis` repository** (2026-09-22) with
  its 29 commits of history intact. OpsFlow is Python-only. If you are looking
  for `think`, `collect`, `triage` or the ccusage ingestion, it is over there.
- Langfuse, ClickHouse and Redis stay on Docker; they do not move to the cluster.
- Meridian Logistics seed data is demoted to a test fixture, out of the README.

## Project structure

```
python/app/
  api/            FastAPI routes; thin — validate, resolve deps, delegate
  orchestrator/   InvestigationManager; owns the 7 phases in fixed order
  specialists/    telemetry.py, historical.py — domain-scoped investigator tools
  retrieval/      Qdrant hybrid search (dense + sparse, RRF fusion)
  governance/     classification + engine; EXECUTE hardcoded blocked in v1
  models/         SQLAlchemy 2.0 async models, one file per entity type
  schemas/        Pydantic v2 request/response schemas
  seed/           synthetic entity + evidence loaders
  tracing/        Langfuse span emission, one span per phase
  db/             async session and declarative base
  llm/            OpenAI-compatible client + prompts
python/alembic/   migrations
python/tests/     pytest, unit/ mirrors app/
docs/             architecture.md, roadmap.md, demo-scenario.md, safety-and-scope.md, setup.md
docs/architecture/  LikeC4 model + generated assets; published to GitHub Pages
.sisyphus/plans/  active and archived plans. TRACKED AND PUBLIC
```

## The seven phases

Fixed order, typed in and out at every stage, one trace span each:

1. Signal Ingestion → 2. Entity Resolution → 3. Evidence Retrieval →
4. Specialist Investigation → 5. Hypothesis Generation →
6. Governance Evaluation → 7. Output Generation

**This is not an autonomous agent and should not become one.** Fixed phases mean
every investigation is structurally comparable and every trace has the same
shape. Dynamic planning adds complexity without adding value here. If you are
tempted to add a phase, add a specialist instead.

## Entity model

```
Account → Site → Fleet → Device → Service
                 Site → Deployment → SoftwareRevision
                 Site → Incident → {Ticket, OperationalEvent}
```

Investigations are **entity-centric, not ticket-centric**. A ticket is one
signal type among several; the entity graph is what stays constant regardless
of which signal arrived first. Preserve this when remapping to real
infrastructure — the hierarchy is the design, the labels are not.

## Build, test and development commands

```sh
cd python
uv sync --extra dev                  # dev is an optional-extra, NOT a
                                     # dependency-group: bare `uv sync` omits pytest/ruff
uv run pytest                        # full suite
uv run pytest tests/unit/test_x.py   # one file
uv run ruff check . && uv run ruff format --check .
uv run alembic upgrade head          # apply migrations
uv run alembic revision --autogenerate -m "msg"
```

Stack:

```sh
cp .env.example .env                 # then set keys; works without LLM_API_KEY
docker compose up -d
curl -sf http://localhost:8000/api/v1/healthz
curl -sf -X POST http://localhost:8000/api/v1/seed
curl -sf -X POST http://localhost:8000/api/v1/investigations \
  -H 'Content-Type: application/json' \
  -d '{"signal_ids":{"ticket_id":"TCK-1001","alert_id":"ALT-2001","event_id":"EVT-3001"}}'
```

Ports: API `:8000`, Qdrant `:6333`, Langfuse `:3000`, Grafana `:3100`,
Postgres `:5432`.

## Coding conventions

- **Python**: 3.12, ruff for lint and format (line-length 100), Pydantic v2, SQLAlchemy 2.0 async
  style (`Mapped[...]` / `mapped_column`). Type hints on public functions.
- **Async throughout.** The DB session, the Qdrant client and the LLM client are
  all async; do not introduce sync blocking calls into a request path.
- **Graceful degradation is a design rule, not an accident.** Specialists and
  hypothesis generation must work with no LLM configured. Never make an LLM key
  a hard requirement for a code path that has a rule-based fallback.
- **One file per entity model**, mirroring `models/` in `schemas/`.
- **Every phase emits a trace span.** A new phase or specialist without tracing
  is incomplete.
- **Commits**: Conventional Commits (`feat:`, `fix:`, `docs:`, `chore:`,
  `test:`, `refactor:`), scoped where useful — `feat(python):`, `fix(ingest):`.

## Testing

- `python/tests/unit/` mirrors `app/`. New module → matching test file.
- Tests must not require a running stack, an LLM key, or network access.
  Fixtures and fakes, not live services.
- Seed data doubles as the integration fixture; keep it deterministic.
- CI (`.github/workflows/ci.yml`) gates the Python half: `ruff check`,
  `ruff format --check`, `pytest`. It must stay green.
- **Ruff is pinned exactly and the rule set is declared explicitly** in
  `pyproject.toml`. Ruff's default rules change between releases, so an
  unpinned version plus an implicit select means the gate moves on its own.
  Do not loosen either to make a check pass.

## Safety and scope

These are commitments the repo makes publicly in `docs/safety-and-scope.md`.
They are not negotiable by an agent.

- **No employer, customer or proprietary data, ever.** All demo data is
  synthetic. Contributions introducing real-world operational data from any
  company are out of scope. When real signals are wired up, they come from the
  author's own infrastructure only.
- **EXECUTE stays blocked.** The system investigates and recommends. If the
  homelab work makes remediation actions concrete, they are *proposed* and gated
  behind out-of-band human approval — the block does not get relaxed to ship a
  feature.
- **Do not overstate maturity.** Alpha, no security review, default credentials,
  not for production. Keep README and `docs/architecture.md`'s implementation
  status table honest; they are the repo's credibility.

## Disclosure

This repository is **public**, and `.gitignore` explicitly un-ignores
`.sisyphus/plans/`, so plan files are tracked and world-readable.

Keep the technical rationale here and the personal rationale out. Nothing about
the author's career, job search, positioning or timelines belongs in this repo —
not in plans, not in commit messages, not in the write-up. That material lives
in a separate private repository. If a plan needs motivating context that is not
purely technical, write the technical half here and the rest there.

Never commit `.env`, credentials, or anything decrypted out of the infra repo's
SOPS secrets. Terraform state must not be committed; choose a remote backend.

## Working on NixOS

uv downloads a generic-linux Python and generic wheels, neither of which runs on
NixOS out of the box. What works locally:

```sh
uv sync --extra dev --locked --python "$(which python3)"
nix run nixpkgs#ruff -- check .              # the wheel's ruff binary will not run
LD_LIBRARY_PATH="$(nix build --no-link --print-out-paths nixpkgs#stdenv.cc.cc.lib)/lib" \
  uv run --python "$(which python3)" pytest  # numpy et al need libstdc++
```

None of this applies in CI, which runs on ubuntu-latest where the wheels are fine.
