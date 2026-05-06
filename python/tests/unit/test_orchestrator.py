import pytest
from unittest.mock import AsyncMock, MagicMock

from app.orchestrator.investigation import InvestigationManager
from app.orchestrator.phases import InvestigationPhase, PHASE_ORDER
from app.schemas.investigation import SignalIds, TelemetryReport, HistoricalReport, EntityContext


def _make_manager() -> InvestigationManager:
    qdrant = MagicMock()
    qdrant._collection_ready = True
    return InvestigationManager(qdrant)


@pytest.mark.asyncio
async def test_orchestrator_executes_all_phases():
    manager = _make_manager()
    signal_ids = SignalIds(ticket_id="TCK-1001", alert_id="ALT-2001", event_id="EVT-3001")

    manager._telemetry = MagicMock()
    manager._telemetry.investigate = AsyncMock(
        return_value=TelemetryReport(
            findings=["Navigation error rate anomaly detected"],
            anomalies_detected=["navigation_error_rate: 47.3%"],
            confidence=0.8,
            evidence_references=["TEL-001"],
        )
    )

    manager._historical = MagicMock()
    manager._historical.investigate = AsyncMock(
        return_value=HistoricalReport(
            similar_incidents=[{"id": "HTKT-001", "summary": "Past nav issue"}],
            recurring_patterns=["Post-update navigation failure"],
            deployment_adjacency=[{"to_version": "3.3.0"}],
            confidence=0.7,
            evidence_references=["HTKT-001"],
        )
    )

    result = await manager.run_investigation(signal_ids)

    assert result.investigation_id.startswith("INV-")
    assert result.trace_id.startswith("trace-")
    assert result.entity_context is not None
    assert result.entity_context.account is not None
    assert len(result.hypotheses) > 0
    assert result.governance_decision is not None
    assert result.operator_briefing != ""
    assert result.customer_response_draft != ""


@pytest.mark.asyncio
async def test_orchestrator_returns_complete_response():
    manager = _make_manager()
    signal_ids = SignalIds(ticket_id="TCK-1001", alert_id="ALT-2001", event_id="EVT-3001")

    manager._telemetry = MagicMock()
    manager._telemetry.investigate = AsyncMock(
        return_value=TelemetryReport(
            findings=["test"],
            confidence=0.5,
            evidence_references=[],
        )
    )
    manager._historical = MagicMock()
    manager._historical.investigate = AsyncMock(
        return_value=HistoricalReport(
            similar_incidents=[],
            confidence=0.3,
            evidence_references=[],
        )
    )

    result = await manager.run_investigation(signal_ids)

    assert result.investigation_id
    assert result.trace_id
    assert result.entity_context
    assert isinstance(result.evidence, list)
    assert result.telemetry_analysis is not None
    assert result.historical_analysis is not None
    assert isinstance(result.hypotheses, list)
    assert result.governance_decision is not None
    assert result.operator_briefing
    assert result.customer_response_draft


@pytest.mark.asyncio
async def test_orchestrator_handles_partial_evidence():
    manager = _make_manager()
    signal_ids = SignalIds(ticket_id="TCK-1001")

    manager._telemetry = MagicMock()
    manager._telemetry.investigate = AsyncMock(
        return_value=TelemetryReport(
            findings=["No telemetry data found"],
            confidence=0.0,
            evidence_references=[],
        )
    )
    manager._historical = MagicMock()
    manager._historical.investigate = AsyncMock(
        return_value=HistoricalReport(
            similar_incidents=[],
            confidence=0.0,
            evidence_references=[],
        )
    )

    result = await manager.run_investigation(signal_ids)

    assert result is not None
    assert len(result.hypotheses) > 0
    assert result.hypotheses[0].confidence < 0.5


def test_phase_order_has_seven_phases():
    assert len(PHASE_ORDER) == 7
    assert PHASE_ORDER[0] == InvestigationPhase.SIGNAL_INGESTION
    assert PHASE_ORDER[-1] == InvestigationPhase.OUTPUT_GENERATION
