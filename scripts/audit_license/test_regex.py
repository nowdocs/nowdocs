"""Unit tests for regex_rules — pure, no network, runnable standalone.

Run:  python3 scripts/audit_license/test_regex.py
Exits non-zero on any failure (suitable for a pre-commit/ci hook).
"""

import os
import sys
import tempfile

sys.path.insert(0, os.path.dirname(__file__))
from regex_rules import scan_repo, _scan_line, DANGEROUS  # noqa: E402

FAILURES: list[str] = []


def check(name: str, cond: bool, detail: str = "") -> None:
    if cond:
        print(f"  ok   {name}")
    else:
        print(f"  FAIL {name}  {detail}")
        FAILURES.append(name)


def line_spdx(line: str) -> list[str]:
    return [spdx for _kind, spdx, _raw in _scan_line(line) if spdx]


# --- line-level extraction ---

def test_spdx_marker():
    ids = line_spdx("SPDX-License-Identifier: Apache-2.0")
    check("spdx_marker extracts Apache-2.0", "Apache-2.0" in ids, str(ids))


def test_bare_mit_not_in_word():
    # "MIT" must not match inside "MITIGATION".
    ids = line_spdx("This is MITIGATION logic.")
    check("MIT not matched inside MITIGATION", "MIT" not in ids, str(ids))


def test_bare_mit_word_boundary():
    ids = line_spdx("Licensed under the MIT License.")
    check("MIT matched as word", "MIT" in ids, str(ids))


def test_cc_by_nd_caught():
    # The critical case: ND modifier must not collapse to plain CC-BY-4.0.
    ids = line_spdx("docs are licensed under CC-BY-ND-4.0")
    check("CC-BY-ND-4.0 caught", "CC-BY-ND-4.0" in ids, str(ids))
    check("...not misread as CC-BY-4.0", "CC-BY-4.0" not in ids, str(ids))


def test_cc_by_sa_caught():
    ids = line_spdx("licensed under CC-BY-SA-4.0 International")
    check("CC-BY-SA-4.0 caught", "CC-BY-SA-4.0" in ids, str(ids))
    check("...not misread as CC-BY-4.0", "CC-BY-4.0" not in ids, str(ids))


def test_cc_prose_noderivatives():
    # Natural language: "Attribution-NoDerivatives 4.0" -> CC-BY-ND-4.0
    ids = line_spdx("under a Creative Commons Attribution-NoDerivatives 4.0 license")
    check("prose NoDerivatives -> CC-BY-ND-4.0", "CC-BY-ND-4.0" in ids, str(ids))


def test_cc_prose_sharealike():
    ids = line_spdx("Creative Commons Attribution-ShareAlike 4.0 International")
    check("prose ShareAlike -> CC-BY-SA-4.0", "CC-BY-SA-4.0" in ids, str(ids))


def test_cc_prose_plain_attribution():
    ids = line_spdx("Creative Commons Attribution 4.0 International")
    check("prose plain -> CC-BY-4.0", "CC-BY-4.0" in ids, str(ids))


def test_gpl_caught():
    ids = line_spdx("This file is GPL-3.0 licensed.")
    check("GPL-3.0 caught", "GPL-3.0" in ids, str(ids))


def test_licensed_under_phrase_kept_when_not_spdx():
    kinds = [k for k, _s, _r in _scan_line("released under a custom proprietary license.")]
    check("licensed_under phrase captured", "licensed_under" in kinds, str(kinds))


def test_licensed_under_spdx_not_duplicated():
    # "licensed under MIT" -> SPDX id captured, not also as licensed_under raw.
    kinds = [k for k, s, _r in _scan_line("licensed under MIT")]
    check("MIT via spdx not licensed_under", "spdx" in kinds and "licensed_under" not in kinds, str(kinds))


# --- repo-level scan ---

def test_repo_scan_finds_root_license_and_readme_prose():
    with tempfile.TemporaryDirectory() as d:
        with open(os.path.join(d, "LICENSE"), "w") as f:
            f.write("MIT License\n\nCopyright (c) X\n")
        os.makedirs(os.path.join(d, "docs"))
        with open(os.path.join(d, "docs", "README.md"), "w") as f:
            f.write("# Docs\n\nDocumentation is licensed under CC-BY-4.0.\n")
        # A vendored dir that must be skipped.
        os.makedirs(os.path.join(d, "node_modules", "pkg"))
        with open(os.path.join(d, "node_modules", "pkg", "LICENSE"), "w") as f:
            f.write("GPL-3.0\n")

        scan = scan_repo(d)
        seen = set(scan.spdx_seen)
        check("root LICENSE -> MIT", "MIT" in seen, str(scan.spdx_seen))
        check("docs README prose -> CC-BY-4.0", "CC-BY-4.0" in seen, str(scan.spdx_seen))
        check("node_modules excluded (no GPL-3.0)", "GPL-3.0" not in seen, str(scan.spdx_seen))
        check("dangerous() empty for MIT+CC-BY-4.0", scan.dangerous() == [], str(scan.dangerous()))


def test_repo_scan_flags_dangerous_nd():
    with tempfile.TemporaryDirectory() as d:
        os.makedirs(os.path.join(d, "docs"))
        with open(os.path.join(d, "docs", "LICENSE.md"), "w") as f:
            f.write("Creative Commons Attribution-NoDerivatives 4.0\n")
        scan = scan_repo(d)
        check("CC-BY-ND-4.0 flagged dangerous", "CC-BY-ND-4.0" in scan.dangerous(), str(scan.dangerous()))


def main() -> int:
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            print(f"[{name}]")
            fn()
    print()
    if FAILURES:
        print(f"FAILED: {len(FAILURES)} -> {FAILURES}")
        return 1
    print("all regex tests passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
