from app.governance.classification import (
    ActionCategory,
    classify_action,
)


class GovernanceDecision:
    """Result of governance evaluation."""

    def __init__(
        self,
        action_classification: ActionCategory,
        severity: str,
        customer_sensitivity: str,
        approved_actions: list[str],
        blocked_actions: list[str],
        escalation_required: bool,
        confidence_threshold: float,
        reasoning: str,
    ):
        self.action_classification = action_classification
        self.severity = severity
        self.customer_sensitivity = customer_sensitivity
        self.approved_actions = approved_actions
        self.blocked_actions = blocked_actions
        self.escalation_required = escalation_required
        self.confidence_threshold = confidence_threshold
        self.reasoning = reasoning

    def to_dict(self) -> dict:
        return {
            "action_classification": self.action_classification.value,
            "severity": self.severity,
            "customer_sensitivity": self.customer_sensitivity,
            "approved_actions": self.approved_actions,
            "blocked_actions": self.blocked_actions,
            "escalation_required": self.escalation_required,
            "confidence_threshold": self.confidence_threshold,
            "reasoning": self.reasoning,
        }


# Actions that are always blocked in v1
ALWAYS_BLOCKED = [ActionCategory.EXECUTE]

# Map of which actions are allowed per classification
ALLOWED_ACTIONS = {
    ActionCategory.INVESTIGATE: ["retrieve_evidence", "query_telemetry", "query_history"],
    ActionCategory.RECOMMEND: [
        "retrieve_evidence",
        "query_telemetry",
        "query_history",
        "draft_recommendation",
    ],
    ActionCategory.ESCALATE: [
        "retrieve_evidence",
        "query_telemetry",
        "query_history",
        "notify_operator",
        "draft_escalation",
    ],
    ActionCategory.COMMUNICATE: [
        "retrieve_evidence",
        "query_telemetry",
        "query_history",
        "draft_customer_response",
    ],
    ActionCategory.EXECUTE: [],  # Always blocked
}


class GovernanceEngine:
    """Evaluates governance constraints on investigation actions."""

    def evaluate(
        self,
        severity: str = "medium",
        customer_sensitivity: str = "internal_only",
        evidence_confidence: float = 0.5,
        hypothesis_description: str = "",
    ) -> GovernanceDecision:
        """Evaluate governance constraints and return decision."""
        action = classify_action(
            severity=severity,
            customer_sensitivity=customer_sensitivity,
            evidence_confidence=evidence_confidence,
        )

        # Determine approved and blocked actions
        approved = ALLOWED_ACTIONS.get(action, [])
        blocked = [a.value for a in ALWAYS_BLOCKED]

        # Additional blocks for low confidence
        if evidence_confidence < 0.3:
            blocked.extend(["draft_recommendation", "draft_customer_response"])
            approved = [a for a in approved if a not in blocked]

        # Escalation required for high severity + customer impact
        escalation_required = severity in ("high", "critical") and customer_sensitivity in (
            "customer_facing",
            "vip_customer",
        )

        reasoning = self._build_reasoning(
            action, severity, customer_sensitivity, evidence_confidence
        )

        return GovernanceDecision(
            action_classification=action,
            severity=severity,
            customer_sensitivity=customer_sensitivity,
            approved_actions=approved,
            blocked_actions=blocked,
            escalation_required=escalation_required,
            confidence_threshold=max(0.5, evidence_confidence),
            reasoning=reasoning,
        )

    def _build_reasoning(
        self, action: ActionCategory, severity: str, sensitivity: str, confidence: float
    ) -> str:
        parts = [f"Action classified as {action.value}."]
        parts.append(f"Severity: {severity}, sensitivity: {sensitivity}.")
        if confidence < 0.3:
            parts.append("Low evidence confidence — output generation restricted.")
        if action == ActionCategory.ESCALATE:
            parts.append("Escalation required due to severity and customer sensitivity.")
        parts.append("EXECUTE actions always blocked in v1.")
        return " ".join(parts)
