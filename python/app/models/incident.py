from __future__ import annotations

from datetime import datetime

from sqlalchemy import DateTime, ForeignKey, String, Text
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.db.base import Base, TimestampMixin


class Incident(Base, TimestampMixin):
    __tablename__ = "incidents"

    id: Mapped[str] = mapped_column(String(50), primary_key=True)
    account_id: Mapped[str] = mapped_column(String(50), ForeignKey("accounts.id"), nullable=False)
    severity: Mapped[str] = mapped_column(String(50), nullable=False, default="medium")
    status: Mapped[str] = mapped_column(String(50), nullable=False, default="open")
    title: Mapped[str] = mapped_column(String(500), nullable=False)
    description: Mapped[str | None] = mapped_column(Text, nullable=True)
    detected_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    resolved_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    account: Mapped["Account"] = relationship(back_populates="incidents")
    tickets: Mapped[list["Ticket"]] = relationship(back_populates="incident")
