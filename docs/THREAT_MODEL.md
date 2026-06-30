# nowdocs Threat Model — Registry Contributor Security Gates

> This document defines the CI-enforced security rules for registry contributor PRs.
> Every contributor must understand these rules **before** submitting a PR.
> See also: spec §6 (security threat model), §6.10 (legal gate), §6.11 (governance).

## Why These Rules Exist

nowdocs runs locally on developer machines and processes untrusted documentation content.
The registry is the primary attack surface: a malicious contributor could inject:

- **Prompt-injection payloads** in documentation text (A1)
- **Adversarial vector embeddings** that make malicious chunks rank high for specific queries (D10)
- **Malicious download sources** that could SSRF or deliver tampered content (A2)
- **Incompatible model versions** that degrade embedding quality silently (A3)
- **Non-redistributable content** that creates legal liability (§6.10)

CI is the enforcement boundary — these checks run on every PR before merge.

---

## CI Rules (7 gates, enforced on every PR)

### 1. Manifest Schema Validation

**What**: Every submitted `manifest.json` must pass `scripts/ci-check-manifest.sh`.

**Checks**:
- `nowdocs_schema_version` = 1 (only v1 supported)
- `retrieval.tokenizer` = "default" (lindera reserved for v2 CJK)

**Why**: Schema drift causes silent data corruption. Version gating ensures the manifest format matches what the binary can process.

**Local verification**:
```bash
bash scripts/ci-check-manifest.sh path/to/manifest.json
```

### 2. Model Version Match

**What**: The embedder spec must match the locked model exactly.

**Locked values**:
| Field | Required Value |
|-------|---------------|
| `model_id` | `jinaai/jina-embeddings-v2-small-en` |
| `vector_dim` | `512` |
| `engine` | `candle` |
| `dtype` | `f16` |
| `model_revision` | non-empty (HF commit SHA pin) |
| `model_sha256` | non-empty (SHA-256 integrity hash) |

**Why (A3)**: Different model versions produce different embedding spaces. A contributor using a different model would create vectors that are incompatible with the canonical model, causing silent search quality degradation. Pinning revision + SHA-256 ensures model integrity on download.

### 3. Legal License Whitelist

**What**: `legal.license` must be one of: `MIT`, `Apache-2.0`, `CC-BY-4.0`.

**Additional constraint**: `CC-BY-4.0` requires non-empty `legal.attribution`.

**Why (§6.10)**: nowdocs distributes documentation content. Only licenses that permit redistribution with attribution are acceptable. Proprietary or restrictive licenses (e.g., Clerk ToS, Tailwind "IP") are rejected. CC-BY-4.0 mandates attribution — without it, the license is not satisfied.

**Canonical crates**: Next.js (MIT), React (CC-BY-4.0), Vue (CC-BY-4.0). Clerk and Tailwind are explicitly excluded (§6.10).

### 4. Download URL Domain Check

**What**: All URLs in `source.source_url` must point to allowed registry domains.

**Allowed domains**:
- `github.com/nowdocs-registry/*` (with `/nowdocs-registry/` path prefix — prevents lookalike domains like `github.com/nowdocs-registry.evil.com`)
- `registry.nowdocs.rs/*`

**Why (A2)**: Prevents SSRF attacks and malicious content sources. If a contributor could point `source_url` to an arbitrary URL, they could:
- Redirect CI downloads to attacker-controlled servers
- Inject tampered archives
- Exfiltrate CI environment variables via DNS/HTTP callbacks

**Local verification**: `scripts/ci-check-manifest.sh` enforces this check.

### 5. CI Rebuild from Text (D10)

**What**: The CI pipeline rejects any vector files (`.lance`, `.faiss`, `vectors.*`, `embeddings.*`). Only text (`chunks.jsonl`) and metadata (`manifest.json`) are accepted. CI rebuilds the embedding index from text using the canonical model.

**Why (D10)**: This is the most critical security gate — it closes three attack vectors simultaneously:

1. **Vector poisoning**: Vectors are opaque floats. A malicious contributor could craft vectors that make a malicious chunk rank #1 for a specific query, even if the text is innocuous. Since vectors can't be audited, the only safe approach is to rebuild from auditable text.

2. **Model version drift**: If contributors use different model versions, the vectors would be incompatible with the canonical model, degrading search quality silently.

3. **Trust boundary**: Text is human-readable and auditable; vectors are not. By requiring text-only submission, every piece of content that influences search results can be reviewed by maintainers.

**Share output** (what contributors submit):
- `manifest.json` — metadata
- `chunks.jsonl` — text chunks with metadata (one JSON object per line)

**What is NOT accepted**: `.lance` directories, vector files, embedding artifacts.

**Local verification**:
```bash
bash scripts/ci-no-vectors.sh .
```

### 6. Golden Eval Gate

**What**: Run the golden evaluation suite (recall@5 >= 0.8, MRR >= 0.6) against a canonical test set.

**Status**: **Deferred** — depends on Task 01 (3b golden eval) being wired into the CI binary. The eval tests run as part of `cargo test` (non-ignored tests), but the full E2E golden gate requires a working retrieve pipeline + model download.

**Why**: Embedding or chunking changes can silently degrade search quality. The golden eval catches regressions before they reach users.

### 7. DCO (Developer Certificate of Origin)

**What**: Every commit in a PR must include a `Signed-off-by:` line.

**Why (D8)**: DCO (not CLA) is the Rust ecosystem convention. It certifies that the contributor has the right to submit the content under the project's license (MIT OR Apache-2.0). CLAs are ineffective for third-party documentation content and create a false sense of legal protection.

**How to sign**: `git commit --signoff` (or `-s` flag).

---

## Local Verification

Run all checks locally before pushing:

```bash
# 1. Manifest validation
bash scripts/ci-check-manifest.sh path/to/manifest.json

# 2. Vector file scan
bash scripts/ci-no-vectors.sh .

# 3. DCO check (between base and HEAD)
bash scripts/ci-check-dco.sh origin/main HEAD

# 4. Run CI tests
bash tests/ci/test_manifest_check.sh
bash tests/ci/test_no_vectors.sh
```

---

## Open Questions

1. **Golden eval compute budget (D10)**: Running the full eval in CI requires downloading the jina-v2-small model (~66MB) and running inference. CI runners have limited compute — estimated ~2-5 minutes for the eval. May need a dedicated runner or model caching.

2. **Manifest file scope**: The current CI checks `manifest.json` files in the repo root. If manifests are nested in subdirectories (e.g., `docsets/nextjs/manifest.json`), the glob pattern needs adjustment.

3. **Future: cargo-deny / cargo-audit**: Task 5e will add dependency auditing. These are complementary to the gates above (code dependencies vs. content security).
