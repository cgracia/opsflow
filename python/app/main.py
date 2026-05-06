from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from prometheus_fastapi_instrumentator import Instrumentator

from app.api.router import api_router


def create_app() -> FastAPI:
    app = FastAPI(
        title="OpsFlow Engine",
        version="0.1.0",
        description="AI-native operational investigation and orchestration engine",
        docs_url="/docs",
        redoc_url="/redoc",
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
