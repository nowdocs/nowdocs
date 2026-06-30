#!/usr/bin/env python3
"""Prepare docs repos for nowdocs ingest.

Strips JSX components, image references, and HTML comments from MDX/markdown
sources. Pure text only — no images, no interactive components.
"""

import os
import re
import shutil
import sys
from pathlib import Path


def strip_mdx(text: str) -> str:
    """Remove JSX/HTML tags, image references, comments, and embedded JSX expressions."""
    # Strip HTML/JSX comments: <!-- ... -->
    text = re.sub(r"<!--.*?-->", "", text, flags=re.DOTALL)
    # Strip markdown image references: ![alt](url)
    text = re.sub(r"!\[[^\]]*\]\([^)]*\)", "", text)
    # Strip HTML <img> tags
    text = re.sub(r"<img[^>]*/?>", "", text, flags=re.IGNORECASE)
    # Strip multi-line JSX components: <Component ...>...</Component>
    # First handle paired components with content (greedy across newlines)
    text = re.sub(r"<[A-Z][A-Za-z0-9]*[^>]*>.*?</[A-Z][A-Za-z0-9]*>", "", text, flags=re.DOTALL)
    # Strip self-closing JSX: <Component prop="value" />
    text = re.sub(r"<[A-Z][A-Za-z0-9]*[^>]*/>", "", text)
    # Strip remaining opening/closing JSX tags: <Foo> or </Foo>
    text = re.sub(r"</?[A-Z][A-Za-z0-9]*[^>]*>", "", text)
    # Strip HTML tags: <div>, <a href="..."> (lowercase, paired or self-closing)
    text = re.sub(r"<[a-z][^>]*/?>", "", text)
    text = re.sub(r"</[a-z][^>]*>", "", text)
    # Strip JSX expressions: {`code`} or {variable} or {expr.method()}
    text = re.sub(r"\{`[^`]*`\}", "", text)
    text = re.sub(r"\{[^{}]*\}", "", text)
    return text


def process_file(src: Path, dst: Path) -> int:
    """Strip MDX/JSX from src, write to dst. Returns 1 if kept, 0 if dropped."""
    if src.stat().st_size > 1_000_000:  # skip files > 1MB
        return 0
    try:
        text = src.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return 0
    cleaned = strip_mdx(text)
    # Drop files that are now effectively empty or just frontmatter
    lines = [l for l in cleaned.splitlines() if l.strip()]
    if len(lines) < 3:
        return 0
    # Collapse runs of blank lines
    cleaned = re.sub(r"\n{3,}", "\n\n", cleaned)
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_text(cleaned, encoding="utf-8")
    return 1


def main():
    if len(sys.argv) < 4:
        print("Usage: prep_docs.py <src_dir> <dst_dir> <exclude_substr>")
        sys.exit(1)
    src_root = Path(sys.argv[1])
    dst_root = Path(sys.argv[2])
    exclude = sys.argv[3] if len(sys.argv) > 3 else ""

    count = 0
    for src in src_root.rglob("*.md*"):
        rel = src.relative_to(src_root)
        if exclude and exclude in str(rel):
            continue
        if src.suffix in (".md", ".mdx"):
            # Normalize extension to .md
            dst = dst_root / rel.with_suffix(".md")
            count += process_file(src, dst)
    print(f"wrote {count} files to {dst_root}")


if __name__ == "__main__":
    main()
