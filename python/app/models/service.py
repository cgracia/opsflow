from sqlalchemy import String
from sqlalchemy.orm import Mapped, mapped_column

from app.db.base import Base, TimestampMixin


class Service(Base, TimestampMixin):
    __tablename__ = "services"

    id: Mapped[str] = mapped_column(String(50), primary_key=True)
    name: Mapped[str] = mapped_column(String(200), nullable=False)
    version: Mapped[str] = mapped_column(String(50), nullable=False)
    status: Mapped[str] = mapped_column(String(50), nullable=False, default="active")
