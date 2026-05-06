import pytest
from unittest.mock import MagicMock, patch

from app.retrieval.client import QdrantManager, COLLECTION_NAME, VECTOR_SIZE


class TestQdrantManager:
    def test_ensure_collection_creates_if_missing(self):
        manager = QdrantManager.__new__(QdrantManager)
        manager.client = MagicMock()
        manager._collection_ready = False
        manager.client.get_collections.return_value.collections = []

        manager.ensure_collection()

        manager.client.create_collection.assert_called_once()
        assert manager._collection_ready is True

    def test_ensure_collection_skips_if_exists(self):
        manager = QdrantManager.__new__(QdrantManager)
        manager.client = MagicMock()
        manager._collection_ready = False
        mock_collection = MagicMock()
        mock_collection.name = COLLECTION_NAME
        manager.client.get_collections.return_value.collections = [mock_collection]

        manager.ensure_collection()

        manager.client.create_collection.assert_not_called()
        assert manager._collection_ready is True

    def test_fake_vector_deterministic(self):
        vec1 = QdrantManager._fake_vector("test-doc-1")
        vec2 = QdrantManager._fake_vector("test-doc-1")
        assert vec1 == vec2
        assert len(vec1) == VECTOR_SIZE

    def test_fake_vector_different_for_different_ids(self):
        vec1 = QdrantManager._fake_vector("doc-1")
        vec2 = QdrantManager._fake_vector("doc-2")
        assert vec1 != vec2

    def test_index_document_calls_upsert(self):
        manager = QdrantManager.__new__(QdrantManager)
        manager.client = MagicMock()
        manager._collection_ready = True

        manager.index_document(
            doc_id="test-1",
            content="navigation error device blocked",
            entity_id="DEV-401",
            entity_type="device",
            source_type="telemetry",
        )

        manager.client.upsert.assert_called_once()
        call_args = manager.client.upsert.call_args
        assert call_args.kwargs["collection_name"] == COLLECTION_NAME
        points = call_args.kwargs["points"]
        assert len(points) == 1
        assert points[0].id == "test-1"

    def test_index_documents_batch(self):
        manager = QdrantManager.__new__(QdrantManager)
        manager.client = MagicMock()
        manager._collection_ready = True

        docs = [
            {"id": f"doc-{i}", "entity_id": "DEV-401", "entity_type": "device",
             "source_type": "ticket", "content": f"content {i}"}
            for i in range(3)
        ]

        manager.index_documents(docs)
        manager.client.upsert.assert_called_once()
        points = manager.client.upsert.call_args.kwargs["points"]
        assert len(points) == 3


class TestSearchFunctions:
    @pytest.mark.asyncio
    async def test_search_evidence_returns_results(self):
        from app.retrieval.search import search_evidence

        manager = QdrantManager.__new__(QdrantManager)
        manager.client = MagicMock()
        manager._collection_ready = True

        mock_point = MagicMock()
        mock_point.id = "doc-1"
        mock_point.score = 0.95
        mock_point.payload = {"entity_id": "DEV-401", "content": "navigation error"}
        manager.client.query_points.return_value = MagicMock(points=[mock_point])

        results = await search_evidence("navigation error", manager)
        assert len(results) == 1
        assert results[0]["entity_id"] == "DEV-401"

    @pytest.mark.asyncio
    async def test_search_by_entity_filters_correctly(self):
        from app.retrieval.search import search_by_entity

        manager = QdrantManager.__new__(QdrantManager)
        manager.client = MagicMock()
        manager._collection_ready = True

        mock_point = MagicMock()
        mock_point.id = "doc-1"
        mock_point.score = 0.85
        mock_point.payload = {"entity_id": "DEV-401"}
        manager.client.query_points.return_value = MagicMock(points=[mock_point])

        results = await search_by_entity("DEV-401", manager)
        assert len(results) == 1
