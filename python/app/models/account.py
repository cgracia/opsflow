from __future__ import annotations

from typing import TYPE_CHECKING

from sqlalchemy import String
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.db.base import Base, TimestampMixin

if TYPE_CHECKING:
    from app.models.incident import Incident
    from app.models.site import Site
    from app.models.ticket import Ticket


class Account(Base, TimestampMixin):
    __tablename__ = "accounts"

    id: Mapped[str] = mapped_column(String(50), primary_key=True)
    name: Mapped[str] = mapped_column(String(200), nullable=False)
    tier: Mapped[str] = mapped_column(String(50), nullable=False, default="standard")
    region: Mapped[str] = mapped_column(String(50), nullable=False, default="us-west")
    sites: Mapped[list[Site]] = relationship(back_populates="account", cascade="all, delete-orphan")
    incidents: Mapped[list[Incident]] = relationship(back_populates="account")
    tickets: Mapped[list[Ticket]] = relationship(back_populates="account")
