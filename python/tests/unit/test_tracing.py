import uuid

import pytest
from unittest.mock import AsyncMock, MagicMock

from app.config import Settings
from app.tracing import LangfuseTracer, create_tracer
from app.tracing.spans import (
    PHASE_SPANS,
)
from app.orchestrator.investigation import InvestigationManager
from app.schemas.investigation import SignalIds, TelemetryReport, HistoricalReport


def _mock_langfuse_client():
    client = MagicMock()
    trace_obj = MagicMock()

    def make_span(**kwargs):
        s = MagicMock()
        s.id = kwargs.get("id", f"span-{uuid.uuid4().hex[:8]}")
        return s

    trace_obj.span.side_effect = make_span
    client.trace.return_value = trace_obj
    return client, trace_obj, None


def _make_tracer() -> tuple[LangfuseTracer, MagicMock]:
    client, trace_obj, _ = _mock_langfuse_client()
    tracer = LangfuseTracer(client=client)
    return tracer, trace_obj


def test_tracer_creates_root_trace():
    tracer, trace_obj = _make_tracer()

    trace_id = tracer.create_trace(name="test-investigation")

    assert trace_id.startswith("trace-")
    assert len(trace_id) > 6
    tracer._client.trace.assert_called_once()
    call_kwargs = tracer._client.trace.call_args
    assert call_kwargs.kwargs["name"] == "test-investigation"
    assert trace_id in tracer._traces


def test_tracer_creates_phase_spans():
    tracer, trace_obj = _make_tracer()
    trace_id = tracer.create_trace(name="inv-1")

    for phase_name in PHASE_SPANS:
        span = tracer.create_span(trace_id=trace_id, name=phase_name)
        assert span is not None

    assert trace_obj.span.call_count == len(PHASE_SPANS)


def test_tracer_handles_missing_config():
    settings = Settings(
        langfuse_public_key="",
        langfuse_secret_key="",
    )
    tracer = create_tracer(settings)

    trace_id = tracer.create_trace(name="noop-test")
    assert trace_id.startswith("trace-")

    span = tracer.create_span(trace_id=trace_id, name="test_span")
    assert span is not None

    tracer.flush()


def test_tracer_attach_evidence_metadata():
    tracer, trace_obj = _make_tracer()
    trace_id = tracer.create_trace(name="evidence-test")
    span = tracer.create_span(trace_id=trace_id, name="evidence_phase")

    evidence = {"source": "telemetry", "anomaly_score": 0.92}
    tracer.attach_evidence(span.id, evidence=evidence)

    span.update.assert_called_with(metadata={"evidence": evidence})


def test_tracer_callable_interface():
    tracer, trace_obj = _make_tracer()
    trace_id = tracer.create_trace(name="callback-test")

    tracer(trace_id, "signal_ingestion", "start")

    trace_obj.span.assert_called_once()
    call_kwargs = trace_obj.span.call_args.kwargs
    assert call_kwargs["name"] == "signal_ingestion"
    assert call_kwargs["metadata"]["status"] == "start"


def test_tracer_callable_with_end_status_ends_span():
    tracer, trace_obj = _make_tracer()
    trace_id = tracer.create_trace(name="end-test")

    tracer(trace_id, "signal_ingestion", "end")

    assert trace_obj.span.call_count == 1
    span_id = trace_obj.span.call_args.kwargs["id"]
    tracer._spans[span_id].end.assert_called_once()


def test_tracer_callable_ignores_unknown_trace():
    tracer, _ = _make_tracer()

    tracer("nonexistent-trace-id", "signal_ingestion", "start")


def test_create_span_without_trace_is_noop():
    tracer, _ = _make_tracer()

    span = tracer.create_span(trace_id="missing", name="orphan_span")
    assert span is not None
    assert span.name == "orphan_span"


def test_end_span_updates_and_ends():
    tracer, _ = _make_tracer()
    trace_id = tracer.create_trace()
    span = tracer.create_span(trace_id=trace_id, name="test")

    tracer.end_span(span.id, output={"result": "ok"}, metadata={"extra": True})

    span.update.assert_called_once_with(
        output={"result": "ok"},
        metadata={"extra": True},
    )
    span.end.assert_called_once()


def test_end_span_unknown_id_is_noop():
    tracer, _ = _make_tracer()
    tracer.end_span("nonexistent-span-id")


