from __future__ import annotations

import logging

from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.ext.asyncio import AsyncSession

from app.db.session import get_db
from app.orchestrator.investigation import InvestigationManager
from app.orchestrator.resolution import ConflictingSignalsError, UnknownSignalsError
from app.retrieval.client import QdrantManager
from app.schemas.investigation import (
    InvestigationRequest,
    InvestigationResponse,
    SeedResult,
    SignalIds,
)
from app.tracing.langfuse import create_tracer

logger = logging.getLogger(__name__)

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
    db: AsyncSession = Depends(get_db),
) -> InvestigationResponse:
    """Run an investigation over the signals' resolved entity graph.

    Failure semantics (v1 returned 200 with an unrelated investigation):

    - 404 when a provided signal ID does not exist — we never investigate
      fiction, and unknown IDs must not resolve to unrelated incidents.
    - 422 when the provided signals do not belong to a single account.
    - 503 when a dependency fails mid-investigation (e.g. Qdrant is down) —
      a broken retrieval path must be visible, never an empty success.
    """
    _validate_signal_ids(request.signal_ids)

    tracer = create_tracer()
    manager = InvestigationManager(qdrant, trace_callback=tracer)
    try:
        result = await manager.run_investigation(request.signal_ids, db_session=db)
    except UnknownSignalsError as exc:
        raise HTTPException(status_code=404, detail=str(exc)) from exc
    except ConflictingSignalsError as exc:
        raise HTTPException(status_code=422, detail=str(exc)) from exc
    except Exception as exc:
        logger.exception("Investigation failed for signals %s", request.signal_ids)
        raise HTTPException(
            status_code=503,
            detail=f"Investigation failed: {type(exc).__name__}: {exc}",
        ) from exc
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
    """Seed Postgres with synthetic entity data and index evidence into Qdrant.

    The three fixture scenarios are disjoint: each has its own account, site,
    fleet, devices and evidence, so distinct investigations can be verified.
    """
    from app.retrieval.indexer import index_all_evidence
    from app.seed.entities import seed_all

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
