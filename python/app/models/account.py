from __future__ import annotations

from sqlalchemy import String
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.db.base import Base, TimestampMixin


class Account(Base, TimestampMixin):
    __tablename__ = "accounts"

    id: Mapped[str] = mapped_column(String(50), primary_key=True)
    name: Mapped[str] = mapped_column(String(200), nullable=False)
    tier: Mapped[str] = mapped_column(String(50), nullable=False, default="standard")
    region: Mapped[str] = mapped_column(String(50), nullable=False, default="us-west")
    sites: Mapped[list["Site"]] = relationship(
        back_populates="account", cascade="all, delete-orphan"
    )
    incidents: Mapped[list["Incident"]] = relationship(back_populates="account")
    tickets: Mapped[list["Ticket"]] = relationship(back_populates="account")
