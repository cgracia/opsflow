import pytest
from unittest.mock import AsyncMock, MagicMock
from datetime import datetime, timezone

import app.specialists.telemetry as telemetry_mod
from app.specialists.telemetry import TelemetryInvestigator
from app.schemas.investigation import TelemetryReport


def _make_qdrant_with_results(results: list[dict]) -> MagicMock:
    manager = MagicMock()
    manager._collection_ready = True
    # search_evidence is an async function that takes manager as arg
    return manager


@pytest.mark.asyncio
async def test_identifies_navigation_error_spike():
    manager = MagicMock()
    manager._collection_ready = True

    mock_results = [
        {
            "id": "TEL-001",
            "entity_id": "DEV-401",
            "source_type": "telemetry",
            "content": (
                "Telemetry snapshot DEV-401:\n"
                "- navigation_error_rate: 47.3% (baseline: 0.2%)\n"
                "- sensor_fusion_latency_ms: 234ms\n"
                "CORRELATION: Error onset directly correlates with v3.3.0"
            ),
            "metadata": {"metric": "navigation_error_rate", "baseline": 0.002, "current": 0.473},
        },
    ]

    # Patch search_evidence to return our mock results
    async def mock_search(*args, **kwargs):
        return mock_results

    investigator = TelemetryInvestigator(manager)

    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(telemetry_mod, "search_evidence", mock_search)
        report = await investigator.investigate(
            device_id="DEV-401",
            fleet_id="FLT-101",
            time_window=(
                datetime(2026, 5, 6, 10, 0, tzinfo=timezone.utc),
                datetime(2026, 5, 6, 14, 0, tzinfo=timezone.utc),
            ),
        )

    assert isinstance(report, TelemetryReport)
    assert len(report.anomalies_detected) > 0
    assert any("navigation_error_rate" in a for a in report.anomalies_detected)
    assert report.confidence > 0.0
    assert "TEL-001" in report.evidence_references


@pytest.mark.asyncio
async def test_returns_empty_report_for_healthy_device():
    manager = MagicMock()

    async def mock_search(*args, **kwargs):
        return []

    investigator = TelemetryInvestigator(manager)

    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(telemetry_mod, "search_evidence", mock_search)
        report = await investigator.investigate(
            device_id="DEV-404",
            fleet_id="FLT-101",
        )

    assert isinstance(report, TelemetryReport)
    assert report.confidence == 0.0
    assert len(report.anomalies_detected) == 0
    assert "No telemetry data" in report.findings[0]
