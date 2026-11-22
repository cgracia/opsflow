import pytest_asyncio
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine

from app.db.base import Base
from app.seed.entities import seed_all
from app.seed.evidence import (
    get_all_evidence,
    get_deployment_manifest,
    get_historical_tickets,
    get_runbooks,
    get_telemetry_snapshots,
)
from app.seed.main import seed_database


@pytest_asyncio.fixture
async def db_session():
    engine = create_async_engine("sqlite+aiosqlite:///:memory:")
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
    session_factory = async_sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)
    async with session_factory() as session, session.begin():
        yield session
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.drop_all)
    await engine.dispose()


async def test_entity_counts(db_session: AsyncSession):
    result = await seed_all(db_session)

    assert len(result["accounts"]) == 3
    assert len(result["sites"]) == 3
    assert len(result["revisions"]) == 5
    assert len(result["fleets"]) == 3
    assert len(result["devices"]) == 16
    assert len(result["deployments"]) == 2
    assert len(result["services"]) == 3
    assert len(result["incidents"]) == 3
    assert len(result["tickets"]) == 3
    assert len(result["events"]) == 5


async def test_scenarios_are_disjoint(db_session: AsyncSession):
    """The three fixture scenarios must not share any entity."""
    result = await seed_all(db_session)

    account_ids = {a.account_id for a in result["sites"].values()}
    assert len(account_ids) == 3
    assert len({f.account_id for f in result["fleets"].values()}) == 3
    assert len({t.account_id for t in result["tickets"].values()}) == 3
    # Every entity key in the merged maps is the entity's own ID.
    for category, entities in result.items():
        for key, entity in entities.items():
            assert key == entity.id, f"{category}: key {key} != id {entity.id}"


async def test_ids_are_deterministic(db_session: AsyncSession):
    from app.seed import entities

    original_now = entities.NOW

    result1 = await seed_all(db_session)
    ids1 = {k: sorted(e.id for e in v.values()) for k, v in result1.items()}

    result2 = await seed_all(db_session)
    ids2 = {k: sorted(e.id for e in v.values()) for k, v in result2.items()}

    assert ids1 == ids2
    assert original_now == entities.NOW


async def test_narrative_consistency(db_session: AsyncSession):
    result = await seed_all(db_session)

    v330_id = result["revisions"]["SWREV-302"].id
    v321_id = result["revisions"]["SWREV-301"].id

    affected = ["DEV-401", "DEV-402", "DEV-403"]
    healthy = ["DEV-404", "DEV-405", "DEV-406", "DEV-407", "DEV-408"]

    for dev_id in affected:
        device = result["devices"][dev_id]
        assert device.software_revision_id == v330_id, f"{dev_id} should be on v3.3.0"
        assert device.status in ("error", "degraded"), f"{dev_id} should be error/degraded"

    for dev_id in healthy:
        device = result["devices"][dev_id]
        assert device.software_revision_id == v321_id, f"{dev_id} should be on v3.2.1"
        assert device.status == "active", f"{dev_id} should be active"

    deployment = result["deployments"]["DEPL-501"]
    anomaly_event = result["events"]["ALT-2001"]
    blocked_event = result["events"]["EVT-3001"]
    incident = result["incidents"]["INC-5001"]
    ticket = result["tickets"]["TCK-1001"]

    assert deployment.started_at < anomaly_event.detected_at
    assert deployment.started_at < blocked_event.detected_at
    assert deployment.started_at < incident.detected_at

    assert ticket.device_id == "DEV-401"
    assert ticket.incident_id == incident.id

    assert blocked_event.device_id == "DEV-401"

    assert deployment.software_revision_id == v330_id


def test_evidence_documents_count():
    evidence = get_all_evidence()
    assert len(evidence) == 19

    assert len(get_historical_tickets()) == 5
    assert len(get_runbooks()) == 3
    assert len(get_telemetry_snapshots()) == 2
    assert len(get_deployment_manifest()) == 1
    from app.seed.evidence import get_atlas_evidence, get_northwind_evidence

    assert len(get_northwind_evidence()) == 2
    assert len(get_atlas_evidence()) == 6


def test_evidence_ids_unique_across_scenarios():
    evidence = get_all_evidence()
    ids = [doc["id"] for doc in evidence]
    assert len(ids) == len(set(ids))


def test_historical_tickets_exist():
    tickets = get_historical_tickets()
    assert len(tickets) == 5

    required_keys = {
        "id",
        "entity_id",
        "entity_type",
        "source_type",
        "content",
        "timestamp",
        "metadata",
    }
    for ticket in tickets:
        assert required_keys.issubset(ticket.keys()), f"Ticket {ticket['id']} missing keys"
        assert ticket["source_type"] == "historical_ticket"
        assert isinstance(ticket["content"], str)
        assert len(ticket["content"]) > 0


async def test_seed_database_entry_point(db_session: AsyncSession):
    result = await seed_database(db_session)
    assert "accounts" in result
    assert "devices" in result
    assert result["accounts"]["ACC-1001"].name == "Meridian Logistics"
