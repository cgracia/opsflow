from __future__ import annotations

from datetime import datetime

from sqlalchemy import DateTime, ForeignKey, String, Text
from sqlalchemy.dialects.sqlite import JSON
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.db.base import Base, TimestampMixin


class OperationalEvent(Base, TimestampMixin):
    __tablename__ = "operational_events"

    id: Mapped[str] = mapped_column(String(50), primary_key=True)
    device_id: Mapped[str | None] = mapped_column(
        String(50), ForeignKey("devices.id"), nullable=True
    )
    fleet_id: Mapped[str | None] = mapped_column(String(50), ForeignKey("fleets.id"), nullable=True)
    site_id: Mapped[str | None] = mapped_column(String(50), ForeignKey("sites.id"), nullable=True)
    account_id: Mapped[str | None] = mapped_column(
        String(50), ForeignKey("accounts.id"), nullable=True
    )
    event_type: Mapped[str] = mapped_column(String(100), nullable=False)
    severity: Mapped[str] = mapped_column(String(50), nullable=False, default="info")
    description: Mapped[str | None] = mapped_column(Text, nullable=True)
    event_metadata: Mapped[dict | None] = mapped_column(JSON, nullable=True)
    detected_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    device: Mapped["Device | None"] = relationship(back_populates="operational_events")
    fleet: Mapped["Fleet | None"] = relationship(back_populates="operational_events")
    site: Mapped["Site | None"] = relationship(back_populates="operational_events")
    account: Mapped["Account | None"] = relationship()
