from datetime import datetime

from pydantic import BaseModel, ConfigDict


class AccountOut(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    name: str
    tier: str
    region: str
    created_at: datetime


class SiteOut(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    account_id: str
    name: str
    location: str | None = None
    timezone: str
    created_at: datetime


class FleetOut(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    site_id: str
    account_id: str
    name: str
    fleet_type: str
    created_at: datetime


class DeviceOut(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    fleet_id: str
    site_id: str
    account_id: str
    device_serial: str
    device_type: str
    software_revision_id: str | None = None
    status: str
    last_seen_at: datetime | None = None
    created_at: datetime


class DeploymentOut(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    fleet_id: str
    software_revision_id: str
    status: str
    started_at: datetime | None = None
    completed_at: datetime | None = None
    created_at: datetime


class ServiceOut(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    name: str
    version: str
    status: str
    created_at: datetime


class SoftwareRevisionOut(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    version: str
    release_notes: str | None = None
    deployed_at: datetime | None = None
    created_at: datetime


class IncidentOut(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    account_id: str
    severity: str
    status: str
    title: str
    description: str | None = None
    detected_at: datetime | None = None
    resolved_at: datetime | None = None
    created_at: datetime


class TicketOut(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    account_id: str
    site_id: str | None = None
    device_id: str | None = None
    incident_id: str | None = None
    subject: str
    body: str | None = None
    priority: str
    channel: str
    status: str
    created_at: datetime


class OperationalEventOut(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: str
    device_id: str | None = None
    fleet_id: str | None = None
    site_id: str | None = None
    account_id: str | None = None
    event_type: str
    severity: str
    description: str | None = None
    event_metadata: dict | None = None
    detected_at: datetime | None = None
    created_at: datetime
