import pytest
from unittest.mock import AsyncMock, MagicMock, patch

from app.config import Settings
from app.llm.client import LLMClient, LLMResponse
from app.llm.prompts import ALL_PROMPTS, TELEMETRY_ANALYSIS, HYPOTHESIS_GENERATION


def test_llm_response_dataclass():
    resp = LLMResponse(text="hello", model="gpt-4o", total_tokens=10)
    assert resp.text == "hello"
    assert resp.model == "gpt-4o"
    assert resp.total_tokens == 10


@pytest.mark.asyncio
async def test_generate_returns_response():
    settings = Settings(database_url="sqlite+aiosqlite:///:memory:", llm_api_key="test-key")

    mock_choice = MagicMock()
    mock_choice.message.content = "Test response"
    mock_choice.finish_reason = "stop"

    mock_usage = MagicMock()
    mock_usage.prompt_tokens = 10
    mock_usage.completion_tokens = 5
    mock_usage.total_tokens = 15

    mock_response = MagicMock()
    mock_response.choices = [mock_choice]
    mock_response.usage = mock_usage
    mock_response.model = "gpt-4o"

    with patch("app.llm.client.AsyncOpenAI") as mock_openai_cls:
        mock_client = AsyncMock()
        mock_client.chat.completions.create = AsyncMock(return_value=mock_response)
        mock_openai_cls.return_value = mock_client

        client = LLMClient(settings)
        result = await client.generate("system prompt", "user prompt")

    assert isinstance(result, LLMResponse)
    assert result.text == "Test response"
    assert result.total_tokens == 15


@pytest.mark.asyncio
async def test_generate_structured_parses_json():
    settings = Settings(database_url="sqlite+aiosqlite:///:memory:", llm_api_key="test-key")

    mock_choice = MagicMock()
    mock_choice.message.content = '{"key": "value"}'
    mock_choice.finish_reason = "stop"

    mock_usage = MagicMock()
    mock_usage.prompt_tokens = 5
    mock_usage.completion_tokens = 3
    mock_usage.total_tokens = 8

    mock_response = MagicMock()
    mock_response.choices = [mock_choice]
    mock_response.usage = mock_usage
    mock_response.model = "gpt-4o"

    with patch("app.llm.client.AsyncOpenAI") as mock_openai_cls:
        mock_client = AsyncMock()
        mock_client.chat.completions.create = AsyncMock(return_value=mock_response)
        mock_openai_cls.return_value = mock_client

        client = LLMClient(settings)
        result = await client.generate_structured("system", "user")

    assert isinstance(result, dict)
    assert result["key"] == "value"


def test_all_prompts_have_six_entries():
    assert len(ALL_PROMPTS) == 6


def test_each_prompt_has_system_and_user():
    for name, prompt in ALL_PROMPTS.items():
        assert "system" in prompt, f"Prompt {name} missing 'system'"
        assert "user" in prompt, f"Prompt {name} missing 'user'"


def test_prompt_templates_render_with_sample_data():
    for name, prompt in ALL_PROMPTS.items():
        try:
            formatted = prompt["user"].format(
                device_id="DEV-401",
                fleet_id="FLT-101",
                start_time="2026-01-01",
                end_time="2026-01-02",
                telemetry_data="test",
                entity_ids="DEV-401",
                entity_types="device",
                symptoms="test",
                evidence="test",
                entity_context="test",
                evidence_summary="test",
                telemetry_findings="test",
                historical_findings="test",
                incident_id="INC-1",
                hypotheses="test",
                telemetry_summary="test",
                historical_summary="test",
                governance_decision="test",
                account_tier="enterprise",
                hypothesis="test",
                confidence="0.8",
                evidence_strength="strong",
                customer_impact="test",
                account_name="Test Corp",
                issue_summary="test",
                impact_summary="test",
                actions_summary="test",
                resolution_timeline="2 hours",
            )
            assert len(formatted) > 0
        except KeyError as e:
            pytest.fail(f"Prompt {name} missing placeholder: {e}")
