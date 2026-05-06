from enum import Enum

from app.schemas.investigation import Hypothesis


class ActionCategory(str, Enum):
    INVESTIGATE = "investigate"
    RECOMMEND = "recommend"
    ESCALATE = "escalate"
    COMMUNICATE = "communicate"
    EXECUTE = "execute"


class SeverityLevel(str, Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


class CustomerSensitivity(str, Enum):
    INTERNAL_ONLY = "internal_only"
    CUSTOMER_FACING = "customer_facing"
    VIP_CUSTOMER = "vip_customer"


def classify_action(
    hypothesis: Hypothesis | None = None,
    severity: str = "medium",
    customer_sensitivity: str = "internal_only",
    evidence_confidence: float = 0.5,
) -> ActionCategory:
    """Classify the recommended action based on incident characteristics."""
    sev = SeverityLevel(severity.lower())
    sens = CustomerSensitivity(customer_sensitivity.lower())

    # CRITICAL always escalates
    if sev == SeverityLevel.CRITICAL:
        return ActionCategory.ESCALATE

    # HIGH + customer-facing escalates
    if sev == SeverityLevel.HIGH and sens in (CustomerSensitivity.CUSTOMER_FACING, CustomerSensitivity.VIP_CUSTOMER):
        return ActionCategory.ESCALATE

    # HIGH + VIP always escalates
    if sev == SeverityLevel.HIGH and sens == CustomerSensitivity.VIP_CUSTOMER:
        return ActionCategory.ESCALATE

    # VIP customer + any severity above LOW = communicate
    if sens == CustomerSensitivity.VIP_CUSTOMER and sev != SeverityLevel.LOW:
        return ActionCategory.COMMUNICATE

    # MEDIUM with good evidence = recommend
    if sev == SeverityLevel.MEDIUM and evidence_confidence >= 0.6:
        return ActionCategory.RECOMMEND

    # Default: investigate
    return ActionCategory.INVESTIGATE
