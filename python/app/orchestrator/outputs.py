"""Phase 7: output generation, gated by the governance decision.

v1 generated both drafts as fixed f-string templates and never consulted the
governance decision — a blocked ``draft_customer_response`` still produced a
draft claiming a root cause had been identified and promising a two-hour
resolution at 20% confidence. This module only emits what governance approves,
and builds drafts from the actual evidence and hypotheses: no invented root
causes, no promised resolution times (v1 has no data from which an ETA could
be derived, so none is ever stated).
"""

import logging

from app.schemas.investigation import (
    EntityContext,
    GovernanceDecision,
    HistoricalReport,
    Hypothesis,
    TelemetryReport,
)

logger = logging.getLogger(__name__)


def customer_draft_allowed(governance: GovernanceDecision) -> bool:
    """True only when the decision approves the draft and nothing blocks it."""
    approved = "draft_customer_response" in governance.approved_actions
    blocked = "draft_customer_response" in governance.blocked_actions
    if blocked:
        logger.info("Customer draft withheld: blocked by governance (%s)", governance.reasoning)
        return False
    return approved


def _primary(hypotheses: list[Hypothesis]) -> Hypothesis | None:
    return next((h for h in hypotheses if h.is_primary), hypotheses[0] if hypotheses else None)


def build_operator_briefing(
    entity_context: EntityContext,
    hypotheses: list[Hypothesis],
    telemetry: TelemetryReport,
    historical: HistoricalReport,
    governance: GovernanceDecision,
    draft_emitted: bool,
) -> str:
    account_name = (
        entity_context.account.get("name", "Unknown") if entity_context.account else "Unknown"
    )
    site_name = entity_context.site.get("name", "Unknown") if entity_context.site else "Unknown"
    primary = _primary(hypotheses)

    if primary is not None and primary.confidence >= 0.5:
        hypothesis_block = (
            f"  {primary.description}\n"
            f"  Confidence: {primary.confidence:.0%}\n"
            f"  Severity: {primary.severity}"
        )
    else:
        hypothesis_block = (
            "  Insufficient evidence for a confident hypothesis — further "
            "investigation required. No root cause should be claimed."
        )

    briefing = (
        "INVESTIGATION BRIEFING\n"
        "==================================================\n"
        f"Investigation for: {account_name}\n"
        "==================================================\n\n"
        "ENTITY CONTEXT:\n"
        f"  Account: {account_name}\n"
        f"  Site: {site_name}\n"
        f"  Fleet: {entity_context.fleet.get('name') if entity_context.fleet else '—'}\n"
        f"  Affected devices: {len(entity_context.devices)}\n\n"
        f"PRIMARY HYPOTHESIS:\n{hypothesis_block}\n\n"
        "TELEMETRY FINDINGS:\n"
        + "\n".join(f"  - {f}" for f in telemetry.findings[:5])
        + "\n\nHISTORICAL PATTERNS:\n"
        + "\n".join(f"  - {p}" for p in historical.recurring_patterns[:3])
        + "\n\nGOVERNANCE:\n"
        f"  Action: {governance.action_classification}\n"
        f"  Escalation: {'YES' if governance.escalation_required else 'NO'}\n"
        f"  Approved outputs: {', '.join(governance.approved_actions) or 'none'}\n"
        f"  {governance.reasoning}\n\n"
        "OUTPUT RESTRICTIONS:\n"
        f"  Customer response draft: "
        f"{'EMITTED' if draft_emitted else 'WITHHELD (not approved under current governance)'}\n"
    )
    return briefing


def build_customer_draft(
    entity_context: EntityContext,
    hypotheses: list[Hypothesis],
    site_name: str,
) -> str:
    """Draft a customer-safe response from actual evidence.

    The root cause is only described as identified when the primary hypothesis
    is strong; otherwise the draft states that investigation is ongoing. No
    resolution time is ever promised.
    """
    account_name = (
        entity_context.account.get("name", "our customer")
        if entity_context.account
        else "our customer"
    )
    primary = _primary(hypotheses)
    device_count = len(entity_context.devices)

    if primary is not None and primary.confidence >= 0.7:
        status_block = (
            "What we know so far: our investigation has converged on a likely cause — "
            f"{primary.description.split('.')[0].lower()}. "
            "We are treating this as the leading explanation and will confirm as we proceed."
        )
    else:
        status_block = (
            "What we know so far: we have confirmed an issue affecting "
            f"{device_count} of your devices at {site_name}. "
            "Our team is actively investigating; we do not yet have a confirmed root cause "
            "and will update you as soon as we do."
        )

    return (
        f"Dear {account_name} Team,\n\n"
        f"We are aware of the issue affecting your devices at {site_name}. "
        "This message is to acknowledge the impact and let you know where our "
        "investigation stands.\n\n"
        f"{status_block}\n\n"
        f"Current status: {device_count} device(s) impacted. "
        "Your other devices continue to operate normally.\n\n"
        "Next steps: our operations team is continuing the investigation and will provide "
        "the next update as soon as there is confirmed information to share. "
        "We are not stating a resolution time until we have evidence to support one.\n\n"
        "Please don't hesitate to reach out if you have any questions.\n\n"
        "Best regards,\nOperations Team"
    )
