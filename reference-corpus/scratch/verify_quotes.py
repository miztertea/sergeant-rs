#!/usr/bin/env python3
"""Independent verifier for BU-P1-* quote_hash values.

For every unit: (1) confirms the recorded `quote` (or, for long spans, its
first 500 chars) appears verbatim/contiguously in the cited source file, and
(2) recomputes sha256 over the exact locator-derived span and checks it
equals the recorded quote_hash. Exits nonzero on any failure.
"""
import json
import re
import hashlib
import sys

CORPUS = "reference-corpus/behavior-units/P1.ndjson"
LOCATOR_RE = re.compile(r"^(?P<file>\S+\.md) L(?P<start>\d+)(?:-(?P<end>\d+))?")


def load_source_text(path):
    with open(path, "r", encoding="utf-8", newline="") as f:
        return f.read()


def main():
    with open(CORPUS, "r", encoding="utf-8") as f:
        units = [json.loads(l) for l in f if l.strip()]

    text_cache = {}
    fails = []
    for u in units:
        src = u["source"]
        quote = src.get("quote")
        quote_hash = src.get("quote_hash")
        if quote is None or quote_hash is None:
            fails.append((u["id"], "MISSING_FIELD"))
            continue

        path = src["path"]
        if path not in text_cache:
            text_cache[path] = load_source_text(path)
        text = text_cache[path]

        # contiguity check: the recorded quote text (verbatim prefix for long
        # spans) must appear as a literal substring of the source file.
        if quote not in text:
            fails.append((u["id"], "QUOTE_NOT_CONTIGUOUS_IN_SOURCE"))
            continue

        # recompute from the locator's line range independently of the quote field
        loc = src["locator"]
        m = LOCATOR_RE.match(loc)
        if not m:
            fails.append((u["id"], f"LOCATOR_UNPARSEABLE: {loc}"))
            continue
        start = int(m.group("start"))
        end = int(m.group("end")) if m.group("end") else start
        lines = text.splitlines(keepends=True)
        span = "".join(lines[start - 1:end])
        if span.endswith("\n"):
            span = span[:-1]
        if span.endswith("\r"):
            span = span[:-1]
        digest = "sha256:" + hashlib.sha256(span.encode("utf-8")).hexdigest()

        if digest != quote_hash:
            fails.append((u["id"], f"HASH_MISMATCH recomputed={digest} recorded={quote_hash}"))
            continue

        # span_bytes sanity for long spans
        if len(span) > 500:
            if src.get("span_bytes") != len(span.encode("utf-8")):
                fails.append((u["id"], "SPAN_BYTES_MISMATCH"))
            if quote != span[:500]:
                fails.append((u["id"], "QUOTE_PREFIX_MISMATCH"))

    print(f"Checked {len(units)} units.")
    if fails:
        print(f"FAILURES: {len(fails)}")
        for f_ in fails:
            print("  ", f_)
        sys.exit(1)
    print("All quote_hash values verified against source spans.")


if __name__ == "__main__":
    main()
