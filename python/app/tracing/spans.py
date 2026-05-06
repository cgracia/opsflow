"""Named span definitions matching investigation phases and sub-operations."""

# --- Phase spans (match InvestigationPhase values from phases.py) ---

SIGNAL_INGESTION = "signal_ingestion"
ENTITY_RESOLUTION = "entity_resolution"
EVIDENCE_RETRIEVAL = "evidence_retrieval"
SPECIALIST_INVESTIGATION = "specialist_investigation"
HYPOTHESIS_GENERATION = "hypothesis_generation"
GOVERNANCE_EVALUATION = "governance_evaluation"
OUTPUT_GENERATION = "output_generation"

PHASE_SPANS = [
    SIGNAL_INGESTION,
    ENTITY_RESOLUTION,
    EVIDENCE_RETRIEVAL,
    SPECIALIST_INVESTIGATION,
    HYPOTHESIS_GENERATION,
    GOVERNANCE_EVALUATION,
    OUTPUT_GENERATION,
]

# --- Sub-spans ---

TELEMETRY_SPECIALIST = "specialist.telemetry"
HISTORICAL_SPECIALIST = "specialist.historical"
RETRIEVAL_HYBRID = "retrieval.hybrid"
RETRIEVAL_ENTITY = "retrieval.entity"
LLM_HYPOTHESIS_GENERATION = "llm.hypothesis_generation"
LLM_OPERATOR_BRIEFING = "llm.operator_briefing"
LLM_CUSTOMER_RESPONSE = "llm.customer_response"
GOVERNANCE_EVALUATE = "governance.evaluate"

# --- Span types for classification ---

SPAN_TYPE_PHASE = "phase"
SPAN_TYPE_GENERATION = "generation"
SPAN_TYPE_TOOL = "tool"
