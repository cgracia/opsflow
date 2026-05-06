import pytest
from httpx import AsyncClient

from app.config import Settings


@pytest.mark.asyncio
async def test_healthz_returns_ok(test_client: AsyncClient) -> None:
    response = await test_client.get("/api/v1/healthz")
    assert response.status_code == 200
    assert response.json() == {"status": "ok"}


def test_config_has_sensible_defaults() -> None:
    """Settings has reasonable default values."""
    settings = Settings(
        database_url="sqlite+aiosqlite:///:memory:",
        llm_api_key="test-key",
        env="test",
    )
    assert settings.llm_model == "gpt-4o"
    assert settings.llm_max_tokens == 2000
    assert settings.llm_temperature == 0.3
    assert settings.env == "test"
    assert settings.qdrant_url == "http://localhost:6333"


def test_config_overrides_from_env() -> None:
    """Settings picks up values from environment variables."""
    import os

    os.environ["LLM_MODEL"] = "gpt-3.5-turbo"
    os.environ["LLM_MAX_TOKENS"] = "500"

    settings = Settings(
        database_url="sqlite+aiosqlite:///:memory:",
        llm_api_key="test-key",
    )
    assert settings.llm_model == "gpt-3.5-turbo"
    assert settings.llm_max_tokens == 500

    # Cleanup
    del os.environ["LLM_MODEL"]
    del os.environ["LLM_MAX_TOKENS"]
