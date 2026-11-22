"""Scenario B — Northwind Cold Chain: weak evidence, low severity.

A deliberately under-evidenced situation: a low-priority ticket about
temperature sensor variance and one vague event. No telemetry snapshots, no
deployment adjacency. The pipeline must report insufficient evidence, and
governance must restrict outputs (no root-cause claims, no customer draft).

All IDs are deterministic and disjoint from the other scenarios.
"""

from app.models.account import Account
from app.models.device import Device
from app.models.fleet import Fleet
from app.models.incident import Incident
from app.models.operational_event import OperationalEvent
from app.models.service import Service
from app.models.site import Site
from app.models.software_revision import SoftwareRevision
from app.models.ticket import Ticket
from app.seed.entities import NOW, T_MINUS_30M, THREE_MONTHS_AGO


def build() -> dict:
    """Return all entities for the Northwind scenario, keyed by category."""
    account = Account(
        id="ACC-2001",
        name="Northwind Cold Chain",
        tier="standard",
        region="us-west",
    )
    site = Site(
        id="SITE-3001",
        account_id=account.id,
        name="Reno Cold Storage",
        location="Reno, NV",
        timezone="America/Los_Angeles",
    )
    revisions = {
        "v205": SoftwareRevision(
            id="SWREV-401",
            version="2.0.5",
            release_notes="Stable. Sensor calibration improvements.",
            deployed_at=THREE_MONTHS_AGO,
        )
    }
    fleet = Fleet(
        id="FLT-301",
        site_id=site.id,
        account_id=account.id,
        name="Freezer Row Fleet",
        fleet_type="refrigerated_mobile_devices",
    )
    devices = {}
    for dev_id, serial, status in [
        ("DEV-701", "NCC-AM-00701", "degraded"),
        ("DEV-702", "NCC-AM-00702", "active"),
        ("DEV-703", "NCC-AM-00703", "active"),
    ]:
        devices[dev_id] = Device(
            id=dev_id,
            fleet_id=fleet.id,
            site_id=site.id,
            account_id=account.id,
            device_serial=serial,
            device_type="refrigerated_mobile_device",
            software_revision_id=revisions["v205"].id,
            status=status,
            last_seen_at=NOW,
        )
    incident = Incident(
        id="INC-6001",
        account_id=account.id,
        severity="low",
        status="open",
        title="Intermittent temperature sensor drift — Freezer Row",
        description="Occasional temperature reading spikes on one freezer unit. "
        "Cause unclear; no customer impact reported.",
        detected_at=T_MINUS_30M,
        resolved_at=None,
    )
    ticket = Ticket(
        id="TCK-2001",
        account_id=account.id,
        site_id=site.id,
        device_id=devices["DEV-701"].id,
        incident_id=incident.id,
        subject="Occasional temperature reading spikes on freezer unit",
        body="We sometimes see the temperature readout on unit 701 jump for a "
        "minute or two and then go back to normal. It has happened a handful "
        "of times this week. Not urgent, just flagging it.",
        priority="low",
        channel="portal",
        status="open",
    )
    events = {
        "sensor_drift": OperationalEvent(
            id="EVT-4001",
            device_id=devices["DEV-701"].id,
            fleet_id=fleet.id,
            site_id=site.id,
            account_id=account.id,
            event_type="sensor_drift_suspected",
            severity="low",
            description="Temperature sensor variance slightly above baseline on DEV-701. "
            "Cause unclear; no customer impact reported.",
            event_metadata={"metric": "temperature_variance", "cause": "unknown"},
            detected_at=T_MINUS_30M,
        )
    }
    services = {
        "monitoring": Service(
            id="SVC-701", name="Sensor Monitoring Service", version="1.8.0", status="active"
        )
    }
    return {
        "accounts": {"northwind": account},
        "sites": {"reno": site},
        "revisions": revisions,
        "fleets": {"freezer_row": fleet},
        "devices": devices,
        "deployments": {},
        "services": services,
        "incidents": {"drift": incident},
        "tickets": {"drift": ticket},
        "events": events,
    }
