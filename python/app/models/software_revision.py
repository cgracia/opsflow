from datetime import datetime

from sqlalchemy import DateTime, String, Text
from sqlalchemy.orm import Mapped, mapped_column

from app.db.base import Base, TimestampMixin


class SoftwareRevision(Base, TimestampMixin):
    __tablename__ = "software_revisions"

    id: Mapped[str] = mapped_column(String(50), primary_key=True)
    version: Mapped[str] = mapped_column(String(50), nullable=False, unique=True)
    release_notes: Mapped[str | None] = mapped_column(Text, nullable=True)
    deployed_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
