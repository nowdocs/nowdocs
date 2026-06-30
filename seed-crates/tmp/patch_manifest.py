#!/usr/bin/env python3
"""Patch a docset manifest.json with correct legal + source fields.

Reads existing manifest from ~/.cache/nowdocs/db/<docset>.manifest.json,
updates legal (license, copyright_holder, attribution) and source
(source_url, entry_url, scraped_at), preserves all other fields.
"""

import json
import sys
from datetime import date
from pathlib import Path

CACHE = Path.home() / ".cache" / "nowdocs" / "db"


def patch(docset: str, license: str, copyright_holder: str, attribution: str,
          source_url: str, entry_url: str):
    mp = CACHE / f"{docset}.manifest.json"
    if not mp.is_file():
        print(f"ERROR: {mp} not found", file=sys.stderr)
        sys.exit(1)
    m = json.loads(mp.read_text(encoding="utf-8"))
    m["legal"] = {
        "license": license,
        "copyright_holder": copyright_holder,
        "attribution": attribution,
    }
    m["source"] = {
        **m.get("source", {}),
        "source_url": source_url,
        "entry_url": entry_url,
        "scraped_at": date.today().isoformat(),
    }
    mp.write_text(json.dumps(m, indent=2) + "\n", encoding="utf-8")
    print(f"patched {mp}: license={license} source={source_url}")


if __name__ == "__main__":
    # docset, license, copyright_holder, attribution, source_url, entry_url
    args = sys.argv[1:]
    if len(args) != 6:
        print("Usage: patch_manifest.py <docset> <license> <copyright_holder> <attribution> <source_url> <entry_url>")
        sys.exit(1)
    patch(*args)
