#!/usr/bin/env python3
"""License audit for a nowdocs registry docset source repo.

Three layers, run in sequence:

  1. regex (regex_rules.scan_repo) — deterministic, zero-hallucination anchor.
     Finds LICENSE files, SPDX markers, and Creative-Commons prose names.
  2. gemma — local vLLM (RedHatAI/gemma-4-12B-it-FP8-Dynamic) reads the regex
     findings + README/CONTRIBUTING and judges the docs license.
  3. glm5.2 — Volcano Engine Ark (coding endpoint, model "glm-5.2") judges the
     same input independently.

The two LLM verdicts are diffed. Disagreements are surfaced at the top of the
report (the human reviews these first); agreements are shown below. regex
dangerous hits (CC-BY-ND/NC/SA, GPL*) are flagged regardless of LLM agreement,
because two LLMs can co-misread a near-identical CC id.

This is a CURATOR tool — for the human curating registry docsets, not for
end users. It is independent of the nowdocs binary: no Cargo.toml change,
no network from nowdocs itself. Model endpoints + keys come from env vars only.

Env vars:
  AUDIT_GEMMA_URL   e.g. http://localhost:8000/v1   (local vLLM, no key)
  AUDIT_GEMMA_MODEL e.g. RedHatAI/gemma-4-12B-it-FP8-Dynamic
  AUDIT_GLM_URL     e.g. https://ark.cn-beijing.volces.com/api/coding
  AUDIT_GLM_MODEL   e.g. glm-5.2
  ARK_API_KEY_HO    Ark key (for glm5.2). gemma needs no key.

Usage:
  python3 audit.py <repo_path> [--out report.json]
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request

sys.path.insert(0, os.path.dirname(__file__))
from regex_rules import DANGEROUS, scan_repo  # noqa: E402

# Licenses the ingest pipeline accepts (mirrors manifest::validate allowlist).
ALLOWLIST = {
    "MIT", "Apache-2.0", "BSD-3-Clause", "BSD-2-Clause", "ISC", "Zlib",
    "0BSD", "Unlicense", "BSL-1.0", "CC0-1.0", "CC-BY-4.0", "CC-BY-3.0",
}

# Files whose prose is most likely to state the *docs* license (vs the code
# license in the root LICENSE). Fed to the LLMs as reading context.
DOCS_PROSE_FILES = ("README.md", "README", "CONTRIBUTING.md", "CONTRIBUTING",
                    "docs/README.md", "docs/LICENSE.md", "docs/NOTICE")

JUDGE_PROMPT = """\
You are auditing the license of the DOCUMENTATION in a source repo, to decide
whether nowdocs may ingest it (copy + strip + chunk + embed + redistribute).

CRITICAL DISTINCTIONS (a single suffix changes the legal meaning):
- CC-BY-4.0        = Attribution only            -> ALLOWED (needs attribution)
- CC-BY-ND-4.0     = NoDerivatives               -> BANNED (derived work forbidden)
- CC-BY-NC-4.0     = NonCommercial               -> BANNED
- CC-BY-SA-4.0     = ShareAlike (copyleft)       -> NOT ACCEPTED by nowdocs policy
- GPL*/AGPL*/LGPL* = strong copyleft             -> BANNED
- MIT/Apache-2.0/BSD/ISC/Zlib/0BSD/Unlicense/BSL-1.0 = permissive -> ALLOWED
- CC0-1.0          = public domain               -> ALLOWED

The repo root LICENSE often covers the CODE, not the docs. The docs license may
be stated separately in a docs/ file or in README prose. Identify the
DOCUMENTATION license specifically.

Here are deterministic regex findings from the repo (ground truth for ids):

{regex_findings}

Here is relevant prose from the repo (README / docs / contributing):

{prose}

