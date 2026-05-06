# Demo Walkthrough: Meridian Logistics Incident

## The Scenario

Meridian Logistics is a national logistics company operating a fleet of autonomous delivery vehicles. On May 5, 2026, operations in the Portland Distribution Center detected an unusual pattern: three autonomous devices (DEV-401, DEV-402, DEV-403) stopped navigating to delivery locations after a fleet-wide software update to version 3.3.0.

This isn't an isolated device failure. Three devices in the same fleet, within a 30-minute window, all experiencing the same symptom after the same software version rollout. The scale and simultaneity trigger the OpsFlow investigation system.

### Context

- **Company:** Meridian Logistics
- **Site:** Portland Distribution Center
- **Fleet:** FLEET-SEATTLE (actually deployed at Portland)
- **Affected Devices:** DEV-401, DEV-402, DEV-403 (3 of 47 devices in fleet)
- **Software Version:** 3.3.0 (just released)
- **Incident Window:** May 5, 2026, 14:00-14:30 UTC

The signal sources:

1. **Ticket TCK-1001:** Support ticket opened at 14:00 UTC from DEV-401 operator reporting navigation failure
2. **Alert ALT-2001:** Fleet monitoring alert at 14:15 UTC detecting 3 devices with stopped navigation status
3. **Event EVT-3001:** Deployment event log showing software update v3.3.0 applied to FLEET-SEATTLE at 13:45 UTC

These three signals converge into a single incident requiring coordinated investigation.

## Triggering the Investigation

### Step 1: Submit Signals

OpsFlow accepts signals via the API:

```bash
curl -sf -X POST http://localhost:8000/api/v1/investigations \
  -H 'Content-Type: application/json' \
  -d '{
    "signal_ids": {
      "ticket_id": "TCK-1001",
      "alert_id": "ALT-2001",
      "event_id": "EVT-3001"
    }
  }'
```

### Step 2: What Happens Next

OpsFlow immediately:

1. **Validates signals** — Checks TCK-1001, ALT-2001, EVT-3001 exist in the system
2. **Maps to entities** — Connects signals to Account: MERIDIAN-LOGISTICS, Site: PORTLAND-DC, Fleet: FLEET-SEATTLE
3. **Dispatches investigation** — Creates INV-1001 and begins 7-phase pipeline

### Step 3: API Response

```json
{
  "investigation_id": "INV-1001",
  "status": "in_progress",
  "entity_context": {
    "account": "MERIDIAN-LOGISTICS",
    "site": "PORTLAND-DC",
    "fleet": "FLEET-SEATTLE",
    "devices": ["DEV-401", "DEV-402", "DEV-403"],
    "deployment": "DEP-1001",
    "software_revision": "v3.3.0"
  },
  "hypotheses": [],
  "governance_decision": "INVESTIGATE",
  "operator_briefing": "Investigation started for 3 devices in FLEET-SEATTLE...",
  "customer_response_draft": "Investigation in progress. We are analyzing signals from TCK-1001..."
}
```

The response shows the investigation was created and mapped to the affected entities.

## Understanding the Investigation

### 7-Phase Pipeline

OpsFlow executes the investigation in sequence:

#### Phase 1: Signal Ingestion

OpsFlow normalizes the three signals:

- **TCK-1001** (ticket): "DEV-401 stopped navigating at 14:00. Operator reported device stuck at route."
- **ALT-2001** (alert): "3 devices in FLEET-SEATTLE have navigation_state=STOPPED for > 15 minutes."
- **EVT-3001** (event): "DEP-1001: Fleet-wide rollout of software v3.3.0 completed at 13:45 UTC."

All three are merged into a unified signal set for the investigation.

#### Phase 2: Entity Resolution

OpsFlow maps signals to the OpsFlow entity hierarchy:

- **Account:** MERIDIAN-LOGISTICS
- **Site:** PORTLAND-DC
- **Fleet:** FLEET-SEATTLE (47 devices)
- **Devices:** DEV-401, DEV-402, DEV-403 (3 affected)
- **Deployment:** DEP-1001 (software rollout)
- **Software Revision:** v3.3.0

OpsFlow understands this is a fleet-wide event affecting a subset of devices.

#### Phase 3: Evidence Retrieval

OpsFlow queries Qdrant (hybrid vector database) with multiple strategies:

**Dense semantic search:**
- Query: "autonomous navigation failure after software update 3.3.0"
- Fetched: 15 evidence items including past incidents, runbook snippets, deployment notes

**Sparse keyword search:**
- Query: "DEV-401 DEV-402 DEV-403 stopped navigation fleet Seattle"
- Fetched: 8 evidence items including device logs, telemetry data

**Reciprocal Rank Fusion (RRF):**
- Combined ranked list: 23 relevant items

The system retrieves evidence across:
- Previous incidents in FLEET-SEATTLE
- Deployment history for DEP-1001
- Device telemetry logs for DEV-401, DEV-402, DEV-403
- Runbook sections on "navigation regression post-update"

#### Phase 4: Specialist Investigation

OpsFlow dispatches to domain-specific investigators:

**Telemetry Investigator:**
- Analyzes device telemetry for DEV-401, DEV-402, DEV-403
- Finds: GPS coordinates stuck at same location (Portland DC entrance)
- Network: All devices report "navigation_module_error: ROUTE_PLANNING_FAILURE"
- Timing: Errors started at 14:00 UTC, coinciding with update deployment completion

**Historical Incident Investigator:**
- Searches past incidents in FLEET-SEATTLE for similar patterns
- Finds: v3.2.1 rollout on 2025-11-15 caused similar navigation regression on 5 devices
- Pattern: Route planning module bug introduced in v3.3.0

