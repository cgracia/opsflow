from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """Application settings loaded from environment variables or .env file."""

    # Database
    database_url: str = "sqlite+aiosqlite:///./opsflow.db"

    # Qdrant
    qdrant_url: str = "http://localhost:6333"
    qdrant_api_key: str = ""

    # Langfuse
    langfuse_public_key: str = ""
    langfuse_secret_key: str = ""
    langfuse_base_url: str = "http://localhost:3000"

    # LLM (OpenAI-compatible)
    llm_api_base: str = "https://api.openai.com/v1"
    llm_api_key: str = ""
    llm_model: str = "gpt-4o"
    llm_max_tokens: int = 2000
    llm_temperature: float = 0.3

    # OpenTelemetry
    otel_endpoint: str = "http://localhost:4317"

    # Environment
    env: str = "dev"

    model_config = {"env_file": ".env", "extra": "ignore"}


def get_settings() -> Settings:
    """Get cached settings instance."""
    return Settings()
