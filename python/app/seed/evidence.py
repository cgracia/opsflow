"""
Evidence data for Qdrant indexing.
These are the documents that will be vectorized and searchable.
"""

from datetime import datetime, timedelta, timezone

NOW = datetime(2026, 5, 6, 14, 0, 0, tzinfo=timezone.utc)
THREE_MONTHS_AGO = NOW - timedelta(days=90)
T_MINUS_2H = NOW - timedelta(hours=2)


def get_historical_tickets() -> list[dict]:
    """5 historical tickets about similar issues."""
    return [
        {
            "id": "HTKT-001",
            "entity_id": "ACC-1001",
            "entity_type": "account",
            "source_type": "historical_ticket",
            "content": "Navigation path planning failures after v3.1.2 update on Warehouse Alpha fleet. "
            "Symptoms: devices reporting NAV_PATH_PLAN_FAILED error. Resolution: rollback to "
            "v3.1.1 and patch navigation config. Root cause: sensor fusion timeout parameter "
            "too aggressive for warehouse environments with metal shelving.",
            "timestamp": THREE_MONTHS_AGO.isoformat(),
            "metadata": {"version": "3.1.2", "resolution": "rollback"},
        },
        {
            "id": "HTKT-002",
            "entity_id": "ACC-1001",
            "entity_type": "account",
            "source_type": "historical_ticket",
            "content": "Intermittent navigation errors on DEV-401 and DEV-402. Devices losing position "
            "estimation in high-traffic warehouse corridors. Related to sensor fusion latency.",
            "timestamp": (THREE_MONTHS_AGO + timedelta(days=7)).isoformat(),
            "metadata": {"devices": ["DEV-401", "DEV-402"]},
        },
        {
            "id": "HTKT-003",
            "entity_id": "ACC-1001",
            "entity_type": "account",
            "source_type": "historical_ticket",
            "content": "Fleet-wide navigation degradation reported after software update. Error rate "
            "spike from baseline 0.2% to 15%. Affected 4 of 8 devices. Rollback resolved.",
            "timestamp": (THREE_MONTHS_AGO + timedelta(days=14)).isoformat(),
            "metadata": {"pattern": "post_update_navigation_failure"},
        },
        {
            "id": "HTKT-004",
            "entity_id": "FLT-101",
            "entity_type": "fleet",
            "source_type": "historical_ticket",
            "content": "Path planning module crash when encountering unexpected obstacles near loading "
            "docks. Workaround: increased obstacle detection radius. Permanent fix in v3.2.0.",
            "timestamp": (THREE_MONTHS_AGO + timedelta(days=30)).isoformat(),
            "metadata": {"area": "loading_docks"},
        },
        {
            "id": "HTKT-005",
            "entity_id": "ACC-1001",
            "entity_type": "account",
            "source_type": "historical_ticket",
            "content": "Customer reported shipment delays due to device navigation issues at Portland DC. "
            "SLA impact: 3 missed delivery windows. Root cause: navigation stack memory leak "
            "causing gradual degradation over 48-hour uptime.",
            "timestamp": (THREE_MONTHS_AGO + timedelta(days=45)).isoformat(),
            "metadata": {"impact": "sla_breach", "root_cause": "memory_leak"},
        },
    ]


def get_runbooks() -> list[dict]:
    """3 runbook documents for operational reference."""
    return [
        {
            "id": "RB-001",
            "entity_id": "FLT-101",
            "entity_type": "fleet",
            "source_type": "runbook",
            "content": "Runbook: Navigation Troubleshooting\n"
            "1. Check device software version (cmd: device-info --version)\n"
            "2. Review navigation error logs (cmd: logs --filter=nav --last=1h)\n"
            "3. Verify sensor fusion status (cmd: sensor-status --all)\n"
            "4. If error rate > 5%, check for recent deployments\n"
            "5. If post-deployment, consider rollback to previous version\n"
            "6. Escalate to navigation team if rollback doesn't resolve\n"
            "Common causes: sensor fusion timeout, path planning parameter mismatch, "
            "map data corruption after update.",
            "timestamp": NOW.isoformat(),
            "metadata": {"type": "runbook", "category": "navigation"},
        },
        {
            "id": "RB-002",
            "entity_id": "FLT-101",
            "entity_type": "fleet",
            "source_type": "runbook",
            "content": "Runbook: Fleet-Wide Software Rollback\n"
            "1. Confirm affected devices and software version\n"
            "2. Halt deployment if in progress\n"
            "3. Rollback command: fleet-deploy --rollback --fleet=FLT-101 --to=PREVIOUS\n"
            "4. Monitor error rates for 30 minutes post-rollback\n"
            "5. Verify all devices report active status\n"
            "6. Document incident and update deployment checklist",
            "timestamp": NOW.isoformat(),
            "metadata": {"type": "runbook", "category": "deployment"},
        },
        {
            "id": "RB-003",
            "entity_id": "ACC-1001",
            "entity_type": "account",
            "source_type": "runbook",
            "content": "Runbook: Device Blocked Response Protocol\n"
            "1. Acknowledge device blocked event within 5 minutes\n"
            "2. Check device telemetry for error patterns\n"
            "3. If single device: attempt remote restart\n"
            "4. If multiple devices: investigate fleet-wide cause\n"
            "5. Notify site operations manager if shipment impact expected\n"
            "6. For enterprise accounts: prepare customer briefing within 30 minutes",
            "timestamp": NOW.isoformat(),
            "metadata": {"type": "runbook", "category": "blocked_device"},
        },
    ]


