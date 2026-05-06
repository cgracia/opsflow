from __future__ import annotations

from fastapi import APIRouter, Depends, HTTPException

from sqlalchemy.ext.asyncio import AsyncSession

from app.db.session import get_db
from app.schemas.investigation import (
    InvestigationRequest,
    InvestigationResponse,
    SeedResult,
    SignalIds,
)
from app.orchestrator.investigation import InvestigationManager
from app.retrieval.client import QdrantManager
from app.tracing.langfuse import create_tracer

router = APIRouter(tags=["investigations"])


def get_qdrant_manager() -> QdrantManager:
    from app.config import get_settings

    settings = get_settings()
    return QdrantManager(url=settings.qdrant_url, api_key=settings.qdrant_api_key)


def _validate_signal_ids(signal_ids: SignalIds) -> None:
    if not any([signal_ids.ticket_id, signal_ids.alert_id, signal_ids.event_id]):
        raise HTTPException(
            status_code=422,
            detail="At least one of ticket_id, alert_id, or event_id must be provided",
        )


@router.get("/healthz")
async def healthz() -> dict[str, str]:
    return {"status": "ok"}


@router.post("/investigations", response_model=InvestigationResponse)
async def create_investigation(
    request: InvestigationRequest,
    qdrant: QdrantManager = Depends(get_qdrant_manager),
) -> InvestigationResponse:
    _validate_signal_ids(request.signal_ids)

    tracer = create_tracer()
    manager = InvestigationManager(qdrant, trace_callback=tracer)
    result = await manager.run_investigation(request.signal_ids)
    return result


@router.get("/investigations/{investigation_id}")
async def get_investigation(investigation_id: str) -> dict[str, str]:
    raise HTTPException(
        status_code=404,
        detail="Investigation storage not yet implemented",
    )


@router.post("/seed", response_model=SeedResult)
async def seed_data(
    db: AsyncSession = Depends(get_db),
    qdrant: QdrantManager = Depends(get_qdrant_manager),
) -> SeedResult:
    """Seed Postgres with synthetic entity data and index evidence into Qdrant."""
    from app.seed.entities import seed_all
    from app.retrieval.indexer import index_all_evidence

    entity_map = await seed_all(db)

    entity_counts = {
        "accounts": len(entity_map["accounts"]),
        "sites": len(entity_map["sites"]),
        "software_revisions": len(entity_map["revisions"]),
        "fleets": len(entity_map["fleets"]),
        "devices": len(entity_map["devices"]),
        "deployments": len(entity_map["deployments"]),
        "services": len(entity_map["services"]),
        "incidents": len(entity_map["incidents"]),
        "tickets": len(entity_map["tickets"]),
        "operational_events": len(entity_map["events"]),
    }

    evidence_count = await index_all_evidence(qdrant)

    return SeedResult(
        entity_counts=entity_counts,
        evidence_count=evidence_count,
        message="Seed completed successfully",
    )
