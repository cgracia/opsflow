"""Live-pipeline integration tests against real Postgres and Qdrant.

Each test names the v1 defect it reproduces; the assertions verify the
correction. These run against the actual services — mocks are not allowed in
this directory.
"""

import re

import pytest
from httpx import AsyncClient

# Tests share one session-scoped event loop with the session-scoped client
# fixture (asyncpg connections are loop-bound).
pytestmark = pytest.mark.asyncio(loop_scope="session")

SCENARIO_A = {"ticket_id": "TCK-1001", "alert_id": "ALT-2001", "event_id": "EVT-3001"}
SCENARIO_B = {"ticket_id": "TCK-2001", "event_id": "EVT-4001"}
SCENARIO_C = {"ticket_id": "TCK-3001", "alert_id": "ALT-4002", "event_id": "EVT-4002"}

# Phrases that a weak-evidence output must never contain. Note "resolution
# time" is deliberately absent: the honest draft *declines* to state one.
FORBIDDEN_WHEN_WEAK = (
    "identified the root cause",
    "root cause has been identified",
)
FORBIDDEN_ETA = re.compile(r"within the next \d|resolution (?:within|expected)")


async def _investigate(client: AsyncClient, signals: dict) -> dict:
    response = await client.post("/api/v1/investigations", json={"signal_ids": signals})
    assert response.status_code == 200, response.text
    return response.json()


async def test_seed_populates_three_disjoint_scenarios(api_client: AsyncClient):
    """v1 /seed 500'd on a fresh database (no tables); it must succeed and
    seed three scenarios so distinct investigations can be verified."""
    response = await api_client.post("/api/v1/seed")
    assert response.status_code == 200
    body = response.json()
    assert body["entity_counts"]["accounts"] == 3
    assert body["entity_counts"]["tickets"] == 3
    assert body["entity_counts"]["operational_events"] == 5
    assert body["evidence_count"] == 19


async def test_unknown_signal_ids_return_404_not_an_incident(api_client: AsyncClient):
    """v1 defect: unknown IDs (NOPE-1) returned 200 with the hard-coded
    Meridian investigation. They must 404, naming the unknown IDs."""
    response = await api_client.post(
        "/api/v1/investigations",
        json={"signal_ids": {"ticket_id": "NOPE-1", "alert_id": "NOPE-2"}},
    )
    assert response.status_code == 404
    detail = response.json()["detail"]
    assert "NOPE-1" in detail and "NOPE-2" in detail


async def test_scenarios_resolve_to_distinct_entity_contexts(api_client: AsyncClient):
    """v1 defect: every investigation returned the same entity context.
    Scenarios A and C must resolve to their own accounts/sites/fleets."""
    a = await _investigate(api_client, SCENARIO_A)
    c = await _investigate(api_client, SCENARIO_C)

    assert a["entity_context"]["account"]["id"] == "ACC-1001"
    assert a["entity_context"]["site"]["name"] == "Portland Distribution Center"
    assert c["entity_context"]["account"]["id"] == "ACC-3001"
    assert c["entity_context"]["site"]["name"] == "Atlas Fulfillment Hub"
    assert a["entity_context"]["account"]["id"] != c["entity_context"]["account"]["id"]

    a_evidence_entities = {e["entity_id"] for e in a["evidence"]}
    c_evidence_entities = {e["entity_id"] for e in c["evidence"]}
    assert a_evidence_entities, "scenario A must retrieve evidence"
    assert c_evidence_entities, "scenario C must retrieve evidence"
    assert not (a_evidence_entities & c_evidence_entities), "evidence must not cross scenarios"


async def test_scenario_a_retrieves_real_evidence(api_client: AsyncClient):
    """v1 defect: retrieval failed on every call (invalid SearchRequest in
    prefetch) and the exception was swallowed — 200 with zero evidence."""
    a = await _investigate(api_client, SCENARIO_A)
    assert len(a["evidence"]) >= 5
    source_types = {e["source_type"] for e in a["evidence"]}
    assert "telemetry" in source_types
    assert "deployment" in source_types
    assert a["telemetry_analysis"]["anomalies_detected"], "telemetry anomalies must be found"


async def test_scenario_a_escalates_and_withholds_customer_draft(api_client: AsyncClient):
    """v1 defect: the customer draft was returned even when governance listed
    draft_customer_response as blocked. High severity + enterprise → ESCALATE,
    which does not approve a customer draft."""
    a = await _investigate(api_client, SCENARIO_A)
    gov = a["governance_decision"]

    assert gov["action_classification"] == "escalate"
    assert gov["escalation_required"] is True
    assert a["customer_response_draft"] is None
    assert "WITHHELD" in a["operator_briefing"]


async def test_weak_evidence_output_restrictions(api_client: AsyncClient):
    """v1 defect: at 20% confidence the customer draft claimed a root cause
    was identified and promised a 2-hour resolution. Weak evidence must
    restrict outputs and claim nothing."""
    b = await _investigate(api_client, SCENARIO_B)
    gov = b["governance_decision"]

    assert b["hypotheses"][0]["confidence"] < 0.3
    assert "draft_customer_response" in gov["blocked_actions"]
    assert b["customer_response_draft"] is None

    briefing = b["operator_briefing"].lower()
    for phrase in FORBIDDEN_WHEN_WEAK:
        assert phrase not in briefing, f"weak-evidence briefing claims: {phrase!r}"
    assert FORBIDDEN_ETA.search(briefing) is None


async def test_scenario_c_emits_grounded_customer_draft(api_client: AsyncClient):
    """COMMUNICATE approves the draft; it must be built from actual evidence:
    the real site name, no invented root cause phrasing, no ETA promise."""
    c = await _investigate(api_client, SCENARIO_C)
    gov = c["governance_decision"]

    assert gov["action_classification"] == "communicate"
    draft = c["customer_response_draft"]
    assert draft is not None
    assert "Atlas Fulfillment Hub" in draft
    assert "Atlas Retail Robotics" in draft
    for phrase in FORBIDDEN_WHEN_WEAK:
        assert phrase not in draft.lower()
    assert FORBIDDEN_ETA.search(draft) is None
    assert c["hypotheses"][0]["confidence"] >= 0.7


async def test_retrieval_failure_is_visible_503(api_client: AsyncClient, monkeypatch):
    """v1 defect: an unreachable Qdrant produced a successful investigation
    with zero evidence. It must surface as 503."""
    monkeypatch.setenv("QDRANT_URL", "http://127.0.0.1:9")
    # New manager per request reads settings lazily, so the env change takes
    # effect without rebuilding the app.
    response = await api_client.post("/api/v1/investigations", json={"signal_ids": SCENARIO_A})
    assert response.status_code == 503
    assert "detail" in response.json()
