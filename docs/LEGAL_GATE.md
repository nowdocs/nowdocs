# Legal Compliance & License Gate

This document records where and how legal gates run in the `nowdocs` ecosystem.

---

## 1. Division of Responsibilities

The nowdocs project separates compliance gates between two repositories: the **main codebase repository** (`nowdocs`) and the **community package registry repository** (`nowdocs-registry`).

### 1.1. Main Repository (`nowdocs`) CI Gates
The main repository enforces code quality, developer provenance, and local configuration validations.
- **Workflow Location:** `.github/workflows/gates.yml`
- **Job Name:** `compliance` (Runs on every Pull Request and Push to the `main` branch).
- **Checks Enforced:**
  1. **SPDX License Allowlist Check:** `scripts/ci-check-manifest.sh`
     - Validates that the license identifier declared in any `manifest.json` file is one of the approved open-source licenses: `MIT`, `Apache-2.0`, or `CC-BY-4.0`.
     - Ensures that when a `CC-BY-4.0` license is specified, a non-empty `attribution` metadata field is present.
  2. **DCO (Developer Certificate of Origin) Verification:** `scripts/ci-check-dco.sh`
     - Requires every commit in a Pull Request to be signed off (`Signed-off-by: Your Name <email>`).
     - Ensures contributors explicitly assert their right to submit the code/docs under the project's dual license (`MIT OR Apache-2.0`).
  3. **No-Vectors Scan:** `scripts/ci-no-vectors.sh`
     - Scans the tree to verify that raw binary `.lance` table files are not checked into the Git history. Only clean source text and manifest configuration should ever be committed.

### 1.2. Registry Repository (`nowdocs-registry`) CI Gates
The community registry repository handles the ingestion, rebuild validation, and publishing of docsets.
- **Workflow Location:** Custom GitHub workflows inside the `nowdocs-registry` repository.
- **Checks Enforced:**
  1. **Source Authenticity & Provenance Check:**
     - Checks the `source.source_url` and `source.entry_url` inside the submitted manifest.
     - Enforces that no copyright-restricted or crawlers-forbidden documentation (e.g., Clerk, Tailwind Labs) is ingested into the registry.
  2. **D10 Vector Reconstruction (Zero-Trust Embedding):**
     - Contributions to the registry must only contain raw chunked text and the manifest (`chunks.jsonl` + `manifest.json`). **No `.lance` binary tables containing contributor-computed vectors are accepted.**
     - The Registry CI automatically rebuilds the vector database from the raw text chunks using a standard, pinned HuggingFace model cache.
     - **Security Value:** This closes adversarial vector injection channels (preventing malicious embeddings that force target queries to hit specific poisoned chunks) and avoids model version drift.

---

## 2. White-listed Licenses

The SPDX identifiers permitted by the validation gate (`scripts/ci-check-manifest.sh`) are restricted to:
- `MIT`
- `Apache-2.0`
- `CC-BY-4.0` (Requires `legal.attribution` to be populated)
