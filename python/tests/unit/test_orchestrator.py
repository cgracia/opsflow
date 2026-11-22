from unittest.mock import AsyncMock, MagicMock

import pytest

from app.orchestrator.investigation import InvestigationManager
from app.orchestrator.phases import PHASE_ORDER, InvestigationPhase
from app.schemas.investigation import HistoricalReport, SignalIds, TelemetryReport


def _entity_map(tier: str = "enterprise") -> dict:
    return {
        "account": {"id": "ACC-1001", "name": "Meridian Logistics", "tier": tier},
        "site": {"id": "SITE-2001", "name": "Portland Distribution Center"},
        "fleet": {"id": "FLT-101", "name": "Warehouse Alpha Fleet"},
        "devices": [
            {"id": "DEV-401", "status": "error", "software": "3.3.0"},
            {"id": "DEV-402", "status": "degraded", "software": "3.3.0"},
        ],
        "deployment": {"id": "DEPL-501", "status": "in_progress", "version": "3.3.0"},
        "software_revision": {"id": "SWREV-302", "version": "3.3.0"},
    }


def _make_manager() -> InvestigationManager:
    qdrant = MagicMock()
    qdrant._collection_ready = True
    qdrant._fake_vector = lambda query: [0.0] * 384
    qdrant.client.query_points.return_value = MagicMock(points=[])
    return InvestigationManager(qdrant)


def _telemetry_mock(confidence: float = 0.8, anomalies: list[str] | None = None):
    m = MagicMock()
    m.investigate = AsyncMock(
        return_value=TelemetryReport(
            findings=["Navigation error rate anomaly detected"],
            anomalies_detected=anomalies
            if anomalies is not None
            else ["navigation_error_rate: 47.3% (baseline: 0.2%)"],
            confidence=confidence,
            evidence_references=["TEL-001"],
        )
    )
    return m


def _historical_mock(
    confidence: float = 0.7,
    adjacency: list[dict] | None = None,
    patterns: list[str] | None = None,
):
    m = MagicMock()
    m.investigate = AsyncMock(
        return_value=HistoricalReport(
            similar_incidents=[{"id": "HTKT-001", "summary": "Past issue"}],
            recurring_patterns=patterns
            if patterns is not None
            else ["Post-update failure requiring rollback"],
            deployment_adjacency=adjacency
            if adjacency is not None
            else [{"to_version": "3.3.0", "status": "halted"}],
            confidence=confidence,
            evidence_references=["HTKT-001"],
        )
    )
    return m


@pytest.mark.asyncio
async def test_orchestrator_executes_all_phases():
    manager = _make_manager()
    manager._telemetry = _telemetry_mock()
    manager._historical = _historical_mock()

    result = await manager.run_investigation(
        SignalIds(ticket_id="TCK-1001", alert_id="ALT-2001", event_id="EVT-3001"),
        entity_map=_entity_map(),
    )

    assert result.investigation_id.startswith("INV-")
    assert result.trace_id.startswith("trace-")
    assert result.entity_context.account["name"] == "Meridian Logistics"
    assert len(result.hypotheses) > 0
    assert result.governance_decision is not None
    assert result.operator_briefing != ""
    # Enterprise + medium severity + strong evidence → COMMUNICATE approves
    # the customer draft.
    assert result.customer_response_draft is not None


@pytest.mark.asyncio
async def test_weak_evidence_withholds_customer_draft():
    manager = _make_manager()
    manager._telemetry = _telemetry_mock(confidence=0.0, anomalies=[])
    manager._historical = _historical_mock(confidence=0.0, adjacency=[], patterns=[])

    result = await manager.run_investigation(
        SignalIds(ticket_id="TCK-1001"),
        entity_map=_entity_map(),
    )

    assert len(result.hypotheses) == 1
    assert result.hypotheses[0].confidence < 0.3
    # Low confidence → governance blocks draft outputs entirely.
    assert "draft_customer_response" in result.governance_decision.blocked_actions
    assert result.customer_response_draft is None
    assert "WITHHELD" in result.operator_briefing


@pytest.mark.asyncio
async def test_blocked_customer_draft_is_never_returned():
    """Explicitly blocked output must never appear in the response."""
    manager = _make_manager()
    manager._telemetry = _telemetry_mock()
    manager._historical = _historical_mock()

    decision = result = None
    engine = manager._governance
    original = engine.evaluate

    def evaluate_with_block(**kwargs):
        nonlocal decision
        decision = original(**kwargs)
        decision.blocked_actions = [*list(decision.blocked_actions), "draft_customer_response"]
        return decision

    engine.evaluate = evaluate_with_block

    result = await manager.run_investigation(
        SignalIds(ticket_id="TCK-1001"), entity_map=_entity_map()
    )
    assert "draft_customer_response" in result.governance_decision.blocked_actions
    assert result.customer_response_draft is None


@pytest.mark.asyncio
async def test_retrieval_failure_propagates():
    """A broken Qdrant must fail the investigation, not yield empty evidence."""
    manager = _make_manager()
    manager._qdrant.client.query_points.side_effect = RuntimeError("connection refused")

    with pytest.raises(RuntimeError, match="connection refused"):
        await manager.run_investigation(SignalIds(ticket_id="TCK-1001"), entity_map=_entity_map())


@pytest.mark.asyncio
async def test_resolution_requires_session_or_entity_map():
    manager = _make_manager()
    with pytest.raises(ValueError, match="database session"):
        await manager.run_investigation(SignalIds(ticket_id="TCK-1001"))


def test_phase_order_has_seven_phases():
    assert len(PHASE_ORDER) == 7
    assert PHASE_ORDER[0] == InvestigationPhase.SIGNAL_INGESTION
    assert PHASE_ORDER[-1] == InvestigationPhase.OUTPUT_GENERATION
