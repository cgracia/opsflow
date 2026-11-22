"""Telemetry specialist: rule-based analysis of telemetry evidence.

Queries Qdrant for telemetry snapshots scoped to the resolved entities and
extracts baseline-deviation anomalies generically (any ``metric: value
(baseline: ...)`` line), so the specialist is not bound to one scenario's
metric names. Retrieval failures propagate — a broken Qdrant must not be
reported as "no telemetry data found".
"""

import logging
import re
from datetime import datetime

from app.retrieval.client import QdrantManager
from app.retrieval.search import search_evidence
from app.schemas.investigation import TelemetryReport

logger = logging.getLogger(__name__)

# Lines like: "- navigation_error_rate: 47.3% (baseline: 0.2%, threshold: 5%)"
BASELINE_LINE_RE = re.compile(
    r"^\s*-\s*(?P<metric>[a-z0-9_]+):\s+(?P<value>[0-9.]+%?)[^)]*\(baseline",
    re.IGNORECASE | re.MULTILINE,
)


class TelemetryInvestigator:
    """Specialist tool for analyzing device/fleet telemetry during investigations."""

    def __init__(self, qdrant_manager: QdrantManager, llm_client=None):
        self._qdrant = qdrant_manager
        self._llm = llm_client  # Optional — can work without LLM using rule-based analysis

    async def investigate(
        self,
        device_id: str,
        fleet_id: str,
        symptoms: str = "",
        time_window: tuple[datetime, datetime] | None = None,
    ) -> TelemetryReport:
        """Analyze telemetry for a device/fleet within a time window."""
        query = f"telemetry anomaly {device_id} {symptoms}"[:500]
        results = await search_evidence(
            query=query,
            manager=self._qdrant,
            entity_ids=[e for e in (device_id, fleet_id) if e],
            source_types=["telemetry"],
            limit=10,
        )

        findings: list[str] = []
        anomalies: list[str] = []
        event_timeline: list[dict] = []
        evidence_refs: list[str] = []
        max_confidence = 0.0

        for result in results:
            content = result.get("content", "")
            evidence_refs.append(result.get("id", ""))

            # Rule-based analysis (works without LLM): any metric that
            # deviates from its stated baseline is an anomaly.
            for match in BASELINE_LINE_RE.finditer(content):
                metric = match.group("metric")
                line = match.group(0).strip().lstrip("- ").strip()
                anomalies.append(line)
                findings.append(f"Metric '{metric}' deviates from its baseline")

            if "correlation" in content.lower() or "coincid" in content.lower():
                findings.append("Temporal correlation with deployment change detected")

            metadata = result.get("metadata", {})
            if metadata:
                event_timeline.append(
                    {
                        "source": result.get("id"),
                        "metric": metadata.get("metric", "unknown"),
                        "details": metadata,
                    }
                )

            if anomalies:
                max_confidence = min(0.95, max_confidence + 0.3)
            if findings:
                max_confidence = min(0.95, max_confidence + 0.2)

        if not results:
            return TelemetryReport(
                findings=[f"No telemetry data found for {device_id}/{fleet_id}."],
                anomalies_detected=[],
                event_timeline=[],
                confidence=0.0,
                evidence_references=[],
            )

        return TelemetryReport(
            findings=findings or ["Telemetry data retrieved, no significant anomalies detected."],
            anomalies_detected=anomalies,
            event_timeline=event_timeline,
            confidence=max_confidence,
            evidence_references=evidence_refs,
        )
