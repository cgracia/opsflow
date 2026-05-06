import os

from sqlalchemy import Column, Integer, String

from app.db.base import Base, TimestampMixin
from app.db.session import init_db
from app.config import Settings


class SampleModel(Base, TimestampMixin):
    """Test model to verify base classes work."""
    __tablename__ = "sample_test"

    id: int = Column(Integer, primary_key=True)
    name: str = Column(String(100))


def test_base_has_declarative_base() -> None:
    """Base is a valid SQLAlchemy declarative base."""
    assert Base is not None
    assert hasattr(Base, "metadata")


def test_timestamp_mixin_has_columns() -> None:
    """TimestampMixin adds created_at and updated_at."""
    assert hasattr(TimestampMixin, "created_at")
    assert hasattr(TimestampMixin, "updated_at")


def test_sample_model_inherits_correctly() -> None:
    """Sample model has id, name, created_at, updated_at."""
    assert hasattr(SampleModel, "id")
    assert hasattr(SampleModel, "name")
    assert hasattr(SampleModel, "created_at")
    assert hasattr(SampleModel, "updated_at")
    assert SampleModel.__tablename__ == "sample_test"


def test_init_db_creates_engine() -> None:
    """init_db creates engine and session factory."""
    settings = Settings(
        database_url="sqlite+aiosqlite:///:memory:",
        env="test",
    )
    init_db(settings)

    from app.db import session as session_mod
    assert session_mod.async_engine is not None
    assert session_mod.AsyncSessionLocal is not None


def test_alembic_config_exists() -> None:
    """Alembic configuration files exist."""
    python_dir = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    assert os.path.exists(os.path.join(python_dir, "alembic.ini"))
    assert os.path.exists(os.path.join(python_dir, "alembic", "env.py"))
