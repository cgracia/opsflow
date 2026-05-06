from unittest.mock import AsyncMock, MagicMock, patch

import pytest
import pytest_asyncio
from httpx import ASGITransport, AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine

from app.api.investigations import get_qdrant_manager
from app.db.base import Base
from app.db.session import get_db
from app.main import create_app


@pytest.fixture
def mock_qdrant():
    m = MagicMock()
    m._collection_ready = True
    return m


@pytest_asyncio.fixture
async def api_client(mock_qdrant):
    app = create_app()
    app.dependency_overrides[get_qdrant_manager] = lambda: mock_qdrant

    engine = create_async_engine("sqlite+aiosqlite:///:memory:")
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
    session_factory = async_sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)

    async def _override_get_db():
        async with session_factory() as session:
            yield session

    app.dependency_overrides[get_db] = _override_get_db

    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        yield client
    app.dependency_overrides.clear()
    await engine.dispose()


@pytest.mark.asyncio
async def test_health_endpoint(api_client: AsyncClient) -> None:
    response = await api_client.get("/api/v1/healthz")
    assert response.status_code == 200
    assert response.json() == {"status": "ok"}


@pytest.mark.asyncio
async def test_seed_endpoint(api_client: AsyncClient) -> None:
    response = await api_client.post("/api/v1/seed")
    assert response.status_code == 200
    body = response.json()
    assert "entity_counts" in body
    assert "evidence_count" in body
    assert body["message"] == "Seed completed successfully"
    assert body["entity_counts"]["accounts"] == 1
    assert body["entity_counts"]["devices"] == 8
    assert body["entity_counts"]["tickets"] == 1
    assert body["evidence_count"] == 11


@pytest.mark.asyncio
async def test_investigation_endpoint_success(api_client: AsyncClient) -> None:
    with (
        patch(
            "app.orchestrator.investigation.search_evidence",
            new_callable=AsyncMock,
            return_value=[],
        ),
        patch("app.specialists.telemetry.search_evidence", new_callable=AsyncMock, return_value=[]),
        patch(
            "app.specialists.historical.search_evidence", new_callable=AsyncMock, return_value=[]
        ),
    ):
        response = await api_client.post(
            "/api/v1/investigations",
            json={"signal_ids": {"ticket_id": "TCK-1001"}},
        )

    assert response.status_code == 200
    body = response.json()
    assert body["investigation_id"].startswith("INV-")
    assert body["trace_id"]
    assert isinstance(body["hypotheses"], list)
    assert body["governance_decision"] is not None
    assert body["operator_briefing"]
    assert body["customer_response_draft"]


@pytest.mark.asyncio
async def test_investigation_validation_empty_signals(api_client: AsyncClient) -> None:
    response = await api_client.post(
        "/api/v1/investigations",
        json={"signal_ids": {}},
    )
    assert response.status_code == 422


@pytest.mark.asyncio
async def test_investigation_not_found(api_client: AsyncClient) -> None:
    response = await api_client.get("/api/v1/investigations/nonexistent-id")
    assert response.status_code == 404
    assert "not yet implemented" in response.json()["detail"].lower()


@pytest.mark.asyncio
async def test_openapi_docs_available(api_client: AsyncClient) -> None:
    response = await api_client.get("/docs")
    assert response.status_code == 200


@pytest.mark.asyncio
async def test_investigation_endpoint_uses_tracer(api_client: AsyncClient) -> None:
    with (
        patch(
            "app.orchestrator.investigation.search_evidence",
            new_callable=AsyncMock,
            return_value=[],
        ),
        patch("app.specialists.telemetry.search_evidence", new_callable=AsyncMock, return_value=[]),
        patch(
            "app.specialists.historical.search_evidence", new_callable=AsyncMock, return_value=[]
        ),
        patch("app.api.investigations.create_tracer") as mock_create_tracer,
    ):
        from app.tracing.langfuse import LangfuseTracer, _NoopTracerCore

        noop_tracer = LangfuseTracer(client=_NoopTracerCore())
        mock_create_tracer.return_value = noop_tracer

        response = await api_client.post(
            "/api/v1/investigations",
            json={"signal_ids": {"ticket_id": "TCK-1001"}},
        )

    assert response.status_code == 200
    mock_create_tracer.assert_called_once()


@pytest.mark.asyncio
async def test_seed_endpoint_calls_indexer(api_client: AsyncClient, mock_qdrant) -> None:
    with patch(
        "app.retrieval.indexer.index_all_evidence", new_callable=AsyncMock, return_value=11
    ) as mock_index:
        response = await api_client.post("/api/v1/seed")

    assert response.status_code == 200
    body = response.json()
    assert body["evidence_count"] == 11
    mock_index.assert_called_once_with(mock_qdrant)
