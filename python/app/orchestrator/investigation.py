import logging
import uuid
from datetime import datetime, timezone
from typing import Any

from app.schemas.investigation import (
    SignalIds,
    InvestigationResponse,
    EntityContext,
    EvidenceItem,
    Hypothesis,
    TelemetryReport,
    HistoricalReport,
    GovernanceDecision,
)
from app.orchestrator.phases import InvestigationPhase
from app.retrieval.client import QdrantManager
from app.retrieval.search import search_evidence
from app.specialists.telemetry import TelemetryInvestigator
from app.specialists.historical import HistoricalInvestigator
from app.governance.engine import GovernanceEngine
from app.llm.client import LLMClient
from app.llm.prompts import HYPOTHESIS_GENERATION
from app.tracing.langfuse import LangfuseTracer
from app.tracing.spans import (
    TELEMETRY_SPECIALIST,
    HISTORICAL_SPECIALIST,
    RETRIEVAL_HYBRID,
    LLM_HYPOTHESIS_GENERATION,
    LLM_OPERATOR_BRIEFING,
    LLM_CUSTOMER_RESPONSE,
    GOVERNANCE_EVALUATE,
)


logger = logging.getLogger(__name__)


class InvestigationManager:
    """Central orchestrator for operational investigations.

    Coordinates 7 phases: signal ingestion → entity resolution → evidence retrieval
    → specialist investigation → hypothesis generation → governance evaluation → output generation.
    """

    def __init__(
        self,
        qdrant_manager: QdrantManager,
        llm_client: LLMClient | None = None,
        telemetry_investigator: TelemetryInvestigator | None = None,
        historical_investigator: HistoricalInvestigator | None = None,
        governance_engine: GovernanceEngine | None = None,
        trace_callback=None,
    ):
        self._qdrant = qdrant_manager
        self._llm = llm_client
        self._telemetry = telemetry_investigator or TelemetryInvestigator(
            qdrant_manager, llm_client
        )
        self._historical = historical_investigator or HistoricalInvestigator(
            qdrant_manager, llm_client
        )
        self._governance = governance_engine or GovernanceEngine()
        self._trace_callback = trace_callback

    async def run_investigation(
        self,
        signal_ids: SignalIds,
        entity_map: dict | None = None,
    ) -> InvestigationResponse:
        investigation_id = f"INV-{uuid.uuid4().hex[:8]}"

        if isinstance(self._trace_callback, LangfuseTracer):
            trace_id = self._trace_callback.create_trace(
                name=f"investigation-{investigation_id}",
            )
        else:
            trace_id = f"trace-{uuid.uuid4().hex[:12]}"

        completed_phases: list[str] = []

        span_id_1 = self._start_phase_span(trace_id, InvestigationPhase.SIGNAL_INGESTION)
        signals = self._ingest_signals(signal_ids)
        completed_phases.append(InvestigationPhase.SIGNAL_INGESTION.value)
        self._end_phase_span(span_id_1)

        span_id_2 = self._start_phase_span(trace_id, InvestigationPhase.ENTITY_RESOLUTION)
        entity_context = await self._resolve_entities(signals, entity_map)
        completed_phases.append(InvestigationPhase.ENTITY_RESOLUTION.value)
        self._end_phase_span(span_id_2)

        span_id_3 = self._start_phase_span(trace_id, InvestigationPhase.EVIDENCE_RETRIEVAL)
        evidence = await self._retrieve_evidence(signals, entity_context, trace_id, span_id_3)
        completed_phases.append(InvestigationPhase.EVIDENCE_RETRIEVAL.value)
        self._end_phase_span(span_id_3)

        span_id_4 = self._start_phase_span(trace_id, InvestigationPhase.SPECIALIST_INVESTIGATION)
        telemetry_report, historical_report = await self._run_specialists(
            entity_context, signals, trace_id, span_id_4
        )
        completed_phases.append(InvestigationPhase.SPECIALIST_INVESTIGATION.value)
        self._end_phase_span(span_id_4)

        span_id_5 = self._start_phase_span(trace_id, InvestigationPhase.HYPOTHESIS_GENERATION)
        hypotheses = await self._generate_hypotheses(
            entity_context,
            evidence,
            telemetry_report,
            historical_report,
            trace_id,
            span_id_5,
        )
        completed_phases.append(InvestigationPhase.HYPOTHESIS_GENERATION.value)
        self._end_phase_span(span_id_5)

        span_id_6 = self._start_phase_span(trace_id, InvestigationPhase.GOVERNANCE_EVALUATION)
        governance = self._evaluate_governance(hypotheses, entity_context, trace_id, span_id_6)
        completed_phases.append(InvestigationPhase.GOVERNANCE_EVALUATION.value)
        self._end_phase_span(span_id_6)

        span_id_7 = self._start_phase_span(trace_id, InvestigationPhase.OUTPUT_GENERATION)
        operator_briefing, customer_response = await self._generate_outputs(
            entity_context,
            hypotheses,
            telemetry_report,
            historical_report,
            governance,
            trace_id,
            span_id_7,
        )
        completed_phases.append(InvestigationPhase.OUTPUT_GENERATION.value)
        self._end_phase_span(span_id_7)

        if isinstance(self._trace_callback, LangfuseTracer):
            self._trace_callback.flush()

        return InvestigationResponse(
            investigation_id=investigation_id,
            trace_id=trace_id,
            entity_context=entity_context,
            evidence=evidence,
            telemetry_analysis=telemetry_report,
            historical_analysis=historical_report,
            hypotheses=hypotheses,
            governance_decision=governance,
            operator_briefing=operator_briefing,
            customer_response_draft=customer_response,
            created_at=datetime.now(timezone.utc),
        )

    def _ingest_signals(self, signal_ids: SignalIds) -> dict:
        signals = {}
        if signal_ids.ticket_id:
            signals["ticket_id"] = signal_ids.ticket_id
        if signal_ids.alert_id:
            signals["alert_id"] = signal_ids.alert_id
        if signal_ids.event_id:
            signals["event_id"] = signal_ids.event_id
        return signals

    async def _resolve_entities(self, signals: dict, entity_map: dict | None) -> EntityContext:
        if entity_map:
            return EntityContext(**entity_map)
        return EntityContext(
            account={"id": "ACC-1001", "name": "Meridian Logistics", "tier": "enterprise"},
            site={"id": "SITE-2001", "name": "Portland Distribution Center"},
            fleet={"id": "FLT-101", "name": "Warehouse Alpha Fleet"},
            devices=[
                {"id": "DEV-401", "status": "error", "software": "v3.3.0"},
                {"id": "DEV-402", "status": "degraded", "software": "v3.3.0"},
                {"id": "DEV-403", "status": "degraded", "software": "v3.3.0"},
            ],
            deployment={"id": "DEPL-501", "status": "in_progress", "version": "v3.3.0"},
            software_revision={"id": "SWREV-302", "version": "v3.3.0"},
        )

    async def _retrieve_evidence(
        self,
        signals: dict,
        entity_context: EntityContext,
        trace_id: str | None = None,
        parent_span_id: str | None = None,
    ) -> list[EvidenceItem]:
        entity_ids = []
        if entity_context.account:
            entity_ids.append(entity_context.account.get("id", ""))
        if entity_context.fleet:
            entity_ids.append(entity_context.fleet.get("id", ""))
        for d in entity_context.devices:
            entity_ids.append(d.get("id", ""))

        retrieval_span_id = self._start_sub_span(trace_id, RETRIEVAL_HYBRID, parent_span_id)
        try:
            results = await search_evidence(
                query="navigation error device blocked anomaly alert",
                manager=self._qdrant,
                entity_ids=entity_ids,
                limit=15,
            )
        except Exception as e:
            logger.warning("Failed to retrieve evidence: %s", e)
            results = []
        self._end_sub_span(retrieval_span_id, output={"evidence_count": len(results)})

        return [
            EvidenceItem(
                source_type=r.get("source_type", "unknown"),
                source_id=r.get("id", ""),
                entity_id=r.get("entity_id", ""),
                entity_type=r.get("entity_type", ""),
                content=r.get("content", ""),
                relevance_score=r.get("score", 0.0),
            )
            for r in results
        ]

    async def _run_specialists(
        self,
        entity_context: EntityContext,
        signals: dict,
        trace_id: str | None = None,
        parent_span_id: str | None = None,
    ) -> tuple[TelemetryReport, HistoricalReport]:
        device_id = entity_context.devices[0]["id"] if entity_context.devices else "unknown"
        fleet_id = entity_context.fleet.get("id", "") if entity_context.fleet else ""

        tel_span_id = self._start_sub_span(trace_id, TELEMETRY_SPECIALIST, parent_span_id)
        telemetry_report = await self._telemetry.investigate(
            device_id=device_id,
            fleet_id=fleet_id,
        )
        self._end_sub_span(tel_span_id)

        hist_span_id = self._start_sub_span(trace_id, HISTORICAL_SPECIALIST, parent_span_id)
        entity_ids = [device_id, fleet_id]
        historical_report = await self._historical.investigate(
            entity_ids=entity_ids,
            entity_types=["device", "fleet"],
            symptoms="navigation error device blocked",
        )
        self._end_sub_span(hist_span_id)

        return telemetry_report, historical_report

    async def _generate_hypotheses(
        self,
        entity_context: EntityContext,
        evidence: list[EvidenceItem],
        telemetry: TelemetryReport,
        historical: HistoricalReport,
        trace_id: str | None = None,
        parent_span_id: str | None = None,
    ) -> list[Hypothesis]:
        llm_span_id = self._start_sub_span(trace_id, LLM_HYPOTHESIS_GENERATION, parent_span_id)
        if self._llm:
            try:
                result = await self._llm.generate_structured(
                    system_prompt=HYPOTHESIS_GENERATION["system"],
                    user_prompt=HYPOTHESIS_GENERATION["user"].format(
                        entity_context=str(entity_context),
                        evidence_summary=str([e.content[:100] for e in evidence[:5]]),
                        telemetry_findings=str(telemetry.findings[:3]),
                        historical_findings=str(historical.recurring_patterns[:3]),
                    ),
                )
                if "hypotheses" in result:
                    hypotheses_result = [
                        Hypothesis(
                            id=h.get("id", f"H-{i}"),
                            description=h.get("description", ""),
                            confidence=h.get("confidence", 0.5),
                            evidence_ids=h.get("evidence_ids", []),
                            severity=h.get("severity", "medium"),
                            is_primary=h.get("is_primary", i == 0),
                        )
                        for i, h in enumerate(result["hypotheses"])
                    ]
                    self._end_sub_span(
                        llm_span_id,
                        output={"hypotheses": len(hypotheses_result), "source": "llm"},
                    )
                    return hypotheses_result
            except Exception as e:
                logger.warning("Failed to generate structured hypotheses using LLM: %s", e)
                pass

        hypotheses = []
        version = (
            entity_context.software_revision.get("version", "")
            if entity_context.software_revision
            else ""
        )

        if telemetry.anomalies_detected and historical.deployment_adjacency:
            hypotheses.append(
                Hypothesis(
                    id="H-1",
                    description=f"Software version {version} introduced a regression in the navigation engine causing path planning failures across affected devices",
                    confidence=0.85,
                    evidence_ids=[e.source_id for e in evidence[:3]]
                    + telemetry.evidence_references[:2],
                    severity="high",
                    is_primary=True,
                )
            )

        if historical.recurring_patterns:
            hypotheses.append(
                Hypothesis(
                    id="H-2",
                    description=f"Recurring sensor fusion/navigation issue pattern — {len(historical.recurring_patterns)} similar past incidents detected",
                    confidence=0.65,
                    evidence_ids=historical.evidence_references[:2],
                    severity="medium",
                    is_primary=False,
                )
            )

        if not hypotheses:
            hypotheses.append(
                Hypothesis(
                    id="H-1",
                    description="Insufficient evidence for confident hypothesis — further investigation required",
                    confidence=0.2,
                    evidence_ids=[],
                    severity="low",
                    is_primary=True,
                )
            )

        self._end_sub_span(
            llm_span_id,
            output={"hypotheses": len(hypotheses), "source": "fallback"},
        )
        return hypotheses

    def _evaluate_governance(
        self,
        hypotheses: list[Hypothesis],
        entity_context: EntityContext,
        trace_id: str | None = None,
        parent_span_id: str | None = None,
    ) -> GovernanceDecision:
        primary = next(
            (h for h in hypotheses if h.is_primary), hypotheses[0] if hypotheses else None
        )
        severity = primary.severity if primary else "medium"
        confidence = primary.confidence if primary else 0.5

        account_tier = (
            entity_context.account.get("tier", "standard") if entity_context.account else "standard"
        )
        sensitivity = "vip_customer" if account_tier == "enterprise" else "internal_only"

        gov_span_id = self._start_sub_span(trace_id, GOVERNANCE_EVALUATE, parent_span_id)
        decision = self._governance.evaluate(
            severity=severity,
            customer_sensitivity=sensitivity,
            evidence_confidence=confidence,
        )
        self._end_sub_span(
            gov_span_id,
            output={
                "action": decision.action_classification.value,
                "escalation": decision.escalation_required,
            },
        )
        return decision

    async def _generate_outputs(
        self,
        entity_context: EntityContext,
        hypotheses: list[Hypothesis],
        telemetry: TelemetryReport,
        historical: HistoricalReport,
        governance: GovernanceDecision,
        trace_id: str | None = None,
        parent_span_id: str | None = None,
    ) -> tuple[str, str]:
        account_name = (
            entity_context.account.get("name", "Unknown") if entity_context.account else "Unknown"
        )
        primary = next(
            (h for h in hypotheses if h.is_primary), hypotheses[0] if hypotheses else None
        )

        briefing_span_id = self._start_sub_span(
            trace_id,
            LLM_OPERATOR_BRIEFING,
            parent_span_id,
        )
        operator_briefing = (
            "INVESTIGATION BRIEFING\n"
            "==================================================\n"
            f"Investigation for: {account_name}\n"
            "==================================================\n\n"
            "ENTITY CONTEXT:\n"
            f"  Account: {entity_context.account}\n"
            f"  Fleet: {entity_context.fleet}\n"
            f"  Affected devices: {len(entity_context.devices)}\n\n"
            "PRIMARY HYPOTHESIS:\n"
            f"  {primary.description if primary else 'No hypothesis'}\n"
            f"  Confidence: {primary.confidence if primary else 0:.0%}\n"
            f"  Severity: {primary.severity if primary else 'unknown'}\n\n"
            "TELEMETRY FINDINGS:\n"
            + "\n".join(f"  - {f}" for f in telemetry.findings[:5])
            + "\n\nHISTORICAL PATTERNS:\n"
            + "\n".join(f"  - {p}" for p in historical.recurring_patterns[:3])
            + "\n\nGOVERNANCE:\n"
            f"  Action: {governance.action_classification.value}\n"
            f"  Escalation: {'YES' if governance.escalation_required else 'NO'}\n"
            f"  {governance.reasoning}\n"
        )
        self._end_sub_span(
            briefing_span_id,
            output={"length": len(operator_briefing)},
        )

        response_span_id = self._start_sub_span(
            trace_id,
            LLM_CUSTOMER_RESPONSE,
            parent_span_id,
        )
        customer_response = (
            f"Dear {account_name} Team,\n\n"
            "We are aware of the issue affecting your devices at the Portland Distribution Center. "
            "Our team has identified the root cause and is actively working on a resolution.\n\n"
            "What we know: A recent software update has caused unexpected behavior in a subset of your devices. "
            "We have halted the update and are preparing a fix.\n\n"
            f"Current status: {len(entity_context.devices)} devices are impacted. "
            "Your remaining devices continue to operate normally.\n\n"
            "Next steps: We expect to have a resolution within the next 2 hours and will provide "
            "an update at that time.\n\n"
            "Please don't hesitate to reach out if you have any questions.\n\n"
            "Best regards,\nOperations Team"
        )
        self._end_sub_span(
            response_span_id,
            output={"length": len(customer_response)},
        )

        return operator_briefing, customer_response

    def _start_phase_span(self, trace_id: str, phase: InvestigationPhase) -> str | None:
        if isinstance(self._trace_callback, LangfuseTracer):
            span = self._trace_callback.create_span(
                trace_id=trace_id,
                name=phase.value,
                span_type="phase",
                metadata={"status": "start"},
            )
            return span.id
        elif self._trace_callback:
            self._trace_callback(trace_id, phase.value, "start")
        return None

    def _end_phase_span(self, span_id: str | None, output: Any = None) -> None:
        if isinstance(self._trace_callback, LangfuseTracer) and span_id is not None:
            self._trace_callback.end_span(span_id, output=output)

    def _start_sub_span(
        self,
        trace_id: str | None,
        name: str,
        parent_id: str | None = None,
    ) -> str | None:
        if isinstance(self._trace_callback, LangfuseTracer) and trace_id is not None:
            span = self._trace_callback.create_span(
                trace_id=trace_id,
                name=name,
                span_type="tool",
                parent_id=parent_id,
            )
            return span.id
        return None

    def _end_sub_span(self, span_id: str | None, output: Any = None) -> None:
        if isinstance(self._trace_callback, LangfuseTracer) and span_id is not None:
            self._trace_callback.end_span(span_id, output=output)
