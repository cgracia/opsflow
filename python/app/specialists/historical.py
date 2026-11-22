"""Historical incident specialist: rule-based pattern analysis.

Queries Qdrant for historical tickets, runbooks and deployment records scoped
to the resolved entities. Pattern rules are generic (post-update rollback,
sensor/latency issues, customer-facing impact) rather than navigation-
specific. Retrieval failures propagate — a broken Qdrant must not be reported
as "no historical data".
"""

import logging
from datetime import datetime

from app.retrieval.client import QdrantManager
from app.retrieval.search import search_evidence
from app.schemas.investigation import HistoricalReport

logger = logging.getLogger(__name__)


class HistoricalInvestigator:
    """Specialist tool for analyzing historical incidents and patterns."""

    def __init__(self, qdrant_manager: QdrantManager, llm_client=None):
        self._qdrant = qdrant_manager
        self._llm = llm_client

    async def investigate(
        self,
        entity_ids: list[str],
        entity_types: list[str],
        time_window: tuple[datetime, datetime] | None = None,
        symptoms: str = "",
    ) -> HistoricalReport:
        """Analyze historical incidents for the given entities."""
        ticket_results = await search_evidence(
            query=f"historical incident failure pattern {symptoms}"[:500],
            manager=self._qdrant,
            entity_ids=entity_ids,
            source_types=["historical_ticket"],
            limit=10,
        )

        runbook_results = await search_evidence(
            query=f"runbook troubleshooting {symptoms}"[:500],
            manager=self._qdrant,
            entity_ids=entity_ids,
            source_types=["runbook"],
            limit=5,
        )

        deployment_results = await search_evidence(
            query=f"deployment manifest software version {symptoms}"[:500],
            manager=self._qdrant,
            entity_ids=entity_ids,
            source_types=["deployment"],
            limit=5,
        )

        all_results = ticket_results + runbook_results + deployment_results
        evidence_refs = [r.get("id", "") for r in all_results]

        similar_incidents = []
        recurring_patterns = []
        deployment_adjacency = []
        known_issues = []

        for result in ticket_results:
            content = result.get("content", "").lower()
            similar_incidents.append(
                {
                    "id": result.get("id"),
                    "summary": result.get("content", "")[:200],
                    "resolution": result.get("metadata", {}).get("resolution", "unknown"),
                }
            )

            if ("update" in content or "deployment" in content) and "rollback" in content:
                recurring_patterns.append("Post-update failure requiring rollback")

            if "sensor fusion" in content or "latency" in content:
                recurring_patterns.append("Sensor/latency issues recurring")

            if "sla" in content or "shipment" in content or "delivery" in content:
                recurring_patterns.append("Customer-facing impact from similar incidents")

        for result in runbook_results:
            category = result.get("metadata", {}).get("category", "general")
            known_issues.append(
                {
                    "id": result.get("id"),
                    "category": category,
                    "summary": result.get("content", "")[:200],
                }
            )

        for result in deployment_results:
            metadata = result.get("metadata", {})
            if metadata.get("status") in ("halted", "in_progress"):
                deployment_adjacency.append(
                    {
                        "deployment_id": result.get("id"),
                        "from_version": metadata.get("version_from"),
                        "to_version": metadata.get("version_to"),
                        "status": metadata.get("status"),
                        "pattern": "Incident coincides with an active/recent deployment",
                    }
                )

        confidence = 0.0
        if similar_incidents:
            confidence += 0.2
        if recurring_patterns:
            confidence += 0.2
        if deployment_adjacency:
            confidence += 0.3
        if known_issues:
            confidence += 0.1
        confidence = min(0.95, confidence)

        if not all_results:
            return HistoricalReport(
                similar_incidents=[],
                recurring_patterns=[],
                deployment_adjacency=[],
                known_issues=[],
                confidence=0.0,
                evidence_references=[],
            )

        return HistoricalReport(
            similar_incidents=similar_incidents,
            recurring_patterns=list(set(recurring_patterns)),
            deployment_adjacency=deployment_adjacency,
            known_issues=known_issues,
            confidence=confidence,
            evidence_references=evidence_refs,
        )