@pytest.mark.asyncio
async def test_investigation_with_tracing():
    client, trace_obj, span_obj = _mock_langfuse_client()
    tracer = LangfuseTracer(client=client)

    qdrant = MagicMock()
    qdrant._collection_ready = True
    manager = InvestigationManager(qdrant, trace_callback=tracer)

    manager._telemetry = MagicMock()
    manager._telemetry.investigate = AsyncMock(
        return_value=TelemetryReport(
            findings=["anomaly"],
            anomalies_detected=["nav_error"],
            confidence=0.8,
            evidence_references=["TEL-001"],
        )
    )
    manager._historical = MagicMock()
    manager._historical.investigate = AsyncMock(
        return_value=HistoricalReport(
            similar_incidents=[],
            recurring_patterns=["pattern-a"],
            deployment_adjacency=[{"to_version": "3.3.0"}],
            confidence=0.7,
            evidence_references=["HTKT-001"],
        )
    )

    signal_ids = SignalIds(ticket_id="TCK-1001", alert_id="ALT-2001")
    result = await manager.run_investigation(signal_ids)

    assert result.trace_id.startswith("trace-")
    assert result.investigation_id.startswith("INV-")
    assert len(result.hypotheses) > 0

    client.trace.assert_called_once()
    assert trace_obj.span.call_count == 14


@pytest.mark.asyncio
async def test_investigation_without_tracing_unchanged():
    qdrant = MagicMock()
    qdrant._collection_ready = True
    manager = InvestigationManager(qdrant)

    manager._telemetry = MagicMock()
    manager._telemetry.investigate = AsyncMock(
        return_value=TelemetryReport(findings=["test"], confidence=0.5)
    )
    manager._historical = MagicMock()
    manager._historical.investigate = AsyncMock(return_value=HistoricalReport(confidence=0.3))

    signal_ids = SignalIds(ticket_id="TCK-1001")
    result = await manager.run_investigation(signal_ids)

    assert result.trace_id.startswith("trace-")
    assert result.investigation_id.startswith("INV-")


def test_span_constants_match_phases():
    expected = [
        "signal_ingestion",
        "entity_resolution",
        "evidence_retrieval",
        "specialist_investigation",
        "hypothesis_generation",
        "governance_evaluation",
        "output_generation",
    ]
    assert PHASE_SPANS == expected


@pytest.mark.asyncio
async def test_phase_spans_are_started_and_ended():
    client, trace_obj, _ = _mock_langfuse_client()
    tracer = LangfuseTracer(client=client)

    qdrant = MagicMock()
    qdrant._collection_ready = True
    manager = InvestigationManager(qdrant, trace_callback=tracer)

    manager._telemetry = MagicMock()
    manager._telemetry.investigate = AsyncMock(
        return_value=TelemetryReport(
            findings=["anomaly"],
            anomalies_detected=["nav_error"],
            confidence=0.8,
            evidence_references=["TEL-001"],
        )
    )
    manager._historical = MagicMock()
    manager._historical.investigate = AsyncMock(
        return_value=HistoricalReport(
            recurring_patterns=["pattern-a"],
            deployment_adjacency=[{"to_version": "3.3.0"}],
            confidence=0.7,
            evidence_references=["HTKT-001"],
        )
    )

    signal_ids = SignalIds(ticket_id="TCK-1001")
    result = await manager.run_investigation(signal_ids)

    assert result.trace_id in tracer._traces

    ended_spans = [s for s in tracer._spans.values() if s.end.called]
    assert len(ended_spans) == 14

    client.flush.assert_called_once()


@pytest.mark.asyncio
async def test_specialist_sub_spans_created():
    client, trace_obj, _ = _mock_langfuse_client()
    tracer = LangfuseTracer(client=client)

    qdrant = MagicMock()
    qdrant._collection_ready = True
    manager = InvestigationManager(qdrant, trace_callback=tracer)

    manager._telemetry = MagicMock()
    manager._telemetry.investigate = AsyncMock(
        return_value=TelemetryReport(findings=["test"], confidence=0.5)
    )
    manager._historical = MagicMock()
    manager._historical.investigate = AsyncMock(return_value=HistoricalReport(confidence=0.3))

    signal_ids = SignalIds(ticket_id="TCK-1001")
    await manager.run_investigation(signal_ids)

    span_names = [call.kwargs.get("name", "") for call in trace_obj.span.call_args_list]
    assert "specialist.telemetry" in span_names
    assert "specialist.historical" in span_names
