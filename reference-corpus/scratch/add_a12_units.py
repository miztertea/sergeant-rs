#!/usr/bin/env python3
"""A12: extract AGENTS.md's 'Procedural skills' trigger->skill->owns routing
table (L111-118, six rows) as new units continuing the BU-P1- id range.
Finding N1-R3-08 cites L111-118. Header/preface (L107-109) is already
covered by BU-P1-021 (agents-invariant, 'load procedures only when their
trigger applies'); this adds one unit per of the six table rows (L113-L118).
"""
import json
import hashlib

CORPUS = "reference-corpus/behavior-units/P1.ndjson"
SRC_PATH = "reference/sergeant-upstream/AGENTS.md"

ROWS = [
    {
        "line": 113,
        "statement": "When a project is named, registered, edited, synced, or graphed, load the load-project procedure, which owns registry lookup, schema, context loading, project edits, sync, and project Graphify.",
        "trigger": "a project is named, registered, edited, synced, or graphed",
        "skill": "load-project",
    },
    {
        "line": 114,
        "statement": "When more than one repository owns the requested outcome, load the cross-repo-work procedure, which owns repository decomposition, dependency and merge order, and per-repository acceptance.",
        "trigger": "more than one repository owns the requested outcome",
        "skill": "cross-repo-work",
    },
    {
        "line": 115,
        "statement": "When dispatch mode is selected or an existing fleet must be operated, load the dispatch procedure, which owns task-tracker integration, worktrees, worker contracts, monitoring, escalation, reconciliation, and cleanup.",
        "trigger": "dispatch mode is selected or an existing fleet must be operated",
        "skill": "dispatch",
    },
    {
        "line": 116,
        "statement": "When the user asks to ingest, backfill, regenerate, inspect, update, or change the wiki, load the wiki procedure, which owns capture behavior, digest generation, schema ownership, and index updates.",
        "trigger": "the user asks to ingest, backfill, regenerate, inspect, update, or change the wiki",
        "skill": "wiki",
    },
    {
        "line": 117,
        "statement": "When the user asks how to install, configure, use, or troubleshoot Sergeant in a read-only capacity, load the sergeant-help procedure, which owns documentation lookup, command verification, prerequisites, and help responses.",
        "trigger": "the user asks how to install, configure, use, or troubleshoot Sergeant (read-only)",
        "skill": "sergeant-help",
    },
    {
        "line": 118,
        "statement": "When the user wants to interactively install, configure, or repair a Sergeant installation, load the sergeant-setup procedure, which owns the interactive setup wizard, prerequisite detection, consent-gated install, project YAML interview, sync verification, and an optional treehouse prompt.",
        "trigger": "the user wants to interactively install, configure, or repair a Sergeant installation",
        "skill": "sergeant-setup",
    },
]


def load_lines(path):
    with open(path, "r", encoding="utf-8", newline="") as f:
        content = f.read()
    return content.splitlines(keepends=True)


def span_for_line(lines, n):
    span = lines[n - 1]
    if span.endswith("\n"):
        span = span[:-1]
    if span.endswith("\r"):
        span = span[:-1]
    return span


def main():
    with open(CORPUS, "r", encoding="utf-8") as f:
        units = [json.loads(l) for l in f if l.strip()]
    existing_ids = {u["id"] for u in units}
    max_n = max(int(u["id"].split("-")[-1]) for u in units)
    assert max_n == 131, f"expected corpus to currently end at 131, found {max_n}"

    lines = load_lines(SRC_PATH)
    new_units = []
    for i, row in enumerate(ROWS):
        new_id = f"BU-P1-{132 + i:03d}"
        assert new_id not in existing_ids

        span = span_for_line(lines, row["line"])
        digest = hashlib.sha256(span.encode("utf-8")).hexdigest()
        quote_hash = f"sha256:{digest}"
        assert len(span) <= 500

        unit = {
            "id": new_id,
            "statement": row["statement"],
            "source": {
                "path": SRC_PATH,
                "locator": f"AGENTS.md L{row['line']}, Procedural skills table row ({row['skill']})",
                "quote": span,
                "quote_hash": quote_hash,
            },
            "scope": "skill loading",
            "trigger": row["trigger"],
            "outcome": f"the {row['skill']} procedure is loaded to own its listed responsibilities",
            "authority": "any actor",
            "confidence": "high",
            "notes": (
                "N1 adjudication A12 (finding N1-R3-08, AGENTS.md L111-118, six rows): "
                "one row of the Procedural skills routing table; the table's preface "
                "('Load procedures only when their trigger applies', L109) is already "
                "captured as BU-P1-021. Overlaps docs/skills.md's 'Sergeant-owned skills' "
                "table (BU-P1-117), which is the same routing information restated with a "
                "location column instead of an owns column; likely merges with it at "
                "synthesis."
            ),
            "representation": "shared-context",
            "workflow": "@@procedural-skills-routing",
            "stage": None,
            "rationale": (
                "One row of the trigger-to-skill-to-owns routing table every mode-selection "
                "and skill-loading procedure consults; the routing table is a single named "
                "reference multiple otherwise-independent procedures look up, not a checkpoint "
                "of any one workflow, and finer-grained than the general 'load procedures only "
                "when triggered' preface already captured as its own agents-invariant "
                "(BU-P1-021)."
            ),
            "alternatives_considered": [
                "agents-invariant (rejected: too coarse — BU-P1-021 already captures the general load-on-trigger rule; this row is the specific routing entry)",
                "workflow (rejected: this is a lookup-table entry, not itself a triggered bounded procedure)",
            ],
            "engine_gap": None,
        }
        new_units.append(unit)

    units.extend(new_units)
    with open(CORPUS, "w", encoding="utf-8") as f:
        for u in units:
            f.write(json.dumps(u, ensure_ascii=False) + "\n")

    print(f"Added {len(new_units)} new units: {[u['id'] for u in new_units]}")


if __name__ == "__main__":
    main()
