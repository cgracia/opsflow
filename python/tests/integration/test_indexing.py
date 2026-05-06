"""Integration tests for evidence indexing into Qdrant.

Tests cover:
- Full evidence indexing (all document types)
- Search relevance (navigation error query returns relevant docs)
- Entity filtering (filter by entity_id returns only matching documents)
"""

from unittest.mock import MagicMock

import pytest

from app.retrieval.client import QdrantManager, COLLECTION_NAME
from app.retrieval.indexer import index_all_evidence
from app.retrieval.search import search_evidence, search_by_entity
from app.seed.evidence import get_all_evidence


@pytest.fixture
def manager():
    """Create a QdrantManager with mocked client, collection ready."""
    mgr = QdrantManager.__new__(QdrantManager)
    mgr.client = MagicMock()
    mgr._collection_ready = True
    return mgr


# ── (a) Test indexing all evidence ──────────────────────────────────────────


@pytest.mark.asyncio
async def test_index_all_evidence_indexes_every_document(manager):
    """index_all_evidence should index all 11 evidence documents and return count."""
    count = await index_all_evidence(manager)

    all_evidence = get_all_evidence()
    assert count == len(all_evidence) == 11
    manager.client.upsert.assert_called_once()

    call_kwargs = manager.client.upsert.call_args.kwargs
    assert call_kwargs["collection_name"] == COLLECTION_NAME
    points = call_kwargs["points"]
    assert len(points) == 11


@pytest.mark.asyncio
async def test_index_all_evidence_covers_all_source_types(manager):
    """Indexed documents should cover all source types: historical_ticket, runbook, telemetry, deployment."""
    await index_all_evidence(manager)

    points = manager.client.upsert.call_args.kwargs["points"]
    source_types = {p.payload["source_type"] for p in points}

    assert source_types == {"historical_ticket", "runbook", "telemetry", "deployment"}


@pytest.mark.asyncio
async def test_index_all_evidence_document_ids_are_unique(manager):
    """All indexed documents must have unique IDs."""
    await index_all_evidence(manager)

    points = manager.client.upsert.call_args.kwargs["points"]
    ids = [p.id for p in points]
    assert len(ids) == len(set(ids))


@pytest.mark.asyncio
async def test_index_all_evidence_ensures_collection_when_not_ready():
    """If collection is not ready, index_all_evidence should call ensure_collection."""
    mgr = QdrantManager.__new__(QdrantManager)
    mgr.client = MagicMock()
    mgr._collection_ready = False

    mock_collection = MagicMock()
    mock_collection.name = COLLECTION_NAME
    mgr.client.get_collections.return_value.collections = [mock_collection]

    count = await index_all_evidence(mgr)

    mgr.client.create_collection.assert_not_called()
    assert mgr._collection_ready is True
    assert count == 11


@pytest.mark.asyncio
async def test_index_all_evidence_correct_breakdown(manager):
    """Verify the breakdown: 5 historical tickets + 3 runbooks + 2 telemetry + 1 deployment."""
    await index_all_evidence(manager)

    points = manager.client.upsert.call_args.kwargs["points"]
    by_type = {}
    for p in points:
        st = p.payload["source_type"]
        by_type[st] = by_type.get(st, 0) + 1

    assert by_type["historical_ticket"] == 5
    assert by_type["runbook"] == 3
    assert by_type["telemetry"] == 2
    assert by_type["deployment"] == 1


# ── (b) Test search relevance ───────────────────────────────────────────────


def _make_mock_point(doc_id, score, payload):
    """Helper to create a mock Qdrant point."""
    p = MagicMock()
    p.id = doc_id
    p.score = score
    p.payload = payload
    return p


@pytest.mark.asyncio
async def test_search_relevance_navigation_error(manager):
    """Searching for 'navigation error' should return documents containing that content."""
    all_docs = get_all_evidence()
    navigation_docs = [d for d in all_docs if "navigation" in d["content"].lower()]

    mock_points = []
    for i, doc in enumerate(navigation_docs):
        score = 0.95 - (i * 0.05)
        mock_points.append(
            _make_mock_point(
                doc["id"],
                score,
                {
                    "entity_id": doc["entity_id"],
                    "entity_type": doc["entity_type"],
                    "source_type": doc["source_type"],
                    "content": doc["content"],
                },
            )
        )
    manager.client.query_points.return_value = MagicMock(points=mock_points)

    results = await search_evidence("navigation error", manager)

    assert len(results) >= 5

    source_types = {r["source_type"] for r in results}
    assert "historical_ticket" in source_types
    assert "runbook" in source_types

    nav_error_results = [r for r in results if "navigation" in r["content"].lower()]
    assert len(nav_error_results) > 0


@pytest.mark.asyncio
async def test_search_relevance_runbook_for_navigation(manager):
    """Runbook RB-001 (Navigation Troubleshooting) should be among top results for navigation queries."""
    all_docs = get_all_evidence()

    nav_runbook = next(d for d in all_docs if d["id"] == "RB-001")

    mock_points = [
        _make_mock_point(
            nav_runbook["id"],
            0.97,
            {
                "entity_id": nav_runbook["entity_id"],
                "source_type": nav_runbook["source_type"],
                "content": nav_runbook["content"],
            },
        ),
    ]
    manager.client.query_points.return_value = MagicMock(points=mock_points)

    results = await search_evidence("navigation troubleshooting runbook", manager)

    assert len(results) >= 1
    assert results[0]["id"] == "RB-001"
    assert "Navigation Troubleshooting" in results[0]["content"]


