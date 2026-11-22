"""Integration test fixtures: REAL Postgres and Qdrant, no mocks.

These tests exist because v1 had a green suite beside a broken live path —
every retrieval test mocked Qdrant, so the broken hybrid query never failed a
test. Set both environment variables to enable:

    OPSFLOW_TEST_DATABASE_URL=postgresql+asyncpg://opsflow:opsflow@localhost:5432/opsflow
    OPSFLOW_TEST_QDRANT_URL=http://localhost:6333

Without them the integration tests skip cleanly (the unit gate stays green).
"""

import asyncio
import os

import pytest
import pytest_asyncio
from httpx import ASGITransport, AsyncClient

TEST_DATABASE_URL = os.environ.get("OPSFLOW_TEST_DATABASE_URL", "")
TEST_QDRANT_URL = os.environ.get("OPSFLOW_TEST_QDRANT_URL", "")

SERVICES_MISSING_REASON = (
    "integration tests need OPSFLOW_TEST_DATABASE_URL and OPSFLOW_TEST_QDRANT_URL "
    "pointing at real Postgres/Qdrant instances"
)


def _services_configured() -> bool:
    return bool(TEST_DATABASE_URL and TEST_QDRANT_URL)


@pytest_asyncio.fixture(loop_scope="session", scope="session")
async def api_client():
    if not _services_configured():
        pytest.skip(SERVICES_MISSING_REASON)

    # The root conftest forces a sqlite URL into the environment for unit
    # tests; integration tests must override it before the app builds engines.
    os.environ["DATABASE_URL"] = TEST_DATABASE_URL
    os.environ["QDRANT_URL"] = TEST_QDRANT_URL
    os.environ.setdefault("LLM_API_KEY", "")

    from app.main import create_app, run_migrations
    from app.retrieval.client import COLLECTION_NAME, QdrantManager

    # Schema via the same Alembic path production uses at startup.
    await asyncio.to_thread(run_migrations)

    # A fresh Qdrant collection so evidence counts are deterministic.
    manager = QdrantManager(url=TEST_QDRANT_URL)
    try:
        collections = [c.name for c in manager.client.get_collections().collections]
        if COLLECTION_NAME in collections:
            manager.client.delete_collection(COLLECTION_NAME)
    except Exception as exc:
        pytest.skip(f"Qdrant unreachable at {TEST_QDRANT_URL}: {exc}")

    app = create_app()
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://integration") as client:
        response = await client.post("/api/v1/seed")
        assert response.status_code == 200, response.text
        yield client
