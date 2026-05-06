from sqlalchemy.ext.asyncio import AsyncSession

from app.seed.entities import seed_all


async def seed_database(session: AsyncSession) -> dict:
    """Main entry point: seed all entities and return entity map."""
    return await seed_all(session)
