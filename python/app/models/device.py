from __future__ import annotations

from datetime import datetime

from sqlalchemy import DateTime, ForeignKey, String
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.db.base import Base, TimestampMixin


class Device(Base, TimestampMixin):
    __tablename__ = "devices"

    id: Mapped[str] = mapped_column(String(50), primary_key=True)
    fleet_id: Mapped[str] = mapped_column(String(50), ForeignKey("fleets.id"), nullable=False)
    site_id: Mapped[str] = mapped_column(String(50), ForeignKey("sites.id"), nullable=False)
    account_id: Mapped[str] = mapped_column(String(50), ForeignKey("accounts.id"), nullable=False)
    device_serial: Mapped[str] = mapped_column(String(100), nullable=False, unique=True)
    device_type: Mapped[str] = mapped_column(String(100), nullable=False)
    software_revision_id: Mapped[str | None] = mapped_column(
        String(50), ForeignKey("software_revisions.id"), nullable=True
    )
    status: Mapped[str] = mapped_column(String(50), nullable=False, default="active")
    last_seen_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    fleet: Mapped["Fleet"] = relationship(back_populates="devices")
    site: Mapped["Site"] = relationship()
    account: Mapped["Account"] = relationship()
    software_revision: Mapped["SoftwareRevision | None"] = relationship()
    tickets: Mapped[list["Ticket"]] = relationship(back_populates="device")
    operational_events: Mapped[list["OperationalEvent"]] = relationship(back_populates="device")
