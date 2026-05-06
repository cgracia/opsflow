"""
Synthetic data seeder for the Meridian Logistics incident narrative.

Scenario: Fleet-wide deployment of software v3.3.0 causes navigation errors
across 3 devices at Portland Distribution Center. A support ticket, anomaly
alert, and blocked-device event all arrive within minutes — the system must
recognize they're the same incident.

All IDs are deterministic for test reproducibility.
"""

from datetime import datetime, timedelta, timezone

from sqlalchemy.ext.asyncio import AsyncSession

from app.models.account import Account
from app.models.deployment import Deployment
from app.models.device import Device
from app.models.fleet import Fleet
from app.models.incident import Incident
from app.models.operational_event import OperationalEvent
from app.models.service import Service
from app.models.site import Site
from app.models.software_revision import SoftwareRevision
from app.models.ticket import Ticket


# Reference timestamps (all UTC)
NOW = datetime(2026, 5, 6, 14, 0, 0, tzinfo=timezone.utc)
T_MINUS_2H = NOW - timedelta(hours=2)
T_MINUS_1H = NOW - timedelta(hours=1)
T_MINUS_30M = NOW - timedelta(minutes=30)
T_MINUS_15M = NOW - timedelta(minutes=15)
T_MINUS_5M = NOW - timedelta(minutes=5)
THREE_MONTHS_AGO = NOW - timedelta(days=90)


async def seed_accounts() -> dict:
    return {
        "meridian": Account(
            id="ACC-1001",
            name="Meridian Logistics",
            tier="enterprise",
            region="us-west",
        )
    }


async def seed_sites(accounts: dict) -> dict:
    return {
        "portland": Site(
            id="SITE-2001",
            account_id=accounts["meridian"].id,
            name="Portland Distribution Center",
            location="Portland, OR",
            timezone="America/Los_Angeles",
        )
    }


async def seed_software_revisions() -> dict:
    return {
        "v321": SoftwareRevision(
            id="SWREV-301",
            version="3.2.1",
            release_notes="Stable release. Navigation stack improvements. Bug fixes for mapping edge cases.",
            deployed_at=THREE_MONTHS_AGO - timedelta(days=60),
        ),
        "v330": SoftwareRevision(
            id="SWREV-302",
            version="3.3.0",
            release_notes="Major update. New navigation engine. Improved path planning. Updated sensor fusion.",
            deployed_at=T_MINUS_2H,
        ),
    }


async def seed_fleets(accounts: dict, sites: dict) -> dict:
    return {
        "warehouse_alpha": Fleet(
            id="FLT-101",
            site_id=sites["portland"].id,
            account_id=accounts["meridian"].id,
            name="Warehouse Alpha Fleet",
            fleet_type="autonomous_mobile_devices",
        )
    }


async def seed_devices(fleets: dict, sites: dict, accounts: dict, revisions: dict) -> dict:
    """8 devices: 3 affected (v3.3.0), 5 healthy (v3.2.1)."""
    devices = {}

    # Affected devices — running v3.3.0
    for idx, (dev_id, serial) in enumerate(
        [
            ("DEV-401", "AMD-AM-00401"),
            ("DEV-402", "AMD-AM-00402"),
            ("DEV-403", "AMD-AM-00403"),
        ]
    ):
        devices[dev_id] = Device(
            id=dev_id,
            fleet_id=fleets["warehouse_alpha"].id,
            site_id=sites["portland"].id,
            account_id=accounts["meridian"].id,
            device_serial=serial,
            device_type="autonomous_mobile_device",
            software_revision_id=revisions["v330"].id,
            status="error" if idx == 0 else "degraded",
            last_seen_at=T_MINUS_5M if idx == 0 else T_MINUS_15M,
        )

    # Healthy devices — still on v3.2.1
    for dev_id, serial in [
        ("DEV-404", "AMD-AM-00404"),
        ("DEV-405", "AMD-AM-00405"),
        ("DEV-406", "AMD-AM-00406"),
        ("DEV-407", "AMD-AM-00407"),
        ("DEV-408", "AMD-AM-00408"),
    ]:
        devices[dev_id] = Device(
            id=dev_id,
            fleet_id=fleets["warehouse_alpha"].id,
            site_id=sites["portland"].id,
            account_id=accounts["meridian"].id,
            device_serial=serial,
            device_type="autonomous_mobile_device",
            software_revision_id=revisions["v321"].id,
            status="active",
            last_seen_at=NOW,
        )

    return devices


async def seed_deployments(fleets: dict, revisions: dict) -> dict:
    return {
        "current": Deployment(
            id="DEPL-501",
            fleet_id=fleets["warehouse_alpha"].id,
            software_revision_id=revisions["v330"].id,
            status="in_progress",
            started_at=T_MINUS_2H,
            completed_at=None,
        )
    }


async def seed_services() -> dict:
    return {
        "navigation": Service(
            id="SVC-601",
            name="Navigation Service",
            version="3.3.0",
            status="degraded",
        )
    }


