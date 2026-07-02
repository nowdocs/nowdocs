#!/usr/bin/env python3
"""Rebuild a markdown directory tree from a share artifact's chunks.jsonl.

The upstream source docs are gitignored (rebuildable intermediate), but
seed-crates/share/nextjs-docs/chunks.jsonl carries every chunk's text +
source_url. Group chunks by source_url, concat by idx, write each group to
<out>/<source_url>. ingest_dir then produces source_urls matching the
original, so golden-query expected_source_urls line up.
"""
import json
import os
import sys
from collections import defaultdict

SRC = "seed-crates/share/nextjs-docs/chunks.jsonl"
OUT = "seed-crates/tmp/nextjs_rebuilt"


def main():
    if not os.path.exists(SRC):
        sys.exit(f"missing {SRC}")
    groups = defaultdict(list)
    with open(SRC) as f:
        for line in f:
            c = json.loads(line)
            groups[c["source_url"]].append(c)
    n = 0
    for url, chunks in groups.items():
        chunks.sort(key=lambda c: c["idx"])
        path = os.path.join(OUT, url)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w") as wf:
            wf.write("\n\n".join(c["text"] for c in chunks))
        n += 1
    print(f"wrote {n} files to {OUT}")


if __name__ == "__main__":
    main()