def get_telemetry_snapshots() -> list[dict]:
    """2 telemetry snapshots showing the anomaly."""
    return [
        {
            "id": "TEL-001",
            "entity_id": "DEV-401",
            "entity_type": "device",
            "source_type": "telemetry",
            "content": "Telemetry snapshot DEV-401 (last 4 hours):\n"
            "- navigation_error_rate: 47.3% (baseline: 0.2%, threshold: 5%)\n"
            "- path_planning_failures: 142 (was 0-2 per hour)\n"
            "- sensor_fusion_latency_ms: 234ms (baseline: 45ms)\n"
            "- position_estimation_accuracy: 2.1m (baseline: 0.3m)\n"
            "- battery_level: 78%\n"
            "- uptime_since_restart: 2h 15m\n"
            "- last_successful_navigation: 13:45 UTC\n"
            "ANOMALY: Navigation error rate began climbing at 13:30 UTC, "
            "coinciding with v3.3.0 deployment reaching this device.",
            "timestamp": NOW.isoformat(),
            "metadata": {
                "metric": "navigation_error_rate",
                "baseline": 0.002,
                "current": 0.473,
            },
        },
        {
            "id": "TEL-002",
            "entity_id": "FLT-101",
            "entity_type": "fleet",
            "source_type": "telemetry",
            "content": "Fleet telemetry summary FLT-101 (last 4 hours):\n"
            "- Fleet navigation_error_rate: 18.2% (3/8 devices affected)\n"
            "- Affected devices: DEV-401 (47.3%), DEV-402 (12.1%), DEV-403 (8.7%)\n"
            "- Healthy devices: DEV-404-408 (all <0.5%, still on v3.2.1)\n"
            "- Pattern: Only devices updated to v3.3.0 showing errors\n"
            "- Deployment DEPL-501 started at 12:00 UTC, reached devices at 13:25 UTC\n"
            "CORRELATION: Error onset directly correlates with v3.3.0 activation per device.",
            "timestamp": NOW.isoformat(),
            "metadata": {
                "metric": "fleet_navigation_error_rate",
                "affected_count": 3,
                "total_count": 8,
            },
        },
    ]


def get_deployment_manifest() -> list[dict]:
    """1 deployment manifest."""
    return [
        {
            "id": "DPM-001",
            "entity_id": "DEPL-501",
            "entity_type": "deployment",
            "source_type": "deployment",
            "content": "Deployment manifest DEPL-501:\n"
            "- Target: Fleet FLT-101 (Warehouse Alpha)\n"
            "- Software: v3.3.0 (SWREV-302)\n"
            "- Previous: v3.2.1 (SWREV-301)\n"
            "- Started: 2026-05-06 12:00 UTC\n"
            "- Strategy: Rolling update, 2 devices at a time\n"
            "- Phase 1: DEV-401, DEV-402 updated at 13:25 UTC\n"
            "- Phase 2: DEV-403 updated at 13:28 UTC\n"
            "- Phase 3: DEV-404, DEV-405 (pending, halted due to errors)\n"
            "- Status: HALTED — navigation errors detected on updated devices\n"
            "Changes in v3.3.0: New navigation engine, updated sensor fusion parameters, "
            "revised path planning algorithm.",
            "timestamp": T_MINUS_2H.isoformat(),
            "metadata": {
                "version_from": "3.2.1",
                "version_to": "3.3.0",
                "strategy": "rolling",
                "status": "halted",
            },
        },
    ]


def get_all_evidence() -> list[dict]:
    """Return all evidence documents for Qdrant indexing."""
    return (
        get_historical_tickets()
        + get_runbooks()
        + get_telemetry_snapshots()
        + get_deployment_manifest()
    )
