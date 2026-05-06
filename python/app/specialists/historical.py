import logging
from datetime import datetime

from app.schemas.investigation import HistoricalReport
from app.retrieval.client import QdrantManager
from app.retrieval.search import search_evidence

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
        """Analyze historical incidents for the given entities.

        Retrieves past tickets, runbooks, and deployment records from Qdrant,
        identifies recurring patterns and deployment adjacency.
        """
        # Search for historical tickets and runbooks
        try:
            ticket_results = await search_evidence(
                query=f"historical navigation error failure pattern {symptoms}",
                manager=self._qdrant,
                entity_ids=entity_ids,
                source_types=["historical_ticket"],
                limit=10,
            )
        except Exception as e:
            logger.warning("Failed to retrieve historical tickets: %s", e)
            ticket_results = []

        try:
            runbook_results = await search_evidence(
                query=f"runbook troubleshooting {symptoms}",
                manager=self._qdrant,
                entity_ids=entity_ids,
                source_types=["runbook"],
                limit=5,
            )
        except Exception as e:
            logger.warning("Failed to retrieve runbook data: %s", e)
            runbook_results = []

        try:
            deployment_results = await search_evidence(
                query="deployment manifest software version",
                manager=self._qdrant,
                entity_ids=entity_ids,
                source_types=["deployment"],
                limit=5,
            )
        except Exception as e:
            logger.warning("Failed to retrieve deployment data: %s", e)
            deployment_results = []

        all_results = ticket_results + runbook_results + deployment_results
        evidence_refs = [r.get("id", "") for r in all_results]

        # Analyze patterns
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

            # Pattern detection
            if "navigation" in content and ("error" in content or "failure" in content):
                if "post_update" not in content and "rollback" in content:
                    recurring_patterns.append("Post-update navigation failure requiring rollback")

            if "sensor fusion" in content or "latency" in content:
                recurring_patterns.append("Sensor fusion latency issues recurring")

            if "sla" in content or "shipment" in content or "delivery" in content:
                recurring_patterns.append("Customer-facing SLA impact from navigation issues")

        for result in runbook_results:
            content = result.get("content", "").lower()
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
            if metadata.get("status") == "halted":
                deployment_adjacency.append(
                    {
                        "deployment_id": result.get("id"),
                        "from_version": metadata.get("version_from"),
                        "to_version": metadata.get("version_to"),
                        "status": metadata.get("status"),
                        "pattern": "Issues detected during deployment, deployment halted",
                    }
                )

        # Calculate confidence
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
