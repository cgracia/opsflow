"""Scenario C — Atlas Retail Robotics: medium severity, enterprise account.

A well-evidenced post-update regression on a barcode-scanning fleet: two
telemetry snapshots with baseline deviations, historical tickets requiring
rollback, a runbook and a halted deployment manifest. Enterprise tier +
medium severity classifies as COMMUNICATE, so governance approves an
evidence-grounded customer draft (with no invented root cause or ETA).

All IDs are deterministic and disjoint from the other scenarios.
"""

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
from app.seed.entities import NOW, T_MINUS_2H, T_MINUS_15M, T_MINUS_30M, THREE_MONTHS_AGO


def build() -> dict:
    """Return all entities for the Atlas scenario, keyed by category."""
    account = Account(
        id="ACC-3001",
        name="Atlas Retail Robotics",
        tier="enterprise",
        region="us-east",
    )
    site = Site(
        id="SITE-4001",
        account_id=account.id,
        name="Atlas Fulfillment Hub",
        location="Columbus, OH",
        timezone="America/New_York",
    )
    revisions = {
        "v532": SoftwareRevision(
            id="SWREV-501",
            version="5.3.2",
            release_notes="Stable pick-line release.",
            deployed_at=THREE_MONTHS_AGO,
        ),
        "v540": SoftwareRevision(
            id="SWREV-502",
            version="5.4.0",
            release_notes="New scanner pipeline, updated exposure parameters.",
            deployed_at=T_MINUS_2H,
        ),
    }
    fleet = Fleet(
        id="FLT-401",
        site_id=site.id,
        account_id=account.id,
        name="Pick Line Fleet",
        fleet_type="autonomous_picking_robots",
    )
    devices = {}
    for idx, (dev_id, serial, rev, status) in enumerate(
        [
            ("DEV-801", "ARR-AM-00801", "v540", "error"),
            ("DEV-802", "ARR-AM-00802", "v540", "degraded"),
            ("DEV-803", "ARR-AM-00803", "v540", "degraded"),
            ("DEV-804", "ARR-AM-00804", "v532", "active"),
            ("DEV-805", "ARR-AM-00805", "v532", "active"),
        ]
    ):
        devices[dev_id] = Device(
            id=dev_id,
            fleet_id=fleet.id,
            site_id=site.id,
            account_id=account.id,
            device_serial=serial,
            device_type="autonomous_picking_robot",
            software_revision_id=revisions[rev].id,
            status=status,
            last_seen_at=T_MINUS_15M if idx < 3 else NOW,
        )
    deployment = Deployment(
        id="DEPL-701",
        fleet_id=fleet.id,
        software_revision_id=revisions["v540"].id,
        status="in_progress",
        started_at=T_MINUS_2H,
        completed_at=None,
    )
    services = {
        "scanning": Service(
            id="SVC-801", name="Barcode Scanning Service", version="5.4.0", status="degraded"
        )
    }
    incident = Incident(
        id="INC-7001",
        account_id=account.id,
        severity="medium",
        status="open",
        title="Barcode scanner misreads on pick line after v5.4.0",
        description="Three picking robots misreading barcodes following the "
        "v5.4.0 deployment. Devices DEV-801, DEV-802, DEV-803 affected.",
        detected_at=T_MINUS_15M,
        resolved_at=None,
    )
    ticket = Ticket(
        id="TCK-3001",
        account_id=account.id,
        site_id=site.id,
        device_id=devices["DEV-801"].id,
        incident_id=incident.id,
        subject="Picking robots misreading barcodes since this morning",
        body="Three of our picking robots started misreading barcodes about an "
        "hour ago, right after the morning update. Orders are queuing but "
        "slower than usual. Can you check what changed?",
        priority="medium",
        channel="email",
        status="open",
    )
    events = {
        "anomaly_alert": OperationalEvent(
            id="ALT-4002",
            device_id=None,
            fleet_id=fleet.id,
            site_id=site.id,
            account_id=account.id,
            event_type="anomaly_detected",
            severity="medium",
            description="Anomaly alert: scanner_misread_rate spike in Pick Line Fleet. "
            "Misread rate increased from 0.4% to 23.1% in the last hour. "
            "3 of 5 devices affected. Correlation with deployment DEPL-701 (v5.4.0).",
            event_metadata={
                "metric": "scanner_misread_rate",
                "baseline_value": 0.004,
                "current_value": 0.231,
                "threshold": 0.02,
                "affected_devices": ["DEV-801", "DEV-802", "DEV-803"],
                "correlated_deployment": "DEPL-701",
            },
            detected_at=T_MINUS_30M,
        ),
        "device_blocked": OperationalEvent(
            id="EVT-4002",
            device_id=devices["DEV-801"].id,
            fleet_id=fleet.id,
            site_id=site.id,
            account_id=account.id,
            event_type="pick_aborted",
            severity="medium",
            description="Device DEV-801 aborted pick: barcode scan failed 3 "
            "consecutive attempts at station 4. Robot waiting for operator.",
            event_metadata={"error_code": "SCAN_MISREAD_STREAK", "station": "4"},
            detected_at=T_MINUS_15M,
        ),
    }
    return {
        "accounts": {"atlas": account},
        "sites": {"columbus": site},
        "revisions": revisions,
        "fleets": {"pick_line": fleet},
        "devices": devices,
        "deployments": {"current": deployment},
        "services": services,
        "incidents": {"misreads": incident},
        "tickets": {"misreads": ticket},
        "events": events,
    }