Answer ONLY with a single JSON object on one line, no markdown:
{{"docs_license": "<SPDX id or UNKNOWN>", "source": "<file:line or 'prose'>", "confidence": "high|medium|low", "rationale": "<one sentence>"}}
"""


# --- model call (stdlib only, OpenAI-compatible /v1/chat/completions) --------

def _chat(base_url: str, model: str, prompt: str, api_key: str | None,
          timeout: float = 120.0) -> str:
    """Call an OpenAI-compatible chat endpoint. Returns assistant text."""
    url = base_url.rstrip("/") + "/chat/completions"
    body = json.dumps({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.0,
    }).encode()
    headers = {"Content-Type": "application/json"}
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"
    req = urllib.request.Request(url, data=body, headers=headers, method="POST")
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            data = json.loads(resp.read().decode())
    except urllib.error.HTTPError as e:
        raise RuntimeError(f"{url} -> HTTP {e.code}: {e.read().decode()[:300]}") from e
    except urllib.error.URLError as e:
        raise RuntimeError(f"{url} -> {e.reason}") from e
    try:
        return data["choices"][0]["message"]["content"].strip()
    except (KeyError, IndexError) as e:
        raise RuntimeError(f"unexpected response shape: {data}") from e


def _parse_verdict(raw: str) -> dict:
    """Extract the JSON object from model output (tolerates surrounding text)."""
    raw = raw.strip().strip("`")
    # Find the first {...} blob.
    start = raw.find("{")
    end = raw.rfind("}")
    if start == -1 or end == -1 or end < start:
        return {"docs_license": "PARSE_ERROR", "raw": raw[:200]}
    try:
        return json.loads(raw[start : end + 1])
    except json.JSONDecodeError:
        return {"docs_license": "PARSE_ERROR", "raw": raw[start : end + 1][:200]}


# --- prose collection for the LLM context ------------------------------------

def _collect_prose(repo: str, limit_chars: int = 6000) -> str:
    import os as _os

    chunks: list[str] = []
    for rel in DOCS_PROSE_FILES:
        p = _os.path.join(repo, rel)
        if _os.path.isfile(p):
            try:
                with open(p, encoding="utf-8", errors="replace") as f:
                    chunks.append(f"### {rel}\n" + f.read()[:2000])
            except OSError:
                pass
        if sum(len(c) for c in chunks) > limit_chars:
            break
    return "\n\n".join(chunks) if chunks else "(no README/docs prose found)"


# --- report rendering --------------------------------------------------------

def _verdict_id(v: dict) -> str:
    s = str(v.get("docs_license", "")).strip().upper()
    return s or "UNKNOWN"


def render_report(repo: str, scan, gemma: dict, glm: dict, *, use_color: bool = True) -> str:
    g_id, l_id = _verdict_id(gemma), _verdict_id(glm)
    agree = g_id == l_id
    regex_dangerous = scan.dangerous()
    regex_seen = scan.spdx_seen

    B = "\033[1m" if use_color else ""
    R = "\033[31m" if use_color else ""   # red
    Y = "\033[33m" if use_color else ""   # yellow
    G = "\033[32m" if use_color else ""   # green
    D = "\033[2m" if use_color else ""    # dim
    X = "\033[0m" if use_color else ""

    lines: list[str] = []
    lines.append(f"{B}=== license audit: {repo}{X}")
    lines.append("")

    # 1. Top-line verdict
    if regex_dangerous:
        lines.append(f"{R}!! REGEX DANGEROUS HIT: {', '.join(regex_dangerous)}{X}")
        lines.append(f"{R}   These are banned by policy regardless of LLM agreement.{X}")
        lines.append("")
    if agree:
        lines.append(f"{G}models AGREE: docs_license = {g_id}{X}")
    else:
        lines.append(f"{R}models DISAGREE:{X} gemma={g_id}  glm5.2={l_id}")
        lines.append(f"{Y}  >> review the disagreement below FIRST <<{X}")
    lines.append("")

    # 2. regex findings summary
    lines.append(f"{B}-- regex (deterministic){X}")
    lines.append(f"   SPDX ids seen: {regex_seen or '(none)'}")
    if regex_dangerous:
        lines.append(f"   dangerous:    {regex_dangerous}")
    allow_match = [s for s in regex_seen if s in ALLOWLIST]
    lines.append(f"   in allowlist: {allow_match or '(none)'}")
    lines.append("")

    # 3. model verdicts
    for label, v, color in (("gemma", gemma, G), ("glm5.2", glm, Y)):
        lines.append(f"{B}-- {label}{X}")
        lines.append(f"   docs_license: {color}{_verdict_id(v)}{X}")
        lines.append(f"   confidence:   {v.get('confidence', '?')}")
        lines.append(f"   source:       {v.get('source', '?')}")
        lines.append(f"   rationale:    {v.get('rationale', '?')}")
        if v.get("raw"):
            lines.append(f"   {D}(parse error raw: {v['raw']}){X}")
        lines.append("")

    # 4. all regex findings (detail)
    lines.append(f"{B}-- all regex findings{X}")
    if not scan.findings:
        lines.append("   (none — no license declaration found anywhere)")
    else:
        for f in scan.findings:
            flag = f" {R}[DANGEROUS]{X}" if f.spdx in DANGEROUS else ""
            spdx = f.spdx or "(prose/no id)"
            lines.append(f"   {f.kind:14s} {f.file}:{f.line}  {spdx}{flag}")
            if f.raw:
                lines.append(f"   {D}raw: {f.raw[:100]}{X}")
            lines.append(f"   {D}ctx: {f.snippet}{X}")
    lines.append("")

    # 5. recommendation
    banned = bool(regex_dangerous) or g_id in DANGEROUS or l_id in DANGEROUS
    if banned:
        lines.append(f"{R}RECOMMEND: DO NOT INGEST (banned license detected){X}")
    elif not agree:
        lines.append(f"{Y}RECOMMEND: human must resolve disagreement before ingest{X}")
    elif g_id not in ALLOWLIST and g_id != "UNKNOWN":
        lines.append(f"{Y}RECOMMEND: {g_id} not in allowlist — review before ingest{X}")
    elif g_id == "UNKNOWN":
        lines.append(f"{Y}RECOMMEND: license unknown — manual review required{X}")
    else:
        lines.append(f"{G}RECOMMEND: {g_id} is allowlisted; proceed (set --attribution if CC-BY){X}")
    lines.append(f"{D}(curator decision — this tool does not auto-ingest){X}")
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser(description="Audit a repo's documentation license.")
    ap.add_argument("repo", help="path to the cloned source repo")
    ap.add_argument("--out", help="also write a machine-readable JSON report here")
    ap.add_argument("--no-color", action="store_true")
    args = ap.parse_args()

    if not os.path.isdir(args.repo):
        print(f"not a directory: {args.repo}", file=sys.stderr)
        return 2

    gemma_url = os.environ.get("AUDIT_GEMMA_URL")
    gemma_model = os.environ.get("AUDIT_GEMMA_MODEL")
    glm_url = os.environ.get("AUDIT_GLM_URL")
    glm_model = os.environ.get("AUDIT_GLM_MODEL")
    ark_key = os.environ.get("ARK_API_KEY_HO")

    # Layer 1: regex (always runs, no deps).
    print("scanning with regex...", file=sys.stderr)
    scan = scan_repo(args.repo)

    def build_prompt() -> str:
        findings = "\n".join(
            f"- {f.kind} {f.file}:{f.line}  spdx={f.spdx or '(none)'}  raw={f.raw}"
            for f in scan.findings
        ) or "(no license declaration found by regex)"
        return JUDGE_PROMPT.format(regex_findings=findings, prose=_collect_prose(args.repo))

    # Layer 2 + 3: two independent LLMs.
    gemma_v, glm_v = {"docs_license": "SKIPPED"}, {"docs_license": "SKIPPED"}
    if not gemma_url or not gemma_model:
        print("AUDIT_GEMMA_URL/AUDIT_GEMMA_MODEL unset — skipping gemma layer", file=sys.stderr)
    else:
        print(f"querying gemma ({gemma_model})...", file=sys.stderr)
        try:
            gemma_v = _parse_verdict(_chat(gemma_url, gemma_model, build_prompt(), api_key=None))
        except RuntimeError as e:
            gemma_v = {"docs_license": "ERROR", "rationale": str(e)}

    if not glm_url or not glm_model:
        print("AUDIT_GLM_URL/AUDIT_GLM_MODEL unset — skipping glm5.2 layer", file=sys.stderr)
    elif not ark_key:
        print("ARK_API_KEY_HO unset — skipping glm5.2 layer", file=sys.stderr)
    else:
        print(f"querying glm5.2 ({glm_model})...", file=sys.stderr)
        try:
            glm_v = _parse_verdict(_chat(glm_url, glm_model, build_prompt(), api_key=ark_key))
        except RuntimeError as e:
            glm_v = {"docs_license": "ERROR", "rationale": str(e)}

    report = render_report(args.repo, scan, gemma_v, glm_v,
                           use_color=not args.no_color)
    print(report)

    if args.out:
        with open(args.out, "w") as f:
            json.dump({
                "repo": args.repo,
                "regex_spdx_seen": scan.spdx_seen,
                "regex_dangerous": scan.dangerous(),
                "gemma": gemma_v,
                "glm5.2": glm_v,
                "agree": _verdict_id(gemma_v) == _verdict_id(glm_v),
            }, f, indent=2)
        print(f"\n(wrote {args.out})", file=sys.stderr)

    # Exit code: 0 = clean (agree + allowlisted), 1 = needs human (disagree/
    #   dangerous/unknown), 2 = usage error. Lets a wrapper script gate on it.
    banned = bool(scan.dangerous()) or _verdict_id(gemma_v) in DANGEROUS or _verdict_id(glm_v) in DANGEROUS
    agree = _verdict_id(gemma_v) == _verdict_id(glm_v)
    if banned or not agree or _verdict_id(gemma_v) in ("UNKNOWN", "ERROR", "PARSE_ERROR", "SKIPPED"):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
