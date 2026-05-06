from app.retrieval.client import QdrantManager
from app.seed.evidence import get_all_evidence


async def index_all_evidence(manager: QdrantManager) -> int:
    """Index all synthetic evidence documents into Qdrant."""
    if not manager._collection_ready:
        manager.ensure_collection()

    documents = get_all_evidence()
    manager.index_documents(documents)
    return len(documents)
