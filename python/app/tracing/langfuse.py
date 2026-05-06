import logging
import uuid
from typing import Any

from app.config import Settings

logger = logging.getLogger(__name__)


class _NoopSpan:
    """Span that records metadata but does nothing — used when Langfuse is not configured."""

    def __init__(self, span_id: str, trace_id: str, name: str, parent_id: str | None = None):
        self.id = span_id
        self.trace_id = trace_id
        self.name = name
        self.parent_id = parent_id
        self.metadata: dict[str, Any] = {}

    def span(self, **kwargs: Any) -> "_NoopSpan":
        return _NoopSpan(
            span_id=kwargs.get("id", str(uuid.uuid4())),
            trace_id=self.trace_id,
            name=kwargs.get("name", "noop"),
        )

    def update(self, **kwargs: Any) -> "_NoopSpan":
        self.metadata.update(kwargs)
        return self

    def end(self) -> None:
        pass


class _NoopTracerCore:
    """No-op stand-in when Langfuse keys are missing."""

    def trace(self, **kwargs: Any) -> Any:
        return _NoopSpan(
            span_id=str(uuid.uuid4()),
            trace_id=kwargs.get("id", str(uuid.uuid4())),
            name=kwargs.get("name", "noop"),
        )

    def flush(self) -> None:
        pass


class LangfuseTracer:
    """Langfuse-backed tracer for investigation spans.

    Works as a drop-in ``trace_callback`` for ``InvestigationManager``.
    When Langfuse credentials are absent the tracer degrades to silent no-ops.
    """

    def __init__(self, client: Any):
        self._client = client
        self._traces: dict[str, Any] = {}
        self._spans: dict[str, Any] = {}
        self._active_trace_id: str | None = None

    def create_trace(self, name: str = "investigation", metadata: dict | None = None) -> str:
        trace_id = f"trace-{uuid.uuid4().hex[:12]}"
        trace_obj = self._client.trace(
            id=trace_id,
            name=name,
            metadata=metadata or {},
        )
        self._traces[trace_id] = trace_obj
        self._active_trace_id = trace_id
        return trace_id

    def create_span(
        self,
        trace_id: str,
        name: str,
        span_type: str = "phase",
        parent_id: str | None = None,
        metadata: dict | None = None,
        input_data: Any = None,
    ) -> Any:
        trace_obj = self._traces.get(trace_id)
        if trace_obj is None:
            logger.warning("No trace found for id=%s, span '%s' dropped", trace_id, name)
            return _NoopSpan(span_id=str(uuid.uuid4()), trace_id=trace_id, name=name)

        span_id = f"span-{uuid.uuid4().hex[:8]}"
        span_kwargs: dict[str, Any] = {
            "id": span_id,
            "name": name,
            "metadata": metadata or {},
        }
        if input_data is not None:
            span_kwargs["input"] = input_data
        if parent_id is not None:
            span_kwargs["parent_observation_id"] = parent_id

        span = trace_obj.span(**span_kwargs)
        self._spans[span_id] = span
        return span

    def end_span(self, span_id: str, output: Any = None, metadata: dict | None = None) -> None:
        span = self._spans.get(span_id)
        if span is None:
            return
        update_kwargs: dict[str, Any] = {}
        if output is not None:
            update_kwargs["output"] = output
        if metadata:
            update_kwargs["metadata"] = metadata
        if update_kwargs:
            span.update(**update_kwargs)
        span.end()

    def attach_evidence(self, span_id: str, evidence: dict | list) -> None:
        span = self._spans.get(span_id)
        if span is None:
            return
        span.update(metadata={"evidence": evidence})

    def flush(self) -> None:
        self._client.flush()

    def __call__(self, trace_id: str, phase_name: str, status: str) -> None:
        """Callback interface compatible with ``InvestigationManager._emit_span``."""
        span_id = f"span-{phase_name}-{uuid.uuid4().hex[:6]}"

        trace_obj = self._traces.get(trace_id)
        if trace_obj is None:
            return

        span = trace_obj.span(
            id=span_id,
            name=phase_name,
            metadata={"status": status},
        )
        self._spans[span_id] = span
        if status == "end":
            span.end()


def create_tracer(settings: Settings | None = None) -> LangfuseTracer:
    """Factory: build a ``LangfuseTracer`` from settings, or a no-op tracer when unconfigured."""
    if settings is None:
        settings = Settings()

    public_key = settings.langfuse_public_key
    secret_key = settings.langfuse_secret_key

    if not public_key or not secret_key:
        return LangfuseTracer(client=_NoopTracerCore())

    try:
        from langfuse import Langfuse

        client = Langfuse(
            public_key=public_key,
            secret_key=secret_key,
            host=settings.langfuse_base_url,
        )
        return LangfuseTracer(client=client)
    except Exception:
        logger.warning("Failed to initialize Langfuse client — falling back to no-op tracer")
        return LangfuseTracer(client=_NoopTracerCore())
