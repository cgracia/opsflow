from enum import Enum


class InvestigationPhase(str, Enum):
    """Seven phases of an operational investigation."""

    SIGNAL_INGESTION = "signal_ingestion"
    ENTITY_RESOLUTION = "entity_resolution"
    EVIDENCE_RETRIEVAL = "evidence_retrieval"
    SPECIALIST_INVESTIGATION = "specialist_investigation"
    HYPOTHESIS_GENERATION = "hypothesis_generation"
    GOVERNANCE_EVALUATION = "governance_evaluation"
    OUTPUT_GENERATION = "output_generation"


PHASE_ORDER = [
    InvestigationPhase.SIGNAL_INGESTION,
    InvestigationPhase.ENTITY_RESOLUTION,
    InvestigationPhase.EVIDENCE_RETRIEVAL,
    InvestigationPhase.SPECIALIST_INVESTIGATION,
    InvestigationPhase.HYPOTHESIS_GENERATION,
    InvestigationPhase.GOVERNANCE_EVALUATION,
    InvestigationPhase.OUTPUT_GENERATION,
]
