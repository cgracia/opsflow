"""Unit tests for database-backed entity resolution (aiosqlite, no services)."""

import pytest
import pytest_asyncio
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine

from app.db.base import Base
from app.orchestrator.resolution import (
    ConflictingSignalsError,
    UnknownSignalsError,
    resolve_from_signals,
)
from app.schemas.investigation import SignalIds
from app.seed.entities import seed_all


@pytest_asyncio.fixture
async def session():
    engine = create_async_engine("sqlite+aiosqlite:///:memory:")
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
    factory = async_sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)
    async with factory() as sess:
        await seed_all(sess)
        yield sess
    await engine.dispose()


async def test_resolves_scenario_a_from_all_three_signals(session: AsyncSession):
    result = await resolve_from_signals(
        session,
        SignalIds(ticket_id="TCK-1001", alert_id="ALT-2001", event_id="EVT-3001"),
    )

    ctx = result.entity_context
    assert ctx.account["id"] == "ACC-1001"
    assert ctx.account["name"] == "Meridian Logistics"
    assert ctx.site["name"] == "Portland Distribution Center"
    assert ctx.fleet["id"] == "FLT-101"
    # Affected devices come from ticket + event FKs and alert metadata.
    assert [d["id"] for d in ctx.devices] == ["DEV-401", "DEV-402", "DEV-403"]
    assert ctx.deployment["id"] == "DEPL-501"
    assert ctx.software_revision["version"] == "3.3.0"
    assert result.severity == "high"
    assert "navigation" in result.query_text.lower()


async def test_resolves_scenario_c_distinctly(session: AsyncSession):
    result = await resolve_from_signals(
        session, SignalIds(ticket_id="TCK-3001", alert_id="ALT-4002")
    )

    ctx = result.entity_context
    assert ctx.account["id"] == "ACC-3001"
    assert ctx.site["name"] == "Atlas Fulfillment Hub"
    assert [d["id"] for d in ctx.devices] == ["DEV-801", "DEV-802", "DEV-803"]
    assert ctx.deployment["id"] == "DEPL-701"
    assert result.severity == "medium"


async def test_unknown_signal_ids_raise(session: AsyncSession):
    """v1 returned a hard-coded incident here; unknown IDs must raise instead."""
    with pytest.raises(UnknownSignalsError) as exc_info:
        await resolve_from_signals(
            session, SignalIds(ticket_id="NOPE-1", alert_id="NOPE-2", event_id="EVT-3001")
        )

    assert exc_info.value.unknown == {"ticket_id": "NOPE-1", "alert_id": "NOPE-2"}


async def test_signals_spanning_accounts_conflict(session: AsyncSession):
    with pytest.raises(ConflictingSignalsError):
        await resolve_from_signals(session, SignalIds(ticket_id="TCK-1001", event_id="EVT-4002"))


async def test_incident_severity_contributes(session: AsyncSession):
    """Severity is the max across incident, ticket priority and event severity."""
    result = await resolve_from_signals(session, SignalIds(event_id="ALT-2001"))
    assert result.severity == "high"
