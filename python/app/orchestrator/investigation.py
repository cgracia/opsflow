import logging
import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy.ext.asyncio import AsyncSession

from app.governance.engine import GovernanceEngine
from app.llm.client import LLMClient
from app.llm.prompts import HYPOTHESIS_GENERATION
from app.orchestrator.outputs import (
    build_customer_draft,
    build_operator_briefing,
    customer_draft_allowed,
)
from app.orchestrator.phases import InvestigationPhase
from app.orchestrator.resolution import (
    ResolutionResult,
    resolve_from_signals,
)
from app.retrieval.client import QdrantManager
from app.retrieval.search import search_evidence
from app.schemas.investigation import (
    EntityContext,
    EvidenceItem,
    GovernanceDecision,
    HistoricalReport,
    Hypothesis,
    InvestigationResponse,
    SignalIds,
    TelemetryReport,
)
from app.specialists.historical import HistoricalInvestigator
from app.specialists.telemetry import TelemetryInvestigator
from app.tracing.langfuse import LangfuseTracer
from app.tracing.spans import (
    GOVERNANCE_EVALUATE,
    HISTORICAL_SPECIALIST,
    LLM_CUSTOMER_RESPONSE,
    LLM_HYPOTHESIS_GENERATION,
    LLM_OPERATOR_BRIEFING,
    RETRIEVAL_HYBRID,
    TELEMETRY_SPECIALIST,
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
        db_session: AsyncSession | None = None,
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
        self._ingest_signals(signal_ids)
        completed_phases.append(InvestigationPhase.SIGNAL_INGESTION.value)
        self._end_phase_span(span_id_1)

        span_id_2 = self._start_phase_span(trace_id, InvestigationPhase.ENTITY_RESOLUTION)
        resolution = await self._resolve_entities(signal_ids, db_session, entity_map)
        entity_context = resolution.entity_context
        completed_phases.append(InvestigationPhase.ENTITY_RESOLUTION.value)
        self._end_phase_span(span_id_2)

        span_id_3 = self._start_phase_span(trace_id, InvestigationPhase.EVIDENCE_RETRIEVAL)
        evidence = await self._retrieve_evidence(
            resolution.query_text, entity_context, trace_id, span_id_3
        )
        completed_phases.append(InvestigationPhase.EVIDENCE_RETRIEVAL.value)
        self._end_phase_span(span_id_3)

        span_id_4 = self._start_phase_span(trace_id, InvestigationPhase.SPECIALIST_INVESTIGATION)
        telemetry_report, historical_report = await self._run_specialists(
            entity_context, resolution.query_text, trace_id, span_id_4
        )
        completed_phases.append(InvestigationPhase.SPECIALIST_INVESTIGATION.value)
        self._end_phase_span(span_id_4)

        span_id_5 = self._start_phase_span(trace_id, InvestigationPhase.HYPOTHESIS_GENERATION)
        hypotheses = await self._generate_hypotheses(
            entity_context,
            evidence,
            telemetry_report,
            historical_report,
            severity=resolution.severity,
            trace_id=trace_id,
            parent_span_id=span_id_5,
        )
        completed_phases.append(InvestigationPhase.HYPOTHESIS_GENERATION.value)
        self._end_phase_span(span_id_5)

        span_id_6 = self._start_phase_span(trace_id, InvestigationPhase.GOVERNANCE_EVALUATION)
        governance = self._evaluate_governance(hypotheses, entity_context, trace_id, span_id_6)
        completed_phases.append(InvestigationPhase.GOVERNANCE_EVALUATION.value)
        self._end_phase_span(span_id_6)

        span_id_7 = self._start_phase_span(trace_id, InvestigationPhase.OUTPUT_GENERATION)
        operator_briefing, customer_response = self._generate_outputs(
            entity_context,
            hypotheses,
            telemetry_report,
            historical_report,
            governance,
            resolution.site_name,
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
            created_at=datetime.now(UTC),
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

    async def _resolve_entities(
        self,
        signal_ids: SignalIds,
        db_session: AsyncSession | None,
        entity_map: dict | None,
    ) -> ResolutionResult:
        """Resolve signals to their entity graph.

        v1 returned a hard-coded entity context here, so unknown signal IDs
        produced an investigation of an unrelated incident. Resolution now
        comes from the database and unknown IDs raise. ``entity_map`` remains
        solely as an explicit injection point for tests.
        """
        if entity_map is not None:
            # Explicit injection point for tests. Underscore keys are
            # test-only knobs consumed here, never part of EntityContext.
            injected = dict(entity_map)
            severity = injected.pop("_severity", "medium")
            query_text = injected.pop("_query_text", "")
            site_name = injected.pop("_site_name", "")
            account_name = injected.get("account", {}).get("name", "")
            return ResolutionResult(
                entity_context=EntityContext(**injected),
                account_name=account_name,
                site_name=site_name,
                severity=severity,
                query_text=query_text,
            )
        if db_session is None:
            raise ValueError(
                "Entity resolution requires a database session or an explicit entity_map"
            )
        return await resolve_from_signals(db_session, signal_ids)

    def _context_entity_ids(self, entity_context: EntityContext) -> list[str]:
        """Entity IDs used to filter retrieval to the resolved graph."""
        entity_ids: list[str] = []
        if entity_context.account:
            entity_ids.append(entity_context.account.get("id", ""))
        if entity_context.site:
            entity_ids.append(entity_context.site.get("id", ""))
        if entity_context.fleet:
            entity_ids.append(entity_context.fleet.get("id", ""))
        for d in entity_context.devices:
            entity_ids.append(d.get("id", ""))
        if entity_context.deployment:
            entity_ids.append(entity_context.deployment.get("id", ""))
        return [e for e in entity_ids if e]

    async def _retrieve_evidence(
        self,
        query_text: str,
        entity_context: EntityContext,
        trace_id: str | None = None,
        parent_span_id: str | None = None,
    ) -> list[EvidenceItem]:
        """Retrieve evidence for the resolved entities.

        Retrieval failures propagate: a broken Qdrant must surface as a failed
        investigation, never as a successful one with zero evidence.
        """
        entity_ids = self._context_entity_ids(entity_context)
        query = query_text or " ".join(entity_ids)

        retrieval_span_id = self._start_sub_span(trace_id, RETRIEVAL_HYBRID, parent_span_id)
        results = await search_evidence(
            query=query,
            manager=self._qdrant,
            entity_ids=entity_ids,
            limit=15,
        )
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
        symptoms: str,
        trace_id: str | None = None,
        parent_span_id: str | None = None,
    ) -> tuple[TelemetryReport, HistoricalReport]:
        """Dispatch to specialists with the resolved entity scope.

        Specialist queries raise on retrieval failure like the main phase.
        """
        device_id = entity_context.devices[0]["id"] if entity_context.devices else "unknown"
        fleet_id = entity_context.fleet.get("id", "") if entity_context.fleet else ""
        deployment_id = entity_context.deployment.get("id", "") if entity_context.deployment else ""

        tel_span_id = self._start_sub_span(trace_id, TELEMETRY_SPECIALIST, parent_span_id)
        telemetry_report = await self._telemetry.investigate(
            device_id=device_id,
            fleet_id=fleet_id,
            symptoms=symptoms,
        )
        self._end_sub_span(tel_span_id)

        hist_span_id = self._start_sub_span(trace_id, HISTORICAL_SPECIALIST, parent_span_id)
        account_id = entity_context.account.get("id", "") if entity_context.account else ""
        entity_ids = [e for e in (account_id, device_id, fleet_id, deployment_id) if e]
        historical_report = await self._historical.investigate(
            entity_ids=entity_ids,
            entity_types=["account", "device", "fleet", "deployment"],
            symptoms=symptoms,
        )
        self._end_sub_span(hist_span_id)

        return telemetry_report, historical_report

    async def _generate_hypotheses(
        self,
        entity_context: EntityContext,
        evidence: list[EvidenceItem],
        telemetry: TelemetryReport,
        historical: HistoricalReport,
        severity: str = "medium",
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
                    description=f"Software version {version} introduced a regression on the "
                    f"affected devices — telemetry anomalies began with this deployment and "
                    f"similar past incidents required a rollback",
                    confidence=0.85,
                    evidence_ids=[e.source_id for e in evidence[:3]]
                    + telemetry.evidence_references[:2],
                    severity=severity,
                    is_primary=True,
                )
            )

        if historical.recurring_patterns:
            hypotheses.append(
                Hypothesis(
                    id="H-2",
                    description=f"Recurring issue pattern — "
                    f"{len(historical.recurring_patterns)} similar past incidents detected",
                    confidence=0.65,
                    evidence_ids=historical.evidence_references[:2],
                    severity=severity,
                    is_primary=False,
                )
            )

        if not hypotheses:
            hypotheses.append(
                Hypothesis(
                    id="H-1",
                    description="Insufficient evidence for confident hypothesis — further "
                    "investigation required",
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

    def _generate_outputs(
        self,
        entity_context: EntityContext,
        hypotheses: list[Hypothesis],
        telemetry: TelemetryReport,
        historical: HistoricalReport,
        governance: GovernanceDecision,
        site_name: str = "",
        trace_id: str | None = None,
        parent_span_id: str | None = None,
    ) -> tuple[str, str | None]:
        """Generate outputs, gated by the governance decision.

        v1 built both drafts here as fixed templates and never consulted
        governance — a blocked customer draft was returned anyway, claiming a
        root cause and a two-hour resolution. The customer draft is now
        emitted only when governance approves it, and both outputs are built
        from the actual evidence and hypotheses.
        """
        draft_allowed = customer_draft_allowed(governance)

        briefing_span_id = self._start_sub_span(
            trace_id,
            LLM_OPERATOR_BRIEFING,
            parent_span_id,
        )
        operator_briefing = build_operator_briefing(
            entity_context, hypotheses, telemetry, historical, governance, draft_allowed
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
            build_customer_draft(entity_context, hypotheses, site_name) if draft_allowed else None
        )
        self._end_sub_span(
            response_span_id,
            output=(
                {"length": len(customer_response)} if customer_response else {"withheld": True}
            ),
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
