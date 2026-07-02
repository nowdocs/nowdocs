# license audit (curator tool)

Three-layer license audit for a source repo's **documentation**, to decide
whether nowdocs may ingest it. This is a **curator tool** — for the human
curating public-registry docsets, not for end users. It is independent of the
`nowdocs` binary: no `Cargo.toml` change, and `nowdocs` itself stays
zero-network / zero-key. This tool *does* call LLM endpoints; that is fine
because it runs on the curator's machine, not inside the shipped binary.

## Why three layers

The Creative Commons family differs by a single suffix with opposite legal
meaning — `CC-BY-4.0` (allowed) vs `CC-BY-ND-4.0` (NoDerivatives, banned) vs
`CC-BY-SA-4.0` (copyleft, not accepted) vs `CC-BY-NC-4.0` (non-commercial,
banned). Two LLMs can both misread a near-identical id, so they are not
trusted alone:

1. **regex** (`regex_rules.py`) — deterministic, zero-hallucination. Finds
   LICENSE files, `SPDX-License-Identifier:` markers, bare SPDX ids, and
   Creative-Commons prose names. Flags dangerous ids (ND/NC/SA/GPL*) regardless
   of what the LLMs say. This is the ground-truth anchor.
2. **gemma** — local vLLM reads the regex findings + README/docs prose and
   judges the *documentation* license (distinct from the code license in the
   root LICENSE).
3. **glm5.2** — Volcano Engine Ark judges the same input independently.

The two LLM verdicts are **diffed**: disagreements are surfaced at the top of
the report (review these first); agreements are shown below. Even on agreement,
the curator spot-checks CC-BY-* / Apache-* families and any ambiguous "Creative
Commons" wording, because two models can co-misread. Agreement reduces review
load; it is not a waiver.

## Setup

Copy `env.example` values into your shell (or a gitignored `.env`). The gemma
key is not needed (local vLLM). The glm5.2 key (`ARK_API_KEY_HO`) must be loaded
from 1Password via `op-secrets` — never hardcoded.

```sh
# Start the local vLLM that serves gemma.
cd ~/Projects/gwmm && docker compose up -d    # maps 127.0.0.1:8000

# Load env (adjust to your op-secrets flow for the Ark key).
export AUDIT_GEMMA_URL="http://localhost:8000/v1"
export AUDIT_GEMMA_MODEL="RedHatAI/gemma-4-12B-it-FP8-Dynamic"
export AUDIT_GLM_URL="https://ark.cn-beijing.volces.com/api/coding/v3"
export AUDIT_GLM_MODEL="glm-5.2"
export ARK_API_KEY_HO="$(op read 'op://...')"   # via op-secrets
```

## Use

```sh
python3 audit.py /path/to/cloned/repo            # human report to stdout
python3 audit.py /path/to/cloned/repo --out r.json   # also machine-readable
python3 audit.py /path/to/cloned/repo --no-color     # pipe-friendly
```

Exit code: `0` = clean (models agree + allowlisted), `1` = needs a human
(disagreement / dangerous / unknown / a layer errored), `2` = usage error.

## Run the regex tests

```sh
python3 test_regex.py      # pure, no network, exits non-zero on failure
```

These cover the hard cases: MIT not matched inside `MITIGATION`, the CC-BY-ND /
CC-BY-SA suffixes not collapsing to plain `CC-BY-4.0`, prose forms
("Attribution-NoDerivatives 4.0"), node_modules exclusion, and dangerous
flagging. Re-run them after any `regex_rules.py` change.

## Policy reflected here

This tool encodes the registry intake policy decided with the curator:

- **Allowed:** MIT, Apache-2.0, BSD-2/3, ISC, Zlib, 0BSD, Unlicense, BSL-1.0,
  CC0-1.0, CC-BY-4.0 (needs `--attribution` at ingest).
- **Banned (regex flags, regardless of LLM):** CC-BY-ND, CC-BY-NC, GPL*, AGPL*,
  LGPL*.
- **Not accepted (regex flags):** CC-BY-SA (copyleft; would force the share
  bundle to re-license under SA).
- **Curator-only intake:** the public registry is curated, not open-submit. This
  tool never auto-ingests; it produces a report the curator acts on.

## Files

- `regex_rules.py` — layer 1 (deterministic scan).
- `audit.py` — layers 2+3 (two LLMs) + diff report.
- `test_regex.py` — regex unit tests.
- `env.example` — endpoint / model / key env vars (no real values).
