from __future__ import annotations

from sqlalchemy import ForeignKey, String, Text
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.db.base import Base, TimestampMixin


class Ticket(Base, TimestampMixin):
    __tablename__ = "tickets"

    id: Mapped[str] = mapped_column(String(50), primary_key=True)
    account_id: Mapped[str] = mapped_column(String(50), ForeignKey("accounts.id"), nullable=False)
    site_id: Mapped[str | None] = mapped_column(String(50), ForeignKey("sites.id"), nullable=True)
    device_id: Mapped[str | None] = mapped_column(
        String(50), ForeignKey("devices.id"), nullable=True
    )
    incident_id: Mapped[str | None] = mapped_column(
        String(50), ForeignKey("incidents.id"), nullable=True
    )
    subject: Mapped[str] = mapped_column(String(500), nullable=False)
    body: Mapped[str | None] = mapped_column(Text, nullable=True)
    priority: Mapped[str] = mapped_column(String(50), nullable=False, default="medium")
    channel: Mapped[str] = mapped_column(String(50), nullable=False, default="email")
    status: Mapped[str] = mapped_column(String(50), nullable=False, default="open")
    account: Mapped["Account"] = relationship(back_populates="tickets")
    site: Mapped["Site | None"] = relationship(back_populates="tickets")
    device: Mapped["Device | None"] = relationship(back_populates="tickets")
    incident: Mapped["Incident | None"] = relationship(back_populates="tickets")
