"""CLI entry point for seeding the database.

Usage:
    cd python && uv run python -m app.seed [--db-url DB_URL]

Defaults to sqlite+aiosqlite:///./opsflow.db if no URL provided.
"""

import asyncio
import sys

from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine

from app.db.base import Base
from app.seed.evidence import get_all_evidence
from app.seed.main import seed_database


async def _seed(db_url: str) -> dict:
    """Run the seed pipeline and return entity counts."""
    engine = create_async_engine(db_url, echo=False, future=True)

    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)

    session_factory = async_sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)

    async with session_factory() as session:
        async with session.begin():
            entities = await seed_database(session)

    counts = {}
    for category, items in entities.items():
        counts[category] = len(items)
        print(f"  {category}: {len(items)}")

    evidence = get_all_evidence()
    counts["evidence"] = len(evidence)
    print(f"  evidence: {len(evidence)}")

    await engine.dispose()
    return counts


def main() -> None:
    db_url = "sqlite+aiosqlite:///./opsflow.db"
    for i, arg in enumerate(sys.argv[1:], 1):
        if arg == "--db-url" and i < len(sys.argv) - 1:
            db_url = sys.argv[i + 1]
            break

    print(f"Seeding database: {db_url}")
    try:
        counts = asyncio.run(_seed(db_url))
        print(f"\nSeed completed successfully. {sum(counts.values())} total records.")
    except Exception as exc:
        print(f"\nSeed failed: {exc}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
