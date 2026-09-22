# OpsFlow → homelab control plane

**Goal:** turn OpsFlow from a demo over synthetic data into an incident
investigation layer for a real self-hosted service estate — running on a k3s
cluster provisioned with Terraform, investigating the infrastructure it runs on.

**Target:** 2026-09-30.

## Why this shape

The v1 pipeline is architecturally complete and empirically hollow: dense
vectors are SHA-256 hashes, sparse vectors are word-count bags, entity
resolution is hardcoded for one seeded scenario, both specialists are keyword
matchers, and all 11 evidence documents are invented. Every one of those is a
consequence of having no real signal source, not of a design flaw. Pointing the
same architecture at a real estate fixes them as a group.

The estate in question is a NixOS host running ~20 Docker Compose stacks, and it
supplies exactly the signals the architecture already models:

- **Alertmanager currently routes every alert to a receiver named `"null"`, and
  Prometheus has no `rule_files` at all.** There is a real, unmet operational
  gap to fill rather than a contrived one.
- **Every image is pinned by SHA digest**, with Diun watching for updates and
  Renovate bumping them. `SoftwareRevision` and deployment adjacency become real
  data rather than seeded fiction.
- **Deployments are git-driven** via an `infra-deploy` helper, so every rollout
  is a commit with a timestamp. The strongest signal in the v1 demo — did a
  rollout coincide with the incident? — becomes a `git log` query.
- **Ollama is already running** with a local model cache: real embeddings and a
  real LLM on own hardware, no API key, no external data egress.

## Entity model remap

| v1 (synthetic) | Homelab (real) |
|---|---|
| Account / Site | host |
| Fleet | compose stack / k8s namespace |
| Device → Service | container → process |
| Deployment | an `infra-deploy` run = a commit to the infra repo |
| SoftwareRevision | pinned image digest |
| Ticket / Alert / Event | uptime outage / Alertmanager alert / notification |

The hierarchy survives intact. That is the main evidence that the v1 entity
model was designed rather than fitted to its demo.

## Scope cuts, decided up front

- The Rust CLI (`src/`, `Cargo.toml`) is extracted to its own repository.
  OpsFlow becomes Python-only and one coherent thing.
- Langfuse, ClickHouse and Redis do **not** move to the cluster. Tracing stays
  optional against a Docker-hosted Langfuse, or is dropped for now.
- **One** specialist done properly, not two done badly.
- The Meridian Logistics seed data is demoted to a test fixture and leaves the
  README.

---

## Day 1 — repo surgery + bare cluster

Cluster first, because hardware is the only unknown-unknown in the plan.
Everything after it is estimable work.

- [ ] Add `LICENSE` (MIT). The README has claimed MIT with no file present
      since the repo was created.
- [ ] Extract `src/`, `Cargo.toml`, `Cargo.lock` and the praxis test fixtures
      to a separate repo, preserving history via `git subtree split`.
- [ ] Rewrite the README lede: what this is now, not what it demoed.
- [ ] Stand up k3s. The `services.k3s-home` NixOS module already exists and is
      currently enabled on zero hosts.
- [ ] Stop when `kubectl get nodes` returns Ready.

## Day 2–3 — real signals in

- [ ] `POST /api/v1/signals/alertmanager`, matching Alertmanager's webhook
      payload schema.
- [ ] Replace the `"null"` receiver in `alertmanager.yml` with a webhook
      pointing at OpsFlow.
- [ ] Write the first real Prometheus alert rules: container down, restart loop,
      disk pressure, node exporter absent, backup job failure.
- [ ] Uptime-monitor webhook normalised into the same signal shape.
- [ ] **Entity ingestion**: parse `compose/*/docker-compose.yml` into the entity
      graph — host → stack → container → pinned digest. This is what replaces
      demo-shaped entity resolution with the real thing.
- [ ] **Deployment ingestion**: infra-repo `git log` → `Deployment` records with
      timestamps, so deployment adjacency is computed against real rollouts.

## Day 4 — replace the simulated components

- [ ] Real embeddings via Ollama (`nomic-embed-text`), replacing the SHA-256
      hash vectors. This is the single largest credibility gap in the README.
- [ ] Real sparse vectors (fastembed BM25 or Qdrant-native), replacing the
      word-count bags.
- [ ] Historical specialist reworked against real evidence: past incidents,
      `git log`, image update history. Telemetry specialist against real
      Prometheus queries — or cut it and ship one specialist well.
- [ ] Delete the corresponding "What is simulated or stubbed" README entries,
      because they will no longer be true.

## Day 5–6 — Terraform + migration

- [ ] Terraform root module using the `kubernetes` (and `helm` where needed)
      providers: namespace, deployments, services, secrets, PVCs, ingress.
- [ ] Migrate `api`, `postgres` and `qdrant` onto the cluster via Terraform.
- [ ] **State backend decision required before writing any HCL.** A committed
      local `.tfstate` is not acceptable in a public repo — pick a remote
      backend, or document the choice explicitly.
- [ ] Cross-host reachability: OpsFlow on the cluster reading Prometheus, the
      uptime monitor and the Docker estate on the Compose host.

## Day 7 — make governance load-bearing

The governance engine currently gates nothing, which is why it reads as
enterprise cosplay. Pointed at a real cluster it becomes the most interesting
component in the repo.

- [ ] `RECOMMEND` outputs become concrete and specific: `kubectl rollout undo`,
      `docker compose restart <svc>`, "revert image to digest `sha256:…`".
- [ ] `EXECUTE` stays blocked — but now blocks something real.
- [ ] Approval path: escalation fires a notification carrying the proposed
      action. A human approves out of band.

## Day 8 — write-up

- [ ] Public write-up: the thesis, the migration, the architecture, what broke
      during the migration and why, what the system has actually caught.
- [ ] **Best case: one real incident it genuinely investigated**, with the
      trace. Worth deliberately provoking one — bump an image to a broken
      digest and watch it converge the alert with the deployment event.

---

## Slip protocol

If this runs late, the order of sacrifice is: Day 7 governance → Day 4 sparse
vectors → the second specialist.

**Do not sacrifice Day 5–6.** The Terraform and k3s work is the half with no
prior art anywhere in the author's public repositories, and it is the half that
cannot be reconstructed from the v1 codebase.
