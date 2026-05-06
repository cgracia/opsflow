from unittest.mock import AsyncMock, MagicMock

import pytest

from app.governance.engine import GovernanceEngine
from app.orchestrator.investigation import InvestigationManager
from app.orchestrator.phases import PHASE_ORDER
from app.schemas.investigation import (
    HistoricalReport,
    SignalIds,
    TelemetryReport,
)
from app.tracing.langfuse import LangfuseTracer


def _meridian_evidence():
    return [
        {
            "id": "EVT-EV001",
            "entity_id": "DEV-401",
            "entity_type": "device",
            "source_type": "telemetry",
            "content": "Navigation error rate spike detected on DEV-401 after deployment v3.3.0",
            "score": 0.95,
        },
        {
            "id": "HTKT-001",
            "entity_id": "ACC-1001",
            "entity_type": "account",
            "source_type": "historical_ticket",
            "content": "Navigation path planning failures after v3.1.2 update. "
            "Resolution: rollback to v3.1.1 and patch navigation config.",
            "score": 0.88,
        },
        {
            "id": "RB-001",
            "entity_id": "FLT-101",
            "entity_type": "fleet",
            "source_type": "runbook",
            "content": "Runbook: Navigation Troubleshooting. If post-deployment, consider rollback.",
            "score": 0.82,
        },
        {
            "id": "DPM-001",
            "entity_id": "DEPL-501",
            "entity_type": "deployment",
            "source_type": "deployment",
            "content": "Deployment DEPL-501: v3.2.1 -> v3.3.0. Status: HALTED.",
            "score": 0.91,
        },
    ]


def _telemetry_report():
    return TelemetryReport(
        findings=[
            "Navigation error rate anomaly detected in telemetry",
            "Temporal correlation with deployment change detected",
        ],
        anomalies_detected=[
            "navigation_error_rate: 47.3% (baseline: 0.2%)",
            "sensor_fusion_latency_ms: 234ms (baseline: 45ms)",
        ],
        event_timeline=[
            {"source": "TEL-001", "metric": "navigation_error_rate", "details": {"current": 0.473}},
        ],
        confidence=0.9,
        evidence_references=["TEL-001", "TEL-002"],
    )


def _historical_report():
    return HistoricalReport(
        similar_incidents=[
            {
                "id": "HTKT-001",
                "summary": "Navigation failures after v3.1.2 update",
                "resolution": "rollback",
            },
            {
                "id": "HTKT-003",
                "summary": "Fleet-wide navigation degradation after update",
                "resolution": "rollback",
            },
        ],
        recurring_patterns=[
            "Post-update navigation failure requiring rollback",
            "Sensor fusion latency issues recurring",
            "Customer-facing SLA impact from navigation issues",
        ],
        deployment_adjacency=[
            {
                "deployment_id": "DPM-001",
                "from_version": "3.2.1",
                "to_version": "3.3.0",
                "status": "halted",
            },
        ],
        known_issues=[
            {
                "id": "RB-001",
                "category": "navigation",
                "summary": "Navigation Troubleshooting runbook",
            },
        ],
        confidence=0.8,
        evidence_references=["HTKT-001", "HTKT-002", "HTKT-003", "RB-001", "DPM-001"],
    )


def _make_manager(trace_callback=None) -> InvestigationManager:
    qdrant = MagicMock()
    qdrant._collection_ready = True
    return InvestigationManager(
        qdrant_manager=qdrant,
        telemetry_investigator=MagicMock(),
        historical_investigator=MagicMock(),
        governance_engine=GovernanceEngine(),
        trace_callback=trace_callback,
    )


