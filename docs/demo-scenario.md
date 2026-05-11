# Demo Scenario: Meridian Logistics Navigation Incident

> **All entities, data, and events in this scenario are synthetic.** Meridian Logistics is a fictional company. No real operational data is used.

## Scenario

Meridian Logistics operates a fleet of autonomous mobile devices at its Portland Distribution Center. A fleet-wide software update to version 3.3.0 is rolled out to Warehouse Alpha Fleet (FLT-101). Within 30 minutes, three of the eight updated devices (DEV-401, DEV-402, DEV-403) stop navigating — all reporting `NAV_PATH_PLAN_FAILED` errors. Three separate signals arrive nearly simultaneously: a support ticket from an operator, an anomaly alert from fleet monitoring, and a device-blocked operational event. The system must recognize these as the same incident and investigate.

**Key entities:**

| Type | ID | Name |
|------|----|------|
| Account | ACC-1001 | Meridian Logistics (enterprise) |
| Site | SITE-2001 | Portland Distribution Center |
| Fleet | FLT-101 | Warehouse Alpha Fleet |
| Devices | DEV-401/402/403 | Affected (error/degraded, v3.3.0) |
| Deployment | DEPL-501 | v3.3.0 rollout (in progress, halted) |

## Running the investigation

After starting the stack and seeding data (see [setup guide](setup.md)):

```bash
curl -sf -X POST http://localhost:8000/api/v1/investigations \
  -H 'Content-Type: application/json' \
  -d '{"signal_ids": {"ticket_id": "TCK-1001", "alert_id": "ALT-2001", "event_id": "EVT-3001"}}'
```

## What the system returns

The investigation response includes:

**Entity context** — The system resolves the three signals to the affected entity graph: Meridian Logistics account, Portland site, Warehouse Alpha Fleet, three affected devices on v3.3.0, and the in-progress deployment.

**Evidence** — Hybrid search retrieves relevant documents from Qdrant: historical tickets describing similar navigation failures after past updates, runbooks for navigation troubleshooting and fleet rollback, telemetry snapshots showing the error rate spike (0.2% baseline to 47.3%), and the deployment manifest showing the v3.3.0 rollout timeline.

**Specialist reports:**

- *Telemetry investigator* detects navigation error rate anomalies and sensor fusion latency elevation, with temporal correlation to the deployment.
- *Historical investigator* finds past incidents with the same pattern (post-update navigation failures requiring rollback), identifies deployment adjacency, and surfaces relevant runbooks.

**Hypotheses** — The system generates ranked hypotheses. The primary hypothesis (typically ~0.85 confidence) is that software v3.3.0 introduced a navigation regression, based on the convergence of telemetry anomalies, deployment timing, and historical precedent.

**Governance decision** — The governance engine classifies the action as ESCALATE (high severity, enterprise account). EXECUTE is blocked. Mandatory human-in-the-loop review is flagged.

**Operator briefing** — A structured internal briefing containing entity context, primary hypothesis with confidence, telemetry findings, historical patterns, and governance decision.

**Customer response draft** — A safe, non-technical summary suitable for external communication.

## Viewing the trace in Langfuse

Open `http://localhost:3000` (credentials: `opsflow@example.com` / `admin`). Navigate to Traces and select the investigation trace. You'll see nested spans for each of the 7 phases, with evidence retrieved, specialist findings, hypotheses generated, and governance decisions — the full reasoning chain is inspectable end-to-end.
