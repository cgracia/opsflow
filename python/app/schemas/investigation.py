from datetime import datetime

from pydantic import BaseModel, ConfigDict


# --- Shared Types ---


class EvidenceItem(BaseModel):
    """A piece of evidence retrieved during investigation."""

    source_type: str  # ticket, doc, telemetry, runbook, log
    source_id: str
    entity_id: str
    entity_type: str
    content: str
    relevance_score: float = 0.0
    timestamp: datetime | None = None


class Hypothesis(BaseModel):
    """A ranked hypothesis about the incident cause."""

    id: str
    description: str
    confidence: float  # 0.0 - 1.0
    evidence_ids: list[str] = []
    severity: str = "medium"
    is_primary: bool = False


class TelemetryReport(BaseModel):
    """Output from the telemetry specialist."""

    findings: list[str] = []
    anomalies_detected: list[str] = []
    event_timeline: list[dict] = []
    confidence: float = 0.0
    evidence_references: list[str] = []


class HistoricalReport(BaseModel):
    """Output from the historical incident specialist."""

    similar_incidents: list[dict] = []
    recurring_patterns: list[str] = []
    deployment_adjacency: list[dict] = []
    known_issues: list[dict] = []
    confidence: float = 0.0
    evidence_references: list[str] = []


class GovernanceDecision(BaseModel):
    """Governance evaluation result."""

    model_config = ConfigDict(from_attributes=True)

    action_classification: str  # INVESTIGATE, RECOMMEND, ESCALATE, COMMUNICATE, EXECUTE
    approved_actions: list[str] = []
    blocked_actions: list[str] = []
    escalation_required: bool = False
    severity: str = "medium"
    customer_sensitivity: str = "internal_only"  # internal_only, customer_facing, vip_customer
    confidence_threshold: float = 0.5
    reasoning: str = ""


class EntityContext(BaseModel):
    """Resolved entity context for the investigation."""

    account: dict | None = None
    site: dict | None = None
    fleet: dict | None = None
    devices: list[dict] = []
    deployment: dict | None = None
    software_revision: dict | None = None


# --- Request/Response ---


class SignalIds(BaseModel):
    """Signal identifiers that triggered the investigation."""

    ticket_id: str | None = None
    alert_id: str | None = None
    event_id: str | None = None


class InvestigationRequest(BaseModel):
    """Request to start an investigation."""

    signal_ids: SignalIds


class InvestigationResponse(BaseModel):
    """Complete investigation result."""

    model_config = ConfigDict(from_attributes=True)

    investigation_id: str
    trace_id: str
    entity_context: EntityContext
    evidence: list[EvidenceItem] = []
    telemetry_analysis: TelemetryReport | None = None
    historical_analysis: HistoricalReport | None = None
    hypotheses: list[Hypothesis] = []
    governance_decision: GovernanceDecision | None = None
    operator_briefing: str = ""
    customer_response_draft: str = ""
    created_at: datetime | None = None


class SeedResult(BaseModel):
    """Result of seeding operation."""

    entity_counts: dict[str, int]
    evidence_count: int
    message: str = "Seed completed successfully"
