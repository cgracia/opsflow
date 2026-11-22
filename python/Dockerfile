# Locked build: the image is installed from the committed uv.lock, never from
# fresh resolution. v1 ran `pip install .`, which resolved qdrant-client 1.20
# while uv.lock pinned 1.17 — the API crashed on import in every container.
FROM python:3.12-slim

COPY --from=ghcr.io/astral-sh/uv:0.13.7 /uv /uvx /usr/local/bin/

WORKDIR /app

# Dependencies first for layer caching; --frozen refuses to touch uv.lock.
COPY pyproject.toml uv.lock ./
RUN uv sync --frozen --no-dev --no-install-project

# Application code (imported from WORKDIR, not installed).
COPY app/ ./app/
COPY alembic/ ./alembic/
COPY alembic.ini .

ENV PATH="/app/.venv/bin:$PATH"

EXPOSE 8000

# Migrations run in the app's startup lifespan; uvicorn serves afterwards.
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"]
