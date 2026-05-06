from sqlalchemy import inspect
from sqlalchemy.dialects.sqlite import JSON
from sqlalchemy.orm import RelationshipProperty

from app.models import (
    Account,
    Deployment,
    Device,
    Fleet,
    Incident,
    OperationalEvent,
    Service,
    Site,
    SoftwareRevision,
    Ticket,
)

ALL_MODELS = [
    Account,
    Deployment,
    Device,
    Fleet,
    Incident,
    OperationalEvent,
    Service,
    Site,
    SoftwareRevision,
    Ticket,
]


def test_all_models_importable():
    assert len(ALL_MODELS) == 10
    for model in ALL_MODELS:
        assert model.__name__ is not None


def test_all_models_have_tablename():
    expected = {
        "Account": "accounts",
        "Site": "sites",
        "Fleet": "fleets",
        "Device": "devices",
        "Deployment": "deployments",
        "Service": "services",
        "SoftwareRevision": "software_revisions",
        "Incident": "incidents",
        "Ticket": "tickets",
        "OperationalEvent": "operational_events",
    }
    for model in ALL_MODELS:
        assert model.__tablename__ == expected[model.__name__], (
            f"{model.__name__}.__tablename__ expected {expected[model.__name__]}"
        )


def test_account_has_required_fields():
    mapper = inspect(Account)
    col_names = {c.key for c in mapper.columns}
    for field in {"id", "name", "tier", "region", "created_at", "updated_at"}:
        assert field in col_names, f"Account missing column '{field}'"


def test_device_has_foreign_keys():
    mapper = inspect(Device)
    fk_col_names = set()
    for c in mapper.columns:
        if c.foreign_keys:
            fk_col_names.add(c.key)
    assert {"fleet_id", "site_id", "account_id", "software_revision_id"}.issubset(fk_col_names)


def test_relationship_account_sites():
    mapper = inspect(Account)
    rel_names = {r.key for r in mapper.relationships}
    assert "sites" in rel_names
    sites_rel = mapper.relationships["sites"]
    assert isinstance(sites_rel, RelationshipProperty)


def test_relationship_account_sites_fleets_devices():
    assert hasattr(Account, "sites")
    assert hasattr(Site, "fleets")
    assert hasattr(Fleet, "devices")
    site_mapper = inspect(Site)
    site_rels = {r.key for r in site_mapper.relationships}
    assert "account" in site_rels
    fleet_mapper = inspect(Fleet)
    fleet_rels = {r.key for r in fleet_mapper.relationships}
    assert "site" in fleet_rels
    device_mapper = inspect(Device)
    device_rels = {r.key for r in device_mapper.relationships}
    assert "fleet" in device_rels


def test_ticket_incident_nullable():
    mapper = inspect(Ticket)
    incident_col = mapper.columns["incident_id"]
    assert incident_col.nullable is True


def test_operational_event_metadata_json():
    mapper = inspect(OperationalEvent)
    metadata_col = mapper.columns["event_metadata"]
    assert isinstance(metadata_col.type, JSON)


def test_deployment_timestamps():
    mapper = inspect(Deployment)
    col_names = {c.key for c in mapper.columns}
    assert "started_at" in col_names
    assert "completed_at" in col_names