#### Phase 5: Hypothesis Generation

OpsFlow synthesizes evidence into ranked hypotheses:

**HYP-1001: Software update v3.3.0 introduced navigation regression**
- Confidence: 0.87
- Supporting evidence: Telemetry shows route planning errors at 14:00, historical precedent in v3.2.1
- Unlikely to be false positive: All 3 devices share exact same error, timing matches deployment

**HYP-1002: Hardware failure affecting all 3 devices simultaneously**
- Confidence: 0.23
- Supporting evidence: Devices are in same fleet, share same site
- Contradicting evidence: No hardware error logs, simultaneous failures extremely unlikely for different devices

**HYP-1003: GPS signal interference in Portland DC**
- Confidence: 0.08
- Supporting evidence: Devices stuck at same location
- Contradicting evidence: Only FLEET-SEATTLE affected, GPS logs show valid signal, network errors specific to navigation module

The investigation proceeds with HYP-1001 as the leading hypothesis.

#### Phase 6: Governance Evaluation

OpsFlow classifies the action and gates sensitive operations:

**Action Classification:**
- **INVESTIGATE** — Ongoing investigation, not resolved

**Gating Decisions:**
- Does not recommend automatic rollback (governance blocks EXECUTE actions)
- Requires human approval for any production changes
- Customer communication flagged for operator review

**Escalation Path:**
- High-severity customer-facing incident
- Flagged for operator briefing and customer draft review
- Engineering team notified

#### Phase 7: Output Generation

OpsFlow produces two bounded outputs:

**Operator Briefing (internal):**
"Investigation completed. Root cause: software update v3.3.0 introduced navigation regression in route planning module. 3/47 devices in FLEET-SEATTLE affected. Recommendation: deploy hotfix v3.3.1. Approve rollback to v3.2.1 if hotfix fails."

**Customer Response Draft (safe):**
"Meridian Logistics is investigating reports of navigation issues in a subset of autonomous delivery devices. We have identified a potential software regression in version 3.3.0 affecting route planning functionality. Our engineering team is actively working on a fix. No customer impact confirmed beyond the affected devices. We will provide updates as investigation progresses."

## Viewing the Investigation in Langfuse

### Navigate to Traces

1. Open http://localhost:3000
2. Login with: opsflow@example.com / admin
3. Click "Traces" in the sidebar
4. Select trace INV-1001

### What You'll See

**Span Hierarchy:**

```
INV-1001 (root)
├── Signal Ingestion (Phase 1)
│   ├── Ticket: TCK-1001 - normalized
│   ├── Alert: ALT-2001 - normalized
│   └── Event: EVT-3001 - normalized
├── Entity Resolution (Phase 2)
│   ├── Account mapping
│   ├── Site mapping
│   ├── Fleet mapping
│   └── Device mapping
├── Evidence Retrieval (Phase 3)
│   ├── Dense semantic search (15 results)
│   ├── Sparse keyword search (8 results)
│   └── RRF fusion (23 combined)
├── Specialist Investigation (Phase 4)
│   ├── Telemetry Investigator
│   │   └── Device telemetry analysis
│   └── Historical Incident Investigator
│       └── Past incidents search
├── Hypothesis Generation (Phase 5)
│   └── Top hypothesis: v3.3.0 regression (87% confidence)
├── Governance Evaluation (Phase 6)
│   ├── Action classification: INVESTIGATE
│   ├── Risk assessment: HIGH
│   └── Human-in-the-loop flag: YES
└── Output Generation (Phase 7)
    ├── Operator briefing
    └── Customer draft
```

**Key Insights:**

1. **Tool Calls:** See all LLM prompts and tool invocations
2. **Evidence Items:** Click on specific evidence items from Qdrant
3. **Confidence Scores:** View hypothesis confidence breakdown
4. **Prompt Tokens:** See token usage per phase

## Viewing the Investigation in Grafana

### Navigate to Dashboards

1. Open http://localhost:3100
2. Login with: admin / admin
3. Select "OpsFlow Investigations" dashboard

### What You'll See

**Metrics Displayed:**

- **Investigation Status:** All investigations by status (IN_PROGRESS, COMPLETED, ESCALATED)
- **Response Time:** Time from signal ingestion to output generation (for INV-1001: 42 seconds)
- **Hypothesis Confidence:** Distribution of confidence scores
- **Governance Flags:** Count of human-in-the-loop escalations
- **Phase Duration:** Time spent in each of the 7 phases

**For INV-1001 Specifically:**

- **Phase Duration:**
  - Signal Ingestion: 2 seconds
  - Entity Resolution: 1 second
  - Evidence Retrieval: 8 seconds
  - Specialist Investigation: 12 seconds
  - Hypothesis Generation: 7 seconds
  - Governance Evaluation: 4 seconds
  - Output Generation: 8 seconds

- **LLM Usage:**
  - Total tokens: 2,847
  - Total cost (estimated): $0.0024 (demo uses synthetic data, real LLM would incur cost)

## Summary

This walkthrough demonstrates how OpsFlow handles a real-world operational incident:

1. **Multi-signal convergence** — Combines ticket, alert, and event into single investigation
2. **Entity-centric reasoning** — Understands fleet-wide patterns, not isolated failures
3. **Hybrid evidence retrieval** — Fuses semantic and keyword search for comprehensive results
4. **Specialist investigation** — Dispatches domain-specific tools for deeper analysis
5. **Governed output** — Produces safe, bounded outputs for operators and customers
6. **Full traceability** — Every phase is visible in Langfuse for debugging and evaluation

The system doesn't just "find" the issue — it explains it, classifies the action, gates sensitive operations, and produces customer-safe communication. All while maintaining complete traceability of the reasoning process.
