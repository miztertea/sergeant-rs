#!/usr/bin/env python3
"""A2: derive quote + quote_hash for every BU-P1-* unit from its cited locator.

Reads reference-corpus/behavior-units/P1.ndjson, resolves each unit's
source.locator to an explicit L<start>-<end> (or L<n>) line range, extracts
the EXACT contiguous byte span from the cited source file (no reconstruction,
no normalization), and writes source.quote / source.quote_hash accordingly.

Convention (recorded here since docs/icm/record-shapes.md leaves the newline
handling to the extractor): the span runs from the first character of the
start line to the last character of the end line, i.e. internal newlines
between lines are included but the span's own trailing newline is not.
The hash is sha256 over that span's exact UTF-8 bytes.
"""
import json
import re
import hashlib
import sys

CORPUS = "reference-corpus/behavior-units/P1.ndjson"

LOCATOR_RE = re.compile(r"^(?P<file>\S+\.md) L(?P<start>\d+)(?:-(?P<end>\d+))?")


def load_source_lines(path):
    with open(path, "r", encoding="utf-8", newline="") as f:
        content = f.read()
    # keepends so we can reconstruct exact byte spans including internal newlines
    return content.splitlines(keepends=True)


def extract_span(lines, start, end):
    # lines is 0-indexed list from splitlines(keepends=True); start/end are 1-indexed inclusive
    span = "".join(lines[start - 1:end])
    if span.endswith("\n"):
        span = span[:-1]
    if span.endswith("\r"):
        span = span[:-1]
    return span


def main():
    with open(CORPUS, "r", encoding="utf-8") as f:
        units = [json.loads(l) for l in f if l.strip()]

    source_cache = {}
    report = []
    for u in units:
        loc = u["source"]["locator"]
        path = u["source"]["path"]
        m = LOCATOR_RE.match(loc)
        if not m:
            report.append((u["id"], "NO_MATCH", loc))
            continue
        start = int(m.group("start"))
        end = int(m.group("end")) if m.group("end") else start

        if path not in source_cache:
            source_cache[path] = load_source_lines(path)
        lines = source_cache[path]

        if start < 1 or end > len(lines):
            report.append((u["id"], "OUT_OF_RANGE", f"{start}-{end} vs {len(lines)} lines"))
            continue

        span = extract_span(lines, start, end)
        span_bytes_full = span.encode("utf-8")
        digest = hashlib.sha256(span_bytes_full).hexdigest()

        if len(span) <= 500:
            quote = span
            span_bytes_field = None
        else:
            quote = span[:500]
            span_bytes_field = len(span_bytes_full)

        u["source"]["quote"] = quote
        if span_bytes_field is not None:
            u["source"]["span_bytes"] = span_bytes_field
        else:
            u["source"].pop("span_bytes", None)
        old_hash = u["source"]["quote_hash"]
        new_hash = f"sha256:{digest}"
        if old_hash != new_hash:
            report.append((u["id"], "HASH_CHANGED", f"{old_hash} -> {new_hash}"))
        u["source"]["quote_hash"] = new_hash

    with open(CORPUS, "w", encoding="utf-8") as f:
        for u in units:
            f.write(json.dumps(u, ensure_ascii=False) + "\n")

    print(f"Processed {len(units)} units.")
    changed = [r for r in report if r[1] == "HASH_CHANGED"]
    other = [r for r in report if r[1] != "HASH_CHANGED"]
    print(f"Hashes changed: {len(changed)}")
    print(f"Other issues: {len(other)}")
    for r in other:
        print("  ", r)


if __name__ == "__main__":
    main()