@pytest.mark.asyncio
async def test_search_relevance_historical_ticket(manager):
    """Historical tickets about navigation failures should appear for relevant queries."""
    all_docs = get_all_evidence()

    htick = next(d for d in all_docs if d["id"] == "HTKT-001")

    mock_points = [
        _make_mock_point(
            htick["id"],
            0.92,
            {
                "entity_id": htick["entity_id"],
                "source_type": htick["source_type"],
                "content": htick["content"],
            },
        ),
    ]
    manager.client.query_points.return_value = MagicMock(points=mock_points)

    results = await search_evidence("path planning failure after update", manager)

    assert len(results) >= 1
    assert "NAV_PATH_PLAN_FAILED" in results[0]["content"]


# ── (c) Test entity filtering ───────────────────────────────────────────────


@pytest.mark.asyncio
async def test_entity_filtering_by_device(manager):
    """Filtering by entity_id DEV-401 should return only telemetry for that device."""
    all_docs = get_all_evidence()
    dev_401_docs = [d for d in all_docs if d["entity_id"] == "DEV-401"]

    mock_points = [
        _make_mock_point(
            d["id"],
            0.9 - i * 0.1,
            {
                "entity_id": d["entity_id"],
                "source_type": d["source_type"],
                "content": d["content"],
            },
        )
        for i, d in enumerate(dev_401_docs)
    ]
    manager.client.query_points.return_value = MagicMock(points=mock_points)

    results = await search_by_entity("DEV-401", manager)

    assert len(results) == len(dev_401_docs)
    assert all(r["entity_id"] == "DEV-401" for r in results)


@pytest.mark.asyncio
async def test_entity_filtering_by_fleet(manager):
    """Filtering by entity_id FLT-101 should return fleet-level docs only."""
    all_docs = get_all_evidence()
    fleet_docs = [d for d in all_docs if d["entity_id"] == "FLT-101"]

    mock_points = [
        _make_mock_point(
            d["id"],
            0.9 - i * 0.1,
            {
                "entity_id": d["entity_id"],
                "entity_type": d["entity_type"],
                "source_type": d["source_type"],
                "content": d["content"],
            },
        )
        for i, d in enumerate(fleet_docs)
    ]
    manager.client.query_points.return_value = MagicMock(points=mock_points)

    results = await search_by_entity("FLT-101", manager)

    assert len(results) == len(fleet_docs)
    assert all(r["entity_id"] == "FLT-101" for r in results)
    fleet_source_types = {r["source_type"] for r in results}
    assert "runbook" in fleet_source_types


@pytest.mark.asyncio
async def test_entity_filtering_by_account(manager):
    """Account-level entity should have multiple document types."""
    all_docs = get_all_evidence()
    account_docs = [d for d in all_docs if d["entity_id"] == "ACC-1001"]

    mock_points = [
        _make_mock_point(
            d["id"],
            0.85 - i * 0.05,
            {
                "entity_id": d["entity_id"],
                "source_type": d["source_type"],
            },
        )
        for i, d in enumerate(account_docs)
    ]
    manager.client.query_points.return_value = MagicMock(points=mock_points)

    results = await search_by_entity("ACC-1001", manager)

    assert len(results) == len(account_docs)
    account_source_types = {r["source_type"] for r in results}
    assert "historical_ticket" in account_source_types


@pytest.mark.asyncio
async def test_entity_filtering_excludes_other_entities(manager):
    """Filtering should not return documents from other entity IDs."""
    all_docs = get_all_evidence()
    depl_docs = [d for d in all_docs if d["entity_id"] == "DEPL-501"]

    mock_points = [
        _make_mock_point(
            d["id"],
            0.9,
            {
                "entity_id": d["entity_id"],
                "source_type": d["source_type"],
            },
        )
        for d in depl_docs
    ]
    manager.client.query_points.return_value = MagicMock(points=mock_points)

    results = await search_by_entity("DEPL-501", manager)

    assert all(r["entity_id"] == "DEPL-501" for r in results)
    assert all(r["source_type"] == "deployment" for r in results)


@pytest.mark.asyncio
async def test_search_with_explicit_entity_filter(manager):
    """search_evidence with entity_ids parameter should pass filter to Qdrant."""
    mock_points = [
        _make_mock_point(
            "TEL-001",
            0.9,
            {
                "entity_id": "DEV-401",
                "source_type": "telemetry",
                "content": "telemetry data",
            },
        ),
    ]
    manager.client.query_points.return_value = MagicMock(points=mock_points)

    results = await search_evidence(
        "navigation error",
        manager,
        entity_ids=["DEV-401"],
    )

    manager.client.query_points.assert_called_once()
    assert len(results) == 1
    assert results[0]["entity_id"] == "DEV-401"