@pytest.mark.asyncio
async def test_golden_path_multi_signal_convergence():
    manager = _make_manager()

    manager._telemetry.investigate = AsyncMock(return_value=_telemetry_report())
    manager._historical.investigate = AsyncMock(return_value=_historical_report())

    signal_ids = SignalIds(ticket_id="TCK-1001", alert_id="ALT-2001", event_id="EVT-3001")

    import app.orchestrator.investigation as inv_module

    original_search = inv_module.search_evidence
    inv_module.search_evidence = AsyncMock(return_value=_meridian_evidence())
    try:
        result = await manager.run_investigation(signal_ids)
    finally:
        inv_module.search_evidence = original_search

    assert result.investigation_id.startswith("INV-")
    assert result.trace_id.startswith("trace-")

    assert result.entity_context.account is not None
    assert result.entity_context.account["name"] == "Meridian Logistics"

    assert len(result.hypotheses) >= 2
    confidences = [h.confidence for h in result.hypotheses]
    assert confidences == sorted(confidences, reverse=True), (
        "hypotheses must be sorted by confidence descending"
    )

    primary = next((h for h in result.hypotheses if h.is_primary), None)
    assert primary is not None
    assert "v3.3.0" in primary.description or "regression" in primary.description.lower()
    assert primary.severity == "high"
    assert primary.confidence >= 0.8

    gov = result.governance_decision
    assert gov is not None
    assert gov.action_classification == "escalate"
    assert len(gov.approved_actions) > 0
    assert "execute" in gov.blocked_actions
    assert gov.escalation_required is True

    assert isinstance(result.operator_briefing, str)
    assert len(result.operator_briefing) > 50

    assert isinstance(result.customer_response_draft, str)
    assert len(result.customer_response_draft) > 50

    for forbidden in ("Langfuse", "Qdrant", "LLM"):
        assert forbidden not in result.customer_response_draft

    assert result.telemetry_analysis is not None
    assert isinstance(result.telemetry_analysis, TelemetryReport)
    assert len(result.telemetry_analysis.anomalies_detected) > 0

    assert result.historical_analysis is not None
    assert isinstance(result.historical_analysis, HistoricalReport)
    assert len(result.historical_analysis.recurring_patterns) > 0
    assert len(result.historical_analysis.deployment_adjacency) > 0


@pytest.mark.asyncio
async def test_cli_seed_command_runs():
    import app.seed.__main__ as seed_main

    assert hasattr(seed_main, "main")
    assert hasattr(seed_main, "_seed")

    from app.seed.entities import seed_accounts, seed_sites, seed_software_revisions
    from app.seed.evidence import get_all_evidence

    accounts = await seed_accounts()
    assert "meridian" in accounts
    assert accounts["meridian"].name == "Meridian Logistics"

    sites = await seed_sites(accounts)
    assert "portland" in sites

    revisions = await seed_software_revisions()
    assert "v330" in revisions

    evidence = get_all_evidence()
    assert len(evidence) >= 10
    source_types = {e["source_type"] for e in evidence}
    assert "telemetry" in source_types
    assert "historical_ticket" in source_types
    assert "runbook" in source_types
    assert "deployment" in source_types


@pytest.mark.asyncio
async def test_investigation_with_tracing_integrated():
    mock_client = MagicMock()
    trace_spans = []
    mock_trace = MagicMock()

    def mock_span(**kwargs):
        s = MagicMock()
        s.id = kwargs.get("id", "span-0")
        s.name = kwargs.get("name", "")
        trace_spans.append(s)
        return s

    mock_trace.span = mock_span
    mock_client.trace = MagicMock(return_value=mock_trace)
    mock_client.flush = MagicMock()

    tracer = LangfuseTracer(client=mock_client)

    manager = _make_manager(trace_callback=tracer)
    manager._telemetry.investigate = AsyncMock(return_value=_telemetry_report())
    manager._historical.investigate = AsyncMock(return_value=_historical_report())

    import app.orchestrator.investigation as inv_module

    original_search = inv_module.search_evidence
    inv_module.search_evidence = AsyncMock(return_value=_meridian_evidence())
    try:
        result = await manager.run_investigation(
            SignalIds(ticket_id="TCK-1001", alert_id="ALT-2001", event_id="EVT-3001")
        )
    finally:
        inv_module.search_evidence = original_search

    assert result.trace_id.startswith("trace-")
    assert result.trace_id in tracer._traces
    assert len(trace_spans) == len(PHASE_ORDER) + 7
