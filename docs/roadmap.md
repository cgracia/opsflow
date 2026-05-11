# Roadmap

Where OpsFlow is headed. Each version is independently useful. No version requires the next to deliver value.

## v1: Multi-Signal Convergence Investigation (Current)

A single end-to-end investigation pipeline that proves the core thesis: structured, governed, entity-centric reasoning over operational signals.

**What works now:**

- Seven-phase investigation pipeline: Signal Ingestion through Output Generation
- Ten entity types modeling Account, Site, Fleet, Device, Deployment, Service, SoftwareRevision, Incident, Ticket, OperationalEvent
- Hybrid retrieval from Qdrant (dense + sparse vectors, reciprocal rank fusion, entity filtering)
- Two specialist investigators: Telemetry (device metrics analysis) and Historical Incident (pattern matching, deployment adjacency)
- Governance engine with five action categories, EXECUTE always blocked, mandatory escalation for high-severity customer-facing incidents
- Full trace emission to Langfuse for every investigation phase
- Synthetic seed data for demonstration and development
- Docker Compose stack: Postgres, Qdrant, Langfuse (with ClickHouse and Redis), Prometheus, Grafana

**What validates this version:**

- Running an investigation from ticket + alert + event signals produces a coherent operator briefing
- The governance layer correctly gates actions and forces escalation where policy demands it
- The full reasoning chain is inspectable in Langfuse traces
- The entity model supports realistic operational hierarchies

## v2: Operational Memory

Moving from single investigations to accumulated operational knowledge. The system learns from patterns across incidents.

**Cross-incident learning.** When the same failure pattern appears three times across different accounts, the system should recognize it without requiring manual correlation. This means storing investigation results in a queryable form and cross-referencing new incidents against past investigations.

**Anomaly clustering.** Telemetry anomalies that appear across multiple devices in the same fleet during the same time window are probably related. Grouping these automatically reduces noise and surfaces systemic issues that individual investigations miss.

**Account intelligence graph.** An account that has had three navigation-related incidents in two months, all following software updates, has a different risk profile than one with isolated incidents. Building a per-account intelligence layer means investigations start with richer context.

**Key difference from v1:** v1 treats each investigation as isolated. v2 makes each investigation smarter by connecting it to everything that came before.

## v3: Reliability State

Adding explicit reliability modeling and policy enforcement. The system tracks not just what happened, but the health state of the entities it monitors.

**Policy engines.** Configurable rules that define what governance looks like per account tier, per severity level, per entity type. Enterprise accounts get tighter escalation requirements. Critical infrastructure gets different remediation windows. Policies are data, not code.

**Customer and account intelligence.** Deepening the per-account knowledge layer. How many incidents has this account had? What is their mean time to resolution? What is their historical severity distribution? This context feeds into investigation prioritization and governance decisions.

**Deployment correlation engine.** Explicit modeling of the relationship between software deployments and incident spikes. When a deployment goes out and incidents rise within a time window, the system should flag the correlation proactively rather than waiting for a human to notice the pattern.

**Key difference from v2:** v2 accumulates knowledge. v3 makes that knowledge actionable through policies, state tracking, and proactive correlation.

## v4: Bounded Automations

Carefully scoped actions the system can suggest and, in limited cases, execute. Still governed. Still human-in-the-loop for anything that touches production.

**Remediation suggestions.** Based on historical patterns, the system can suggest specific remediation steps. "The last three times this pattern appeared, a rollback to the previous software revision resolved it within 30 minutes." Suggestions, not actions.

**Rollback recommendations.** When deployment correlation detects a strong signal (incidents spike immediately after a rollout to fleet X), the system produces a structured rollback recommendation with evidence, risk assessment, and affected entities.

**Evaluation harnesses.** Automated evaluation of investigation quality against historical data. Did the system retrieve the right evidence? Did it call the right specialist? Did governance gate correctly? Continuous evaluation against labeled datasets, not one-off manual review.

**Key difference from v3:** v3 observes and correlates. v4 begins to recommend specific actions with evidence-backed justification.

## v5: Runtime Governance

Multi-environment operations with advanced autonomy under strict human checkpoints. This is the long-term direction, not the current plan.

**Multi-environment operations.** Running investigations across staging, canary, and production environments simultaneously. Comparing incident patterns between environments to detect regressions before they hit production.

**Advanced autonomy with human checkpoints.** For well-understood failure modes with high-confidence patterns and low blast radius, the system could propose bounded automated actions. Every such action requires explicit human approval. The governance model evolves from "block everything beyond recommendations" to "allow specific actions under specific conditions with specific approvals."

**Key difference from v4:** v4 recommends. v5 acts, but only within tightly defined boundaries and always with human authorization.

---

Each version builds on the previous one, but none is a prerequisite for the next to be useful. v1 delivers value on its own. v2 makes v1 smarter. v3 makes v2 actionable. v4 adds teeth. v5 extends reach.
