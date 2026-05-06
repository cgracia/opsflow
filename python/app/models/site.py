from __future__ import annotations

from sqlalchemy import ForeignKey, String
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.db.base import Base, TimestampMixin


class Site(Base, TimestampMixin):
    __tablename__ = "sites"

    id: Mapped[str] = mapped_column(String(50), primary_key=True)
    account_id: Mapped[str] = mapped_column(String(50), ForeignKey("accounts.id"), nullable=False)
    name: Mapped[str] = mapped_column(String(200), nullable=False)
    location: Mapped[str] = mapped_column(String(200), nullable=True)
    timezone: Mapped[str] = mapped_column(String(50), nullable=False, default="UTC")
    account: Mapped["Account"] = relationship(back_populates="sites")
    fleets: Mapped[list["Fleet"]] = relationship(
        back_populates="site", cascade="all, delete-orphan"
    )
    tickets: Mapped[list["Ticket"]] = relationship(back_populates="site")
    operational_events: Mapped[list["OperationalEvent"]] = relationship(back_populates="site")
