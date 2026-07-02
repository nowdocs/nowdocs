"""Unit tests for audit.classify_exit — pure, no network, no LLMs.

Run:  python3 scripts/audit_license/test_audit.py
Exits non-zero on any failure.

Covers the exit-code contract documented in audit.py:
  0 = clean (agree + allowlisted), 1 = needs human otherwise.
The MPL-2.0 regression (agree but not allowlisted → must be 1, not 0) is
the bug these tests pin down.
"""

import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from audit import classify_exit  # noqa: E402

FAILURES: list[str] = []


def check(name: str, cond: bool, detail: str = "") -> None:
    if cond:
        print(f"  ok   {name}")
    else:
        print(f"  FAIL {name}  {detail}")
        FAILURES.append(name)


class _Scan:
    """Minimal scan stub: dangerous() returns a configurable list."""

    def __init__(self, dangerous_ids=None):
        self._d = dangerous_ids or []

    def dangerous(self):
        return self._d


def _v(license_id: str) -> dict:
    return {"docs_license": license_id, "rationale": "stub"}


def test_agree_allowlisted_is_clean():
    scan = _Scan()
    rc = classify_exit(scan, _v("MIT"), _v("MIT"))
    check("MIT agree+allowlisted -> 0", rc == 0, str(rc))


def test_agree_not_allowlisted_is_not_clean():
    # The bug: MPL-2.0 is agreed and not dangerous, but not in ALLOWLIST.
    # A wrapper gating on "0 = clean" would wave it through; must return 1.
    scan = _Scan()
    rc = classify_exit(scan, _v("MPL-2.0"), _v("MPL-2.0"))
    check("MPL-2.0 agree but not allowlisted -> 1", rc == 1, str(rc))


def test_disagree_is_not_clean():
    scan = _Scan()
    rc = classify_exit(scan, _v("MIT"), _v("Apache-2.0"))
    check("disagree -> 1", rc == 1, str(rc))


def test_dangerous_regex_is_not_clean():
    scan = _Scan(["CC-BY-ND-4.0"])
    rc = classify_exit(scan, _v("CC-BY-ND-4.0"), _v("CC-BY-ND-4.0"))
    check("regex dangerous -> 1", rc == 1, str(rc))


def test_dangerous_verdict_is_not_clean():
    scan = _Scan()
    rc = classify_exit(scan, _v("CC-BY-NC-SA-4.0"), _v("CC-BY-NC-SA-4.0"))
    check("verdict dangerous (NC-SA) -> 1", rc == 1, str(rc))


def test_unknown_is_not_clean():
    scan = _Scan()
    rc = classify_exit(scan, _v("UNKNOWN"), _v("UNKNOWN"))
    check("UNKNOWN -> 1", rc == 1, str(rc))


def test_cc_by_allowlisted_is_clean():
    scan = _Scan()
    rc = classify_exit(scan, _v("CC-BY-4.0"), _v("CC-BY-4.0"))
    check("CC-BY-4.0 agree+allowlisted -> 0", rc == 0, str(rc))


def main() -> int:
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            print(f"[{name}]")
            fn()
    print()
    if FAILURES:
        print(f"FAILED: {len(FAILURES)} -> {FAILURES}")
        return 1
    print("all audit tests passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
