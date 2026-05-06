import logging
from datetime import datetime

from app.schemas.investigation import TelemetryReport
from app.retrieval.client import QdrantManager
from app.retrieval.search import search_evidence

logger = logging.getLogger(__name__)


class TelemetryInvestigator:
    """Specialist tool for analyzing device/fleet telemetry during investigations."""

    def __init__(self, qdrant_manager: QdrantManager, llm_client=None):
        self._qdrant = qdrant_manager
        self._llm = llm_client  # Optional — can work without LLM using rule-based analysis

    async def investigate(
        self,
        device_id: str,
        fleet_id: str,
        time_window: tuple[datetime, datetime] | None = None,
    ) -> TelemetryReport:
        """Analyze telemetry for a device/fleet within a time window.

        Retrieves telemetry snapshots from Qdrant, analyzes patterns,
        and returns a structured TelemetryReport.
        """
        # Retrieve telemetry evidence from Qdrant
        entity_ids = [device_id, fleet_id]
        try:
            results = await search_evidence(
                query=f"telemetry navigation error rate anomaly device {device_id}",
                manager=self._qdrant,
                entity_ids=entity_ids,
                source_types=["telemetry"],
                limit=10,
            )
        except Exception as e:
            logger.warning("Failed to retrieve telemetry evidence: %s", e)
            results = []

        # Extract findings from evidence
        findings = []
        anomalies = []
        event_timeline = []
        evidence_refs = []
        max_confidence = 0.0

        for result in results:
            content = result.get("content", "")
            evidence_refs.append(result.get("id", ""))

            # Rule-based analysis (works without LLM)
            if "navigation_error_rate" in content.lower():
                # Extract the error rate value
                for line in content.split("\n"):
                    if "navigation_error_rate" in line.lower():
                        anomalies.append(line.strip())
                        findings.append("Navigation error rate anomaly detected in telemetry")

            if "sensor_fusion_latency" in content.lower():
                for line in content.split("\n"):
                    if "sensor_fusion_latency" in line.lower():
                        anomalies.append(line.strip())

            if "correlation" in content.lower() or "coincid" in content.lower():
                findings.append("Temporal correlation with deployment change detected")

            # Build event timeline from metadata
            metadata = result.get("metadata", {})
            if metadata:
                event_timeline.append(
                    {
                        "source": result.get("id"),
                        "metric": metadata.get("metric", "unknown"),
                        "details": metadata,
                    }
                )

            # Calculate confidence based on evidence strength
            if anomalies:
                max_confidence = min(0.95, max_confidence + 0.3)
            if findings:
                max_confidence = min(0.95, max_confidence + 0.2)

        # If no evidence found, return empty report
        if not results:
            return TelemetryReport(
                findings=["No telemetry data found for the specified device/fleet."],
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
