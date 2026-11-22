"""Phase 2: entity resolution backed by the database.

Replaces v1's hard-coded Meridian Logistics constant, which returned an
unrelated incident for any signal IDs. Signals are looked up in Postgres;
unknown IDs raise :class:`UnknownSignalsError` so the API can answer 404
instead of investigating fiction. The entity graph is then walked from the
signals' foreign keys — the hierarchy is the design, the labels are not.
"""

import logging
from dataclasses import dataclass

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.account import Account
from app.models.deployment import Deployment
from app.models.device import Device
from app.models.fleet import Fleet
from app.models.incident import Incident
from app.models.operational_event import OperationalEvent
from app.models.site import Site
from app.models.software_revision import SoftwareRevision
from app.models.ticket import Ticket
from app.schemas.investigation import EntityContext, SignalIds

logger = logging.getLogger(__name__)

SEVERITY_ORDER = {"info": 0, "low": 1, "medium": 2, "high": 3, "critical": 4}


class UnknownSignalsError(Exception):
    """One or more provided signal IDs do not exist."""

    def __init__(self, unknown: dict[str, str]):
        self.unknown = unknown
        detail = ", ".join(f"{field} '{sid}'" for field, sid in unknown.items())
        super().__init__(f"Unknown signal IDs: {detail}")


class ConflictingSignalsError(Exception):
    """Provided signals do not belong to a single account."""


@dataclass
class ResolutionResult:
    """Everything later phases need from resolution."""

    entity_context: EntityContext
    account_name: str = ""
    site_name: str = ""
    severity: str = "medium"  # highest severity across the resolved signals
    query_text: str = ""  # signal content, used to build retrieval queries


def _max_severity(severities: list[str | None]) -> str:
    ranked = [s for s in severities if s]
    if not ranked:
        return "medium"
    return max(ranked, key=lambda s: SEVERITY_ORDER.get(s.lower(), 2))


def _device_refs(ticket: Ticket | None, events: list[OperationalEvent]) -> list[str]:
    """Device IDs referenced by the signals, in arrival order, deduplicated."""
    refs: list[str] = []
    if ticket is not None and ticket.device_id:
        refs.append(ticket.device_id)
    for event in events:
        if event.device_id:
            refs.append(event.device_id)
        affected = (event.event_metadata or {}).get("affected_devices") or []
        refs.extend(d for d in affected if isinstance(d, str))
    return list(dict.fromkeys(refs))


async def _fetch_map(session: AsyncSession, model, ids: list[str]) -> dict:
    if not ids:
        return {}
    rows = (await session.execute(select(model).where(model.id.in_(ids)))).scalars().all()
    return {row.id: row for row in rows}


async def resolve_from_signals(session: AsyncSession, signal_ids: SignalIds) -> ResolutionResult:
    """Resolve signal IDs to their entity graph.

    Every provided ID must exist; otherwise ``UnknownSignalsError`` is raised.
    All signals must belong to the same account.
    """
    ticket: Ticket | None = None
    events: list[OperationalEvent] = []
    unknown: dict[str, str] = {}

    if signal_ids.ticket_id:
        ticket = await session.get(Ticket, signal_ids.ticket_id)
        if ticket is None:
            unknown["ticket_id"] = signal_ids.ticket_id
    for field, signal_id in (("alert_id", signal_ids.alert_id), ("event_id", signal_ids.event_id)):
        if signal_id:
            event = await session.get(OperationalEvent, signal_id)
            if event is None:
                unknown[field] = signal_id
            else:
                events.append(event)

    if unknown:
        raise UnknownSignalsError(unknown)

    account_ids = {
        ref
        for ref in [
            ticket.account_id if ticket else None,
            *(e.account_id for e in events),
        ]
        if ref
    }
    if len(account_ids) > 1:
        raise ConflictingSignalsError(
            f"Signals span multiple accounts: {', '.join(sorted(account_ids))}"
        )
    if not account_ids:
        raise UnknownSignalsError({"signal_ids": "no resolvable signals provided"})

    account_id = account_ids.pop()

    # Severity: the strongest signal carries, so governance sees the real risk.
    incident: Incident | None = None
    if ticket is not None and ticket.incident_id:
        incident = await session.get(Incident, ticket.incident_id)
    severity = _max_severity(
        [
            incident.severity if incident else None,
            ticket.priority if ticket else None,
            *(e.severity for e in events),
        ]
    )

    # Devices referenced by the signals.
    device_ids = _device_refs(ticket, events)
    devices = await _fetch_map(session, Device, device_ids)
    ordered_devices = [devices[d] for d in device_ids if d in devices]

    # Fleet: from an event, else from the first referenced device.
    fleet_id = next((e.fleet_id for e in events if e.fleet_id), None)
    if fleet_id is None and ordered_devices:
        fleet_id = ordered_devices[0].fleet_id
    fleet = await session.get(Fleet, fleet_id) if fleet_id else None

    # Site: from the ticket or an event, else from the fleet.
    site_id = next(
        (s for s in [ticket.site_id if ticket else None, *(e.site_id for e in events)] if s),
        None,
    )
    if site_id is None and fleet is not None:
        site_id = fleet.site_id
    site = await session.get(Site, site_id) if site_id else None

    account = await session.get(Account, account_id)

    # Most recent deployment for the fleet — in-progress preferred.
    deployment: Deployment | None = None
    if fleet is not None:
        result = await session.execute(
            select(Deployment)
            .where(Deployment.fleet_id == fleet.id)
            .order_by(Deployment.started_at.desc().nullslast())
        )
        candidates = result.scalars().all()
        deployment = next(
            (d for d in candidates if d.status in ("in_progress", "pending")),
            candidates[0] if candidates else None,
        )

    revision: SoftwareRevision | None = None
    revision_ids = {d.software_revision_id for d in ordered_devices if d.software_revision_id}
    if deployment is not None:
        revision_ids.add(deployment.software_revision_id)
    revisions = await _fetch_map(session, SoftwareRevision, sorted(revision_ids))
    if deployment is not None:
        revision = revisions.get(deployment.software_revision_id)

    revision_version = revision.version if revision else ""
    context = EntityContext(
        account=(
            {"id": account.id, "name": account.name, "tier": account.tier} if account else None
        ),
        site={"id": site.id, "name": site.name} if site else None,
        fleet={"id": fleet.id, "name": fleet.name} if fleet else None,
        devices=[
            {
                "id": d.id,
                "status": d.status,
                "software": (
                    revisions[d.software_revision_id].version
                    if d.software_revision_id in revisions
                    else ""
                ),
            }
            for d in ordered_devices
        ],
        deployment=(
            {
                "id": deployment.id,
                "status": deployment.status,
                "version": revision_version,
            }
            if deployment
            else None
        ),
        software_revision=({"id": revision.id, "version": revision.version} if revision else None),
    )

    query_text = " ".join(
        part
        for part in [
            ticket.subject if ticket else "",
            (ticket.body or "") if ticket else "",
            *(e.description or "" for e in events),
        ]
        if part
    )[:2000]

    logger.info(
        "Resolved signals to account=%s fleet=%s devices=%d severity=%s",
        account_id,
        fleet_id,
        len(ordered_devices),
        severity,
    )

    return ResolutionResult(
        entity_context=context,
        account_name=account.name if account else "",
        site_name=site.name if site else "",
        severity=severity,
        query_text=query_text,
    )
