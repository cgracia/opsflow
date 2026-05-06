import os

import pytest
import pytest_asyncio
from httpx import ASGITransport, AsyncClient

# Force test environment before importing app
os.environ["ENV"] = "test"
os.environ["DATABASE_URL"] = "sqlite+aiosqlite:///:memory:"
os.environ["LLM_API_KEY"] = "test-key-mock"

from app.config import Settings
from app.main import create_app


@pytest.fixture
def test_settings() -> Settings:
    """Settings configured for testing."""
    return Settings(
        database_url="sqlite+aiosqlite:///:memory:",
        llm_api_key="test-key-mock",
        llm_model="gpt-4o",
        env="test",
    )


@pytest_asyncio.fixture
async def test_client(test_settings: Settings) -> AsyncClient:
    """Async test client for the FastAPI application."""
    app = create_app()
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        yield client
