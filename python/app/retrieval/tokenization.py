"""Deterministic token-to-sparse-index mapping shared by indexing and search.

The v1 sparse leg is a word-count bag, not BM25 (disclosed in the README).
For a word-count bag to match anything at all, indexing and querying must map
tokens to the *same* vector indices. Indices are therefore derived from a
stable hash of the token — never from per-document or per-query enumeration,
which produced indices that could not coincide and a sparse leg that matched
nothing.
"""

import hashlib
import re

from qdrant_client.models import SparseVector

SPARSE_VOCAB_SIZE = 8192
_TOKEN_RE = re.compile(r"[a-z0-9]+")


def tokenize(text: str) -> list[str]:
    """Lowercase alphanumeric tokens."""
    return _TOKEN_RE.findall(text.lower())


def _token_index(token: str) -> int:
    """Stable index for a token, independent of document or query context."""
    digest = hashlib.blake2b(token.encode(), digest_size=8).digest()
    return int.from_bytes(digest, "big") % SPARSE_VOCAB_SIZE


def sparse_from_text(text: str) -> SparseVector:
    """Build a word-count sparse vector from text using the shared mapping."""
    counts: dict[int, float] = {}
    for token in tokenize(text):
        index = _token_index(token)
        counts[index] = counts.get(index, 0.0) + 1.0
    indices = sorted(counts)
    return SparseVector(indices=indices, values=[counts[i] for i in indices])
