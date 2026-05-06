from __future__ import annotations

from sqlalchemy import ForeignKey, String
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.db.base import Base, TimestampMixin


class Fleet(Base, TimestampMixin):
    __tablename__ = "fleets"

    id: Mapped[str] = mapped_column(String(50), primary_key=True)
    site_id: Mapped[str] = mapped_column(String(50), ForeignKey("sites.id"), nullable=False)
    account_id: Mapped[str] = mapped_column(String(50), ForeignKey("accounts.id"), nullable=False)
    name: Mapped[str] = mapped_column(String(200), nullable=False)
    fleet_type: Mapped[str] = mapped_column(String(100), nullable=False)
    site: Mapped["Site"] = relationship(back_populates="fleets")
    account: Mapped["Account"] = relationship()
    devices: Mapped[list["Device"]] = relationship(
        back_populates="fleet", cascade="all, delete-orphan"
    )
    deployments: Mapped[list["Deployment"]] = relationship(back_populates="fleet")
    operational_events: Mapped[list["OperationalEvent"]] = relationship(back_populates="fleet")
