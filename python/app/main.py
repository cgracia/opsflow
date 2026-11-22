import asyncio
import logging
from contextlib import asynccontextmanager
from pathlib import Path

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from prometheus_fastapi_instrumentator import Instrumentator

from app.api.router import api_router

logger = logging.getLogger(__name__)

_ROOT = Path(__file__).resolve().parent.parent  # python/


def run_migrations() -> None:
    """Apply Alembic migrations to head.

    Called from a worker thread (via asyncio.to_thread) because the async
    Alembic environment uses asyncio.run internally, which cannot execute
    inside the already-running event loop. A fresh clone therefore gets its
    schema created on API startup — ``POST /seed`` no longer depends on a
    manual step that nobody ran.
    """
    from alembic.config import Config

    from alembic import command

    config = Config(_ROOT / "alembic.ini")
    config.set_main_option("script_location", str(_ROOT / "alembic"))
    command.upgrade(config, "head")


@asynccontextmanager
async def lifespan(app: FastAPI):
    await asyncio.to_thread(run_migrations)
    yield


def create_app() -> FastAPI:
    app = FastAPI(
        title="OpsFlow Engine",
        version="0.1.0",
        description="AI-native operational investigation and orchestration engine",
        docs_url="/docs",
        redoc_url="/redoc",
        lifespan=lifespan,
    )

    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    app.include_router(api_router, prefix="/api/v1")

    _ = Instrumentator().instrument(app).expose(app)

    return app


app = create_app()
