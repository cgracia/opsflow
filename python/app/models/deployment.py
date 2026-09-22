from __future__ import annotations

from datetime import datetime
from typing import TYPE_CHECKING

from sqlalchemy import DateTime, ForeignKey, String
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.db.base import Base, TimestampMixin

if TYPE_CHECKING:
    from app.models.fleet import Fleet
    from app.models.software_revision import SoftwareRevision


class Deployment(Base, TimestampMixin):
    __tablename__ = "deployments"

    id: Mapped[str] = mapped_column(String(50), primary_key=True)
    fleet_id: Mapped[str] = mapped_column(String(50), ForeignKey("fleets.id"), nullable=False)
    software_revision_id: Mapped[str] = mapped_column(
        String(50), ForeignKey("software_revisions.id"), nullable=False
    )
    status: Mapped[str] = mapped_column(String(50), nullable=False, default="pending")
    started_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    completed_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    fleet: Mapped[Fleet] = relationship(back_populates="deployments")
    software_revision: Mapped[SoftwareRevision] = relationship()
