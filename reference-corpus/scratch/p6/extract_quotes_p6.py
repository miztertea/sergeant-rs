#!/usr/bin/env python3
"""A2 for P6: derive quote + quote_hash for every BU-P6-* unit from its cited locator.

Reads reference-corpus/behavior-units/P6.ndjson, resolves each unit's
source.locator to one or more L<start>-<end> (or L<n>) line ranges, and
extracts the EXACT contiguous byte span for the FIRST range named in the
locator (P6 locators frequently cite several supporting ranges,
comma-separated, sometimes prefixed by a dotted key path like
"tasks.check, _check_td, L169-172" or suffixed by a parenthetical
cross-reference; the first L<range> is consistently where the extractor
anchored the statement's core claim). No reconstruction, no normalization
beyond the span choice itself.

Convention (matches reference-corpus/scratch/extract_quotes.py's P1
precedent): the span runs from the first character of the start line to the
last character of the end line -- internal newlines are included, the
span's own trailing newline is not. Hash is sha256 over the span's exact
UTF-8 bytes. Longer-than-500-char spans keep their first 500 chars as
`quote` and record the full byte length as `span_bytes`.

Units whose primary range does not yield a span at all (out-of-range line
numbers, missing file) are reported, not silently patched.
"""
import json
import re
import hashlib
import sys

CORPUS = "reference-corpus/behavior-units/P6.ndjson"
RANGE_RE = re.compile(r"L(\d+)(?:-(\d+))?")


def load_lines(path, cache):
    if path not in cache:
        with open(path, "r", encoding="utf-8", newline="") as f:
            content = f.read()
        cache[path] = content.splitlines(keepends=True)
    return cache[path]


def extract_span(lines, start, end):
    span = "".join(lines[start - 1:end])
    if span.endswith("\n"):
        span = span[:-1]
    if span.endswith("\r"):
        span = span[:-1]
    return span


def all_ranges(locator):
    return [(int(m.group(1)), int(m.group(2)) if m.group(2) else int(m.group(1)))
            for m in RANGE_RE.finditer(locator)]


def main():
    with open(CORPUS, "r", encoding="utf-8") as f:
        units = [json.loads(l) for l in f if l.strip()]

    cache = {}
    report = []
    for u in units:
        src = u["source"]
        loc = src["locator"]
        path = src["path"]
        ranges = all_ranges(loc)
        if not ranges:
            report.append((u["id"], "NO_RANGE_FOUND", loc))
            continue
        start, end = ranges[0]

        try:
            lines = load_lines(path, cache)
        except OSError as e:
            report.append((u["id"], "FILE_ERROR", str(e)))
            continue

        if start < 1 or end > len(lines) or start > end:
            report.append((u["id"], "OUT_OF_RANGE", f"{start}-{end} vs {len(lines)} lines"))
            continue

        span = extract_span(lines, start, end)
        if span.strip() == "":
            report.append((u["id"], "EMPTY_SPAN", f"{start}-{end}"))
            continue

        span_bytes_full = span.encode("utf-8")
        digest = hashlib.sha256(span_bytes_full).hexdigest()

        if len(span) <= 500:
            quote = span
            span_bytes_field = None
        else:
            quote = span[:500]
            span_bytes_field = len(span_bytes_full)

        old_hash = src.get("quote_hash")
        new_hash = f"sha256:{digest}"

        src["quote"] = quote
        if span_bytes_field is not None:
            src["span_bytes"] = span_bytes_field
        else:
            src.pop("span_bytes", None)
        src["quote_hash"] = new_hash

        if old_hash != new_hash:
            report.append((u["id"], "HASH_CHANGED", f"{old_hash} -> {new_hash}"))
        if len(ranges) > 1:
            report.append((u["id"], "MULTI_RANGE_FIRST_USED", f"used {start}-{end} of {ranges}"))

    with open(CORPUS, "w", encoding="utf-8") as f:
        for u in units:
            f.write(json.dumps(u, ensure_ascii=False) + "\n")

    print(f"Processed {len(units)} units.")
    changed = [r for r in report if r[1] == "HASH_CHANGED"]
    multi = [r for r in report if r[1] == "MULTI_RANGE_FIRST_USED"]
    problems = [r for r in report if r[1] not in ("HASH_CHANGED", "MULTI_RANGE_FIRST_USED")]
    print(f"Hashes changed: {len(changed)}")
    print(f"Multi-range locators (first range used): {len(multi)}")
    print(f"PROBLEMS: {len(problems)}")
    for r in problems:
        print("  ", r)
    print("--- multi-range detail ---")
    for r in multi:
        print("  ", r)


if __name__ == "__main__":
    main()
