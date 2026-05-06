import json
import logging
from dataclasses import dataclass

from openai import AsyncOpenAI

from app.config import Settings

logger = logging.getLogger(__name__)


@dataclass
class LLMResponse:
    text: str
    model: str = ""
    prompt_tokens: int = 0
    completion_tokens: int = 0
    total_tokens: int = 0
    finish_reason: str = ""


class LLMClient:
    """Async OpenAI-compatible LLM client with retry support."""

    def __init__(self, settings: Settings | None = None):
        if settings is None:
            from app.config import get_settings

            settings = get_settings()
        self._settings = settings
        self._client = AsyncOpenAI(
            api_key=settings.llm_api_key or "dummy-key",
            base_url=settings.llm_api_base,
        )

    async def generate(
        self,
        system_prompt: str,
        user_prompt: str,
        temperature: float | None = None,
        max_tokens: int | None = None,
    ) -> LLMResponse:
        """Generate a text completion."""
        response = await self._client.chat.completions.create(
            model=self._settings.llm_model,
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt},
            ],
            temperature=temperature or self._settings.llm_temperature,
            max_tokens=max_tokens or self._settings.llm_max_tokens,
        )

        choice = response.choices[0]
        usage = response.usage

        return LLMResponse(
            text=choice.message.content or "",
            model=response.model,
            prompt_tokens=usage.prompt_tokens if usage else 0,
            completion_tokens=usage.completion_tokens if usage else 0,
            total_tokens=usage.total_tokens if usage else 0,
            finish_reason=choice.finish_reason or "",
        )

    async def generate_structured(
        self,
        system_prompt: str,
        user_prompt: str,
        response_schema: type | None = None,
        temperature: float | None = None,
    ) -> dict:
        """Generate a structured JSON response."""
        # Append JSON instruction to system prompt
        json_instruction = (
            "\n\nYou MUST respond with valid JSON only. No markdown, no explanation, just JSON."
        )
        full_system = system_prompt + json_instruction

        result = await self.generate(
            system_prompt=full_system,
            user_prompt=user_prompt,
            temperature=temperature or 0.1,
        )

        # Parse JSON from response
        text = result.text.strip()
        if text.startswith("```"):
            text = text.split("```")[1]
            if text.startswith("json"):
                text = text[4:]
            text = text.strip()

        try:
            return json.loads(text)
        except json.JSONDecodeError:
            logger.warning("Failed to parse LLM JSON response, returning raw text")
            return {"raw_response": result.text}
