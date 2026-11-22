"""Hybrid evidence search: RRF fusion of dense and sparse legs.

Both legs use the qdrant-client ``Prefetch`` API (``SearchRequest`` objects
were removed from ``query_points`` prefetch handling and raised a validation
error on every call, which callers used to swallow).

The dense leg is a deterministic hash of the query (disclosed simulation —
not semantically meaningful). The sparse leg uses the shared tokenization in
``app.retrieval.tokenization`` so query indices coincide with index-time
indices and keyword overlap actually matches.
"""

from qdrant_client.models import (
    FieldCondition,
    Filter,
    Fusion,
    FusionQuery,
    MatchAny,
    Prefetch,
)

from app.retrieval.client import COLLECTION_NAME, QdrantManager
from app.retrieval.tokenization import sparse_from_text


def _make_filter(
    entity_ids: list[str] | None = None,
    source_types: list[str] | None = None,
) -> Filter | None:
    conditions = []
    if entity_ids:
        conditions.append(FieldCondition(key="entity_id", match=MatchAny(any=entity_ids)))
    if source_types:
        conditions.append(FieldCondition(key="source_type", match=MatchAny(any=source_types)))
    return Filter(must=conditions) if conditions else None


async def search_evidence(
    query: str,
    manager: QdrantManager,
    entity_ids: list[str] | None = None,
    source_types: list[str] | None = None,
    limit: int = 10,
) -> list[dict]:
    """Hybrid search: RRF fusion of dense + sparse vectors.

    Raises whatever the Qdrant client raises on transport or request errors —
    callers must not swallow retrieval failures into empty results.
    """
    if not manager._collection_ready:
        manager.ensure_collection()

    query_filter = _make_filter(entity_ids, source_types)

    prefetch = [
        # Dense leg (deterministic hash vector — disclosed simulation)
        Prefetch(query=manager._fake_vector(query), filter=query_filter, limit=limit),
        # Sparse leg (shared tokenization, matches index-time vectors)
        Prefetch(query=sparse_from_text(query), using="bm25", filter=query_filter, limit=limit),
    ]

    results = manager.client.query_points(
        collection_name=COLLECTION_NAME,
        prefetch=prefetch,
        query=FusionQuery(fusion=Fusion.RRF),
        limit=limit,
        with_payload=True,
    )

    return [
        {
            # Prefer the readable document ID from the payload; the stored
            # point ID is a deterministic UUID (Qdrant >= 1.20 requirement).
            "id": (point.payload or {}).get("doc_id", str(point.id)),
            "score": point.score,
            **(point.payload or {}),
        }
        for point in results.points
    ]


async def search_by_entity(
    entity_id: str,
    manager: QdrantManager,
    limit: int = 5,
) -> list[dict]:
    """Search documents filtered by entity ID."""
    return await search_evidence(
        query=entity_id,
        manager=manager,
        entity_ids=[entity_id],
        limit=limit,
    )
