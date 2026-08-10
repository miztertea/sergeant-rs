#!/usr/bin/env python3
"""Final validation pass over P1.ndjson after A2/A9/A11/A12 fixups."""
import json
import re
import hashlib
import sys

CORPUS = "reference-corpus/behavior-units/P1.ndjson"
LOCATOR_RE = re.compile(r"^(?P<file>\S+\.md) L(?P<start>\d+)(?:-(?P<end>\d+))?")
REQUIRED_TOP = ["id", "statement", "source", "scope", "trigger", "outcome",
                "authority", "confidence", "notes", "representation",
                "workflow", "stage", "rationale", "alternatives_considered",
                "engine_gap"]
REQUIRED_SRC = ["path", "locator", "quote", "quote_hash"]
VALID_REPS = {"agents-invariant", "workflow", "stage", "stage-context",
              "helper", "shared-helper", "shared-context",
              "obsolete-mechanism", "engine-gap"}
VALID_CONF = {"high", "medium", "low"}

problems = []

with open(CORPUS, "r", encoding="utf-8") as f:
    raw_lines = [l for l in f if l.strip()]

units = []
for i, l in enumerate(raw_lines):
    try:
        units.append(json.loads(l))
    except json.JSONDecodeError as e:
        problems.append(f"line {i+1}: invalid JSON: {e}")

print(f"Total lines/units: {len(units)}")

# id uniqueness + stability of numbering
ids = [u["id"] for u in units]
if len(ids) != len(set(ids)):
    dupes = {i for i in ids if ids.count(i) > 1}
    problems.append(f"duplicate ids: {dupes}")

expected_ids = {f"BU-P1-{n:03d}" for n in range(1, 138)}
actual_ids = set(ids)
if expected_ids != actual_ids:
    problems.append(f"id set mismatch: missing={expected_ids-actual_ids} extra={actual_ids-expected_ids}")

# field completeness
text_cache = {}
for u in units:
    for k in REQUIRED_TOP:
        if k not in u:
            problems.append(f"{u.get('id','?')}: missing top-level field {k}")
    src = u.get("source", {})
    for k in REQUIRED_SRC:
        if k not in src:
            problems.append(f"{u.get('id','?')}: missing source.{k}")

    if u.get("representation") not in VALID_REPS:
        problems.append(f"{u['id']}: invalid representation {u.get('representation')}")
    if u.get("confidence") not in VALID_CONF:
        problems.append(f"{u['id']}: invalid confidence {u.get('confidence')}")
    if u.get("rationale") in (None, ""):
        problems.append(f"{u['id']}: null/empty rationale")
    if len(u.get("statement", "")) == 0:
        problems.append(f"{u['id']}: empty statement")
    if src.get("quote", "") == "":
        problems.append(f"{u['id']}: empty quote")

    rep = u.get("representation")
    if rep in ("workflow", "stage") and u.get("alternatives_considered") == []:
        problems.append(f"{u['id']}: {rep} unit has empty alternatives_considered")
    if rep == "engine-gap" and (u.get("engine_gap") in (None, {})):
        problems.append(f"{u['id']}: engine-gap unit missing engine_gap payload")
    if rep != "engine-gap" and u.get("engine_gap") not in (None,):
        problems.append(f"{u['id']}: non-engine-gap unit has non-null engine_gap")

    # re-verify hash against source span independently
    quote = src.get("quote")
    quote_hash = src.get("quote_hash")
    path = src.get("path")
    if path and quote and quote_hash:
        if path not in text_cache:
            with open(path, "r", encoding="utf-8", newline="") as f:
                text_cache[path] = f.read()
        text = text_cache[path]
        if quote not in text:
            problems.append(f"{u['id']}: quote not found contiguously in {path}")
        loc = src.get("locator", "")
        m = LOCATOR_RE.match(loc)
        if m:
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
                problems.append(f"{u['id']}: recomputed hash mismatch {digest} != {quote_hash}")
            if len(span) > 500:
                if quote != span[:500]:
                    problems.append(f"{u['id']}: long-span quote is not first-500-char prefix")
                if src.get("span_bytes") != len(span.encode('utf-8')):
                    problems.append(f"{u['id']}: span_bytes mismatch")

print(f"Problems found: {len(problems)}")
for p in problems:
    print("  -", p)

sys.exit(1 if problems else 0)
