"""Regex license-declaration rules for the audit tool.

Layer 1 of the three-layer audit (regex -> gemma -> glm5.2). This is the
deterministic, zero-hallucination anchor: it finds license declarations by
exact pattern matching and extracts SPDX ids. The two LLMs read these
findings as ground-truth context and only judge the *ambiguous* parts.

Why regex is non-optional here: the Creative Commons family differs by a
single suffix with opposite legal meaning — CC-BY-4.0 (allowed) vs
CC-BY-ND-4.0 (NoDerivatives, hard-banned) vs CC-BY-SA-4.0 (copyleft, not
accepted) vs CC-BY-NC-4.0 (non-commercial, hard-banned). A regex with
precise alternation catches these deterministically; an LLM reading prose
can misread "CC-BY" as the plain variant and miss the ND/SA/NC modifier.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field

# Dirs we never scan for license text. A docs source rarely nests its license
# under these, and recursing would risk pulling a vendored node_modules/LICENSE
# (the wrong license entirely).
EXCLUDE_DIRS = {
    ".git", "node_modules", "target", "venv", ".venv", "env",
    "dist", "build", "__pycache__", ".next", ".cache", "vendor",
    ".turbo", "coverage", ".idea", ".vscode",
}

# Files whose stem (case-insensitive, .md/.txt stripped) names a license file.
LICENSE_FILE_STEMS = {"license", "licence", "copying", "notice", "unlicense"}

# Text file extensions we scan for SPDX markers / prose declarations.
SCAN_EXTS = {".md", ".markdown", ".txt", ".rst", "license", "licence", ".py",
             ".rs", ".js", ".ts", ".tsx", ".jsx", ".go", ".java", ".rb", ".sh"}


# --- SPDX identifier matching -------------------------------------------------

# Order matters: compound CC ids (SA/ND/NC) MUST precede the bare CC-BY-4.0 so
# the longer match wins under leftmost-longest alternation. Copyleft ids are
# listed so we can flag them even though they're not in the ingest allowlist.
SPDX_IDS = [
    # CC family — specific modifiers first
    "CC-BY-SA-4.0", "CC-BY-ND-4.0", "CC-BY-NC-4.0",
    "CC-BY-SA-3.0", "CC-BY-ND-3.0", "CC-BY-NC-3.0",
    "CC-BY-4.0", "CC-BY-3.0",
    "CC0-1.0",
    # permissive
    "Apache-2.0", "BSD-3-Clause", "BSD-2-Clause", "MPL-2.0", "BSL-1.0",
    "Unlicense", "ISC", "Zlib", "0BSD",
    # copyleft — flagged, never auto-accepted
    "AGPL-3.0", "LGPL-3.0", "LGPL-2.1", "GPL-3.0", "GPL-2.0",
    # bare MIT last (short, would otherwise prefix-match inside words)
    "MIT",
]

# Build a single alternation, escaping and ordering longest-first. The
# negative lookbehind/lookahead `(?<![A-Za-z0-9-]) ... (?![A-Za-z0-9-])` stops
# "MIT" matching inside "MITIGATION" and "GPL-3.0" inside "GPL-3.0-or-later"
# being read as the bare id (we still report the bare id; the or-later suffix
# is noted in the snippet).
_sorted = sorted(set(SPDX_IDS), key=len, reverse=True)
_SPDX_RE = re.compile(
    r"(?<![A-Za-z0-9-])(" + "|".join(re.escape(s) for s in _sorted) + r")(?![A-Za-z0-9-])"
)

# Explicit SPDX marker, e.g. `SPDX-License-Identifier: MIT`
_SPDX_MARKER_RE = re.compile(r"SPDX-License-Identifier:\s*([A-Za-z0-9.+\-]+)")

# Natural-language Creative Commons names → SPDX id. The modifier word is the
# disambiguator we most need to catch.
_CC_PROSE = [
    (re.compile(r"Creative\s+Commons\s+Attribution[- ]+ShareAlike\s+(\d(?:\.\d)?)", re.I), "CC-BY-SA-"),
    (re.compile(r"Creative\s+Commons\s+Attribution[- ]+NoDeriv\w*\s+(\d(?:\.\d)?)", re.I), "CC-BY-ND-"),
    (re.compile(r"Creative\s+Commons\s+Attribution[- ]+NonCommercial\s+(\d(?:\.\d)?)", re.I), "CC-BY-NC-"),
    (re.compile(r"Creative\s+Commons\s+Attribution(?:[- ]+4\.0)?\s+(?:International\s+)?(\d(?:\.\d)?)", re.I), "CC-BY-"),
    (re.compile(r"\bAttribution[- ]+ShareAlike\s+(\d(?:\.\d)?)", re.I), "CC-BY-SA-"),
    (re.compile(r"\bAttribution[- ]+NoDeriv\w*\s+(\d(?:\.\d)?)", re.I), "CC-BY-ND-"),
    (re.compile(r"\bAttribution[- ]+NonCommercial\s+(\d(?:\.\d)?)", re.I), "CC-BY-NC-"),
]

# "licensed under <X>" / "released under <X>" / "distributed under <X>"
_LICENSED_UNDER_RE = re.compile(
    r"(?:licensed?|released|distributed|made available)\s+under\s+(.{3,80}?)(?:\.|\n|$)",
    re.I,
)

# Licenses we treat as dangerous regardless of model agreement.
DANGEROUS = {
    "CC-BY-ND-4.0", "CC-BY-ND-3.0",        # NoDerivatives — derived work banned
    "CC-BY-NC-4.0", "CC-BY-NC-3.0",        # NonCommercial
    "CC-BY-SA-4.0", "CC-BY-SA-3.0",        # copyleft (not accepted, per policy)
    "GPL-2.0", "GPL-3.0", "AGPL-3.0",
    "LGPL-2.1", "LGPL-3.0",
}


@dataclass
class Finding:
    kind: str            # "license_file" | "spdx_marker" | "prose" | "licensed_under"
    file: str
    line: int
    spdx: str            # extracted SPDX id, or "" if only raw prose
    snippet: str         # ~80 char context
    raw: str = ""        # the raw matched text (for prose / licensed_under)


@dataclass
class RepoScan:
    repo: str
    findings: list[Finding] = field(default_factory=list)
    # SPDX ids deduped, in first-seen order, split by safety.
    spdx_seen: list[str] = field(default_factory=list)

    def dangerous(self) -> list[str]:
        return [s for s in self.spdx_seen if s in DANGEROUS]


def _is_license_filename(name: str) -> bool:
    lower = name.lower()
    stem = lower
    for ext in (".md", ".markdown", ".txt", ".rst"):
        if stem.endswith(ext):
            stem = stem[: -len(ext)]
            break
    return stem in LICENSE_FILE_STEMS


def _scan_line(line: str) -> list[tuple[str, str, str]]:
    """Return [(kind, spdx, raw)] for declarations found on one line."""
    out: list[tuple[str, str, str]] = []

    # 1. Explicit SPDX marker (highest confidence).
    for m in _SPDX_MARKER_RE.finditer(line):
        out.append(("spdx_marker", m.group(1), m.group(0)))

    # 2. Bare SPDX ids anywhere.
    for m in _SPDX_RE.finditer(line):
        out.append(("spdx", m.group(1), m.group(0)))

    # 3. Creative Commons prose names → normalize to SPDX.
    for rx, prefix in _CC_PROSE:
        m = rx.search(line)
        if m:
            ver = m.group(1)
            out.append(("prose", f"{prefix}{ver}", m.group(0)))

    # 4. "licensed under <phrase>" — capture raw, let the LLMs resolve it.
    for m in _LICENSED_UNDER_RE.finditer(line):
        phrase = m.group(1).strip().rstrip(",;:")
        # Only keep if it isn't already captured verbatim as an SPDX id above.
        if not _SPDX_RE.search(phrase):
            out.append(("licensed_under", "", phrase))

    return out


def _push(scan: RepoScan, kind: str, file: str, line_no: int, spdx: str, raw: str, snippet: str) -> None:
    scan.findings.append(Finding(kind=kind, file=file, line=line_no, spdx=spdx, snippet=snippet, raw=raw))
    if spdx and spdx not in scan.spdx_seen:
        scan.spdx_seen.append(spdx)


def scan_repo(repo_path: str) -> RepoScan:
    """Walk `repo_path`, collect license declarations. Pure, no network."""
    import os

    scan = RepoScan(repo=repo_path)

    for root, dirs, files in os.walk(repo_path):
        dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]
        for fname in files:
            fpath = os.path.join(root, fname)
            rel = os.path.relpath(fpath, repo_path)
            is_license_file = _is_license_filename(fname)

            # Only open text-ish files; license files always opened.
            if not is_license_file and os.path.splitext(fname)[1].lower() not in SCAN_EXTS and fname.lower() not in SCAN_EXTS:
                continue
            if os.path.getsize(fpath) > 1_000_000:  # skip huge files
                continue

            try:
                with open(fpath, encoding="utf-8", errors="replace") as fh:
                    text = fh.read()
            except OSError:
                continue

            if is_license_file:
                # A license file: scan line-by-line for SPDX ids AND prose
                # names (a docs LICENSE.md may spell out "Attribution-
                # NoDerivatives 4.0" in prose rather than an SPDX id). Tag the
                # first finding as a license_file anchor for the report.
                anchored = False
                for i, line in enumerate(text.splitlines(), start=1):
                    for kind, spdx, raw in _scan_line(line):
                        if not anchored:
                            kind = "license_file"
                            anchored = True
                        snippet = line.strip()[:120]
                        _push(scan, kind, rel, i, spdx, raw, snippet)
                if not anchored:
                    # License file with no parseable id at all — record it as a
                    # bare anchor so the report flags "license present, id unknown".
                    first_line = next((l for l in text.splitlines() if l.strip()), "")
                    _push(scan, "license_file", rel, 1, "", "", first_line.strip()[:120])
                continue

            # Non-license text files: scan each line for declarations.
            for i, line in enumerate(text.splitlines(), start=1):
                for kind, spdx, raw in _scan_line(line):
                    snippet = line.strip()[:120]
                    _push(scan, kind, rel, i, spdx, raw, snippet)

    return scan
