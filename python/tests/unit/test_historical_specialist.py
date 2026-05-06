import pytest
from datetime import datetime, timezone

from app.specialists import historical as historical_mod
from app.specialists.historical import HistoricalInvestigator
from app.schemas.investigation import HistoricalReport


@pytest.mark.asyncio
async def test_finds_historical_navigation_issue():
    manager = None  # We mock search_evidence so no real manager needed

    ticket_results = [
        {
            "id": "HTKT-001",
            "entity_id": "ACC-1001",
            "source_type": "historical_ticket",
            "content": "Navigation path planning failures after v3.1.2 update. Rollback to v3.1.1 resolved.",
            "metadata": {"version": "3.1.2", "resolution": "rollback"},
        },
    ]
    runbook_results = [
        {
            "id": "RB-001",
            "entity_id": "FLT-101",
            "source_type": "runbook",
            "content": "Runbook: Navigation Troubleshooting. Check for recent deployments.",
            "metadata": {"category": "navigation"},
        },
    ]
    deployment_results = [
        {
            "id": "DPM-001",
            "entity_id": "DEPL-501",
            "source_type": "deployment",
            "content": "Deployment DEPL-501 v3.3.0 halted due to errors",
            "metadata": {"version_from": "3.2.1", "version_to": "3.3.0", "status": "halted"},
        },
    ]

    call_count = [0]

    async def mock_search(*args, **kwargs):
        source = kwargs.get("source_types", [])
        if "historical_ticket" in source:
            return ticket_results
        if "runbook" in source:
            return runbook_results
        if "deployment" in source:
            return deployment_results
        return []

    investigator = HistoricalInvestigator(manager)

    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(historical_mod, "search_evidence", mock_search)
        report = await investigator.investigate(
            entity_ids=["DEV-401", "FLT-101"],
            entity_types=["device", "fleet"],
            symptoms="navigation error device blocked",
        )

    assert isinstance(report, HistoricalReport)
    assert len(report.similar_incidents) == 1
    assert report.similar_incidents[0]["id"] == "HTKT-001"
    assert len(report.known_issues) == 1
    assert report.known_issues[0]["category"] == "navigation"
    assert len(report.deployment_adjacency) == 1
    assert report.deployment_adjacency[0]["to_version"] == "3.3.0"
    assert report.confidence > 0.0
    assert "HTKT-001" in report.evidence_references
    assert "RB-001" in report.evidence_references


@pytest.mark.asyncio
async def test_handles_entity_with_no_history():
    manager = None

    async def mock_search(*args, **kwargs):
        return []

    investigator = HistoricalInvestigator(manager)

    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(historical_mod, "search_evidence", mock_search)
        report = await investigator.investigate(
            entity_ids=["DEV-900"],
            entity_types=["device"],
        )

    assert isinstance(report, HistoricalReport)
    assert report.confidence == 0.0
    assert len(report.similar_incidents) == 0
    assert len(report.recurring_patterns) == 0
