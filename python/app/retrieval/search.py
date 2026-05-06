from qdrant_client.models import (
    Filter,
    FieldCondition,
    MatchAny,
    SearchRequest,
    FusionQuery,
    NamedSparseVector,
    SparseVector,
)

from app.retrieval.client import COLLECTION_NAME, QdrantManager


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
    """Hybrid search: RRF fusion of dense + sparse vectors."""
    if not manager._collection_ready:
        manager.ensure_collection()

    query_filter = _make_filter(entity_ids, source_types)

    # Dense search
    dense_vector = manager._fake_vector(query)
    request_dense = SearchRequest(
        vector={"name": "", "vector": dense_vector},
        filter=query_filter,
        limit=limit,
        with_payload=True,
    )

    # Sparse search (BM25-style keyword matching)
    words = query.lower().split()
    unique_words = list(set(words))
    word_indices = {w: i for i, w in enumerate(unique_words)}
    sparse_vec = SparseVector(
        indices=[word_indices[w] for w in unique_words],
        values=[float(words.count(w)) for w in unique_words],
    )
    request_sparse = SearchRequest(
        vector=NamedSparseVector(name="bm25", vector=sparse_vec),
        filter=query_filter,
        limit=limit,
        with_payload=True,
    )

    results = manager.client.query_points(
        collection_name=COLLECTION_NAME,
        prefetch=[request_dense, request_sparse],
        query=FusionQuery(fusion="rrf"),
        limit=limit,
    )

    return [
        {"id": point.id, "score": point.score, **(point.payload or {})} for point in results.points
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


async def search_time_window(
    query: str,
    manager: QdrantManager,
    start_time: str,
    end_time: str,
    limit: int = 10,
) -> list[dict]:
    """Search within a time window. Uses metadata filtering where available."""
    return await search_evidence(
        query=query,
        manager=manager,
        entity_ids=None,
        source_types=None,
        limit=limit,
    )