async def seed_incidents(accounts: dict) -> dict:
    return {
        "current": Incident(
            id="INC-5001",
            account_id=accounts["meridian"].id,
            severity="high",
            status="open",
            title="Multi-device navigation failure — Warehouse Alpha Fleet",
            description="Three devices in Warehouse Alpha Fleet experiencing navigation errors "
            "following v3.3.0 deployment. Devices DEV-401, DEV-402, DEV-403 affected.",
            detected_at=T_MINUS_5M,
            resolved_at=None,
        )
    }


async def seed_tickets(accounts: dict, sites: dict, devices: dict, incidents: dict) -> dict:
    return {
        "current": Ticket(
            id="TCK-1001",
            account_id=accounts["meridian"].id,
            site_id=sites["portland"].id,
            device_id=devices["DEV-401"].id,
            incident_id=incidents["current"].id,
            subject="3 devices stopped navigating after update",
            body="Hi, we have 3 devices (AMD-AM-00401, 00402, 00403) that started showing "
            "navigation errors about 30 minutes ago. They're all in Warehouse Alpha. "
            "The devices keep reporting 'path planning failed' and two of them are now "
            "stationary. We had a fleet-wide update roll out about 2 hours ago. "
            "Please investigate urgently — this is blocking our afternoon shipments.",
            priority="high",
            channel="email",
            status="open",
        )
    }


async def seed_operational_events(devices: dict, fleets: dict, sites: dict, accounts: dict) -> dict:
    return {
        "blocked_device": OperationalEvent(
            id="EVT-3001",
            device_id=devices["DEV-401"].id,
            fleet_id=fleets["warehouse_alpha"].id,
            site_id=sites["portland"].id,
            account_id=accounts["meridian"].id,
            event_type="device_blocked",
            severity="high",
            description="Device DEV-401 blocked: navigation path planning failure. "
            "Device stationary at warehouse bay 7. Last successful navigation at 13:45 UTC.",
            event_metadata={
                "error_code": "NAV_PATH_PLAN_FAILED",
                "location": "bay_7",
                "last_successful_nav": T_MINUS_15M.isoformat(),
                "sensor_status": "nominal",
                "battery_level": 78,
            },
            detected_at=T_MINUS_5M,
        ),
        "anomaly_alert": OperationalEvent(
            id="ALT-2001",
            device_id=None,
            fleet_id=fleets["warehouse_alpha"].id,
            site_id=sites["portland"].id,
            account_id=accounts["meridian"].id,
            event_type="anomaly_detected",
            severity="high",
            description="Anomaly alert: navigation_error_rate spike in Warehouse Alpha Fleet. "
            "Error rate increased from 0.2% to 47.3% in the last 30 minutes. "
            "3 of 8 devices affected. Correlation with deployment DEPL-501 (v3.3.0).",
            event_metadata={
                "metric": "navigation_error_rate",
                "baseline_value": 0.002,
                "current_value": 0.473,
                "threshold": 0.05,
                "affected_devices": ["DEV-401", "DEV-402", "DEV-403"],
                "correlated_deployment": "DEPL-501",
            },
            detected_at=T_MINUS_30M,
        ),
    }


async def seed_all(session: AsyncSession) -> dict:
    """Seed all entities and return a dict of all created entities by category.

    Clears existing data first (idempotent).
    """
    import sqlalchemy

    # Clear existing data (reverse dependency order)
    for table in [
        "operational_events",
        "tickets",
        "incidents",
        "deployments",
        "devices",
        "fleets",
        "services",
        "software_revisions",
        "sites",
        "accounts",
    ]:
        await session.execute(sqlalchemy.text(f"DELETE FROM {table}"))

    # Seed in dependency order
    accounts = await seed_accounts()
    sites = await seed_sites(accounts)
    revisions = await seed_software_revisions()
    fleets = await seed_fleets(accounts, sites)
    devices = await seed_devices(fleets, sites, accounts, revisions)
    deployments = await seed_deployments(fleets, revisions)
    services = await seed_services()
    incidents = await seed_incidents(accounts)
    tickets = await seed_tickets(accounts, sites, devices, incidents)
    events = await seed_operational_events(devices, fleets, sites, accounts)

    # Persist all
    all_entities = (
        list(accounts.values())
        + list(sites.values())
        + list(revisions.values())
        + list(fleets.values())
        + list(devices.values())
        + list(deployments.values())
        + list(services.values())
        + list(incidents.values())
        + list(tickets.values())
        + list(events.values())
    )
    for entity in all_entities:
        session.add(entity)

    await session.flush()

    return {
        "accounts": accounts,
        "sites": sites,
        "revisions": revisions,
        "fleets": fleets,
        "devices": devices,
        "deployments": deployments,
        "services": services,
        "incidents": incidents,
        "tickets": tickets,
        "events": events,
    }
