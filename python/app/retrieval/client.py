from qdrant_client import QdrantClient
from qdrant_client.models import (
    Distance,
    VectorParams,
    SparseVectorParams,
    SparseIndexParams,
    PointStruct,
    SparseVector,
)

COLLECTION_NAME = "operational_evidence"
VECTOR_SIZE = 384  # all-MiniLM-L6-v2 dimension


def _to_sparse(sparse_dict: dict | None) -> SparseVector:
    """Convert a {index: value} dict to a SparseVector, or return empty."""
    if not sparse_dict:
        return SparseVector(indices=[], values=[])
    indices = [int(k) for k in sparse_dict.keys()]
    values = [float(v) for v in sparse_dict.values()]
    return SparseVector(indices=indices, values=values)


class QdrantManager:
    """Manages Qdrant collection lifecycle and indexing."""

    def __init__(self, url: str = "http://localhost:6333", api_key: str = ""):
        self.client = QdrantClient(url=url, api_key=api_key or None)
        self._collection_ready = False

    def ensure_collection(self) -> None:
        """Create collection if it doesn't exist, with dense + sparse vectors."""
        collections = self.client.get_collections().collections
        names = [c.name for c in collections]

        if COLLECTION_NAME not in names:
            self.client.create_collection(
                collection_name=COLLECTION_NAME,
                vectors_config=VectorParams(size=VECTOR_SIZE, distance=Distance.COSINE),
                sparse_vectors_config={
                    "bm25": SparseVectorParams(
                        index=SparseIndexParams(on_disk=False),
                    )
                },
            )
        self._collection_ready = True

    def index_document(
        self,
        doc_id: str,
        content: str,
        entity_id: str,
        entity_type: str,
        source_type: str,
        timestamp: str | None = None,
        metadata: dict | None = None,
        dense_vector: list[float] | None = None,
        sparse_vector: dict | None = None,
    ) -> None:
        """Index a single document into the collection."""
        if not self._collection_ready:
            self.ensure_collection()

        # Use fake deterministic vector if none provided (for tests/dev)
        if dense_vector is None:
            dense_vector = self._fake_vector(doc_id)

        payload = {
            "entity_id": entity_id,
            "entity_type": entity_type,
            "source_type": source_type,
            "content": content,
            "timestamp": timestamp,
            "metadata": metadata or {},
        }

        point = PointStruct(
            id=doc_id,
            vector={"": dense_vector, "bm25": _to_sparse(sparse_vector)},
            payload=payload,
        )
        self.client.upsert(collection_name=COLLECTION_NAME, points=[point])

    def index_documents(self, documents: list[dict]) -> None:
        """Index multiple documents at once."""
        if not self._collection_ready:
            self.ensure_collection()

        points = []
        for doc in documents:
            dense = doc.get("dense_vector") or self._fake_vector(doc["id"])
            points.append(
                PointStruct(
                    id=doc["id"],
                    vector={"": dense, "bm25": _to_sparse(doc.get("sparse_vector"))},
                    payload={
                        "entity_id": doc["entity_id"],
                        "entity_type": doc["entity_type"],
                        "source_type": doc["source_type"],
                        "content": doc["content"],
                        "timestamp": doc.get("timestamp"),
                        "metadata": doc.get("metadata", {}),
                    },
                )
            )

        self.client.upsert(collection_name=COLLECTION_NAME, points=points)

    @staticmethod
    def _fake_vector(doc_id: str) -> list[float]:
        """Generate deterministic fake vector from doc_id for testing."""
        import hashlib

        hash_bytes = hashlib.sha256(doc_id.encode()).digest()
        vector = []
        for i in range(VECTOR_SIZE):
            byte_val = hash_bytes[i % len(hash_bytes)]
            vector.append(byte_val / 255.0 - 0.5)
        return vector

    def get_collection_info(self):
        """Get collection info for debugging."""
        return self.client.get_collection(collection_name=COLLECTION_NAME)
