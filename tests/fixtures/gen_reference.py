# Run once: python3 tests/fixtures/gen_reference.py
# Requires: pip install sentence-transformers torch
# Produces tests/fixtures/jina_ref.json with the canonical 512-dim vector
# for a pinned query, from the reference (Python) embedder.
import json
from sentence_transformers import SentenceTransformer

MODEL = "jinaai/jina-embeddings-v2-small-en"
QUERY = "how to use clerkMiddleware"

m = SentenceTransformer(MODEL, trust_remote_code=True)
vec = m.encode(QUERY, normalize_embeddings=False).tolist()
rev = m.model.config.get("_name_or_path", "unknown")

out = {
    "model_id": MODEL,
    "query": QUERY,
    "vector": vec,
    "dim": len(vec),
    "source": "sentence-transformers",
    "revision": rev,
}
with open("tests/fixtures/jina_ref.json", "w") as f:
    json.dump(out, f)
print(f"wrote fixture dim={len(vec)}")
