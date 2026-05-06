import pytest

from app.governance.classification import (
    ActionCategory, SeverityLevel, CustomerSensitivity, classify_action,
)
from app.governance.engine import GovernanceEngine


class TestClassification:
    def test_critical_always_escalates(self):
        result = classify_action(severity="critical", customer_sensitivity="internal_only")
        assert result == ActionCategory.ESCALATE

    def test_high_vip_escalates(self):
        result = classify_action(severity="high", customer_sensitivity="vip_customer")
        assert result == ActionCategory.ESCALATE

    def test_high_customer_facing_escalates(self):
        result = classify_action(severity="high", customer_sensitivity="customer_facing")
        assert result == ActionCategory.ESCALATE

    def test_medium_with_good_evidence_recommends(self):
        result = classify_action(severity="medium", evidence_confidence=0.7)
        assert result == ActionCategory.RECOMMEND

    def test_low_internal_investigates(self):
        result = classify_action(severity="low", customer_sensitivity="internal_only")
        assert result == ActionCategory.INVESTIGATE

    def test_vip_non_low_communicates(self):
        result = classify_action(severity="medium", customer_sensitivity="vip_customer")
        assert result == ActionCategory.COMMUNICATE


class TestGovernanceEngine:
    def setup_method(self):
        self.engine = GovernanceEngine()

    def test_high_severity_vip_requires_escalation(self):
        decision = self.engine.evaluate(
            severity="high", customer_sensitivity="vip_customer", evidence_confidence=0.8
        )
        assert decision.escalation_required is True
        assert "execute" in decision.blocked_actions

    def test_low_severity_internal_no_escalation(self):
        decision = self.engine.evaluate(
            severity="low", customer_sensitivity="internal_only"
        )
        assert decision.escalation_required is False
        assert decision.action_classification == ActionCategory.INVESTIGATE

    def test_execute_always_blocked(self):
        for sev in ["low", "medium", "high", "critical"]:
            for sens in ["internal_only", "customer_facing", "vip_customer"]:
                decision = self.engine.evaluate(severity=sev, customer_sensitivity=sens)
                assert "execute" in decision.blocked_actions, f"EXECUTE not blocked for {sev}/{sens}"

    def test_low_confidence_restricts_output(self):
        decision = self.engine.evaluate(
            severity="medium", evidence_confidence=0.1
        )
        assert "draft_recommendation" in decision.blocked_actions or "draft_customer_response" in decision.blocked_actions

    def test_decision_to_dict(self):
        decision = self.engine.evaluate(severity="high", customer_sensitivity="customer_facing")
        d = decision.to_dict()
        assert "action_classification" in d
        assert "approved_actions" in d
        assert "blocked_actions" in d
        assert "escalation_required" in d
        assert "reasoning" in d

    def test_approved_actions_non_empty(self):
        decision = self.engine.evaluate(severity="medium", evidence_confidence=0.7)
        assert len(decision.approved_actions) > 0
