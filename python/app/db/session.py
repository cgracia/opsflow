from collections.abc import AsyncGenerator

from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine

from app.config import Settings, get_settings


def _create_engine(settings: Settings):
    """Create async SQLAlchemy engine from settings."""
    return create_async_engine(
        settings.database_url,
        echo=settings.env == "dev",
        future=True,
    )


def _create_session_factory(engine):
    """Create async session factory."""
    return async_sessionmaker(
        engine,
        class_=AsyncSession,
        expire_on_commit=False,
    )


# Module-level references — initialized lazily via init_db()
async_engine = None
AsyncSessionLocal = None


def init_db(settings: Settings | None = None) -> None:
    """Initialize database engine and session factory."""
    global async_engine, AsyncSessionLocal
    if settings is None:
        settings = get_settings()
    async_engine = _create_engine(settings)
    AsyncSessionLocal = _create_session_factory(async_engine)


async def get_db() -> AsyncGenerator[AsyncSession, None]:
    """FastAPI dependency that yields an async database session."""
    if AsyncSessionLocal is None:
        init_db()
    async with AsyncSessionLocal() as session:
        try:
            yield session
            await session.commit()
        except Exception:
            await session.rollback()
            raise
