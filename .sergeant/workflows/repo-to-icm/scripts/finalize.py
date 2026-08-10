#!/usr/bin/env python3
"""finalize.py — the D9 disposition finalize helper for repo-to-icm itself.

Layer 3 (`_config`-adjacent) deterministic machinery, invoked once by this
workflow's own closing stage, `90-reconcile` (`docs/icm/convention.md` §1a,
D9 finalize convention; `docs/gauntlet/contracts/N2.md` U3).

WHAT THIS DOES AND DOES NOT DO
-------------------------------
At the end of a run, every stage of *this* workflow (`00-contract` through
`90-reconcile`) has its own `output/` directory materialized in the run
worktree, each declaring — in that stage's `output/README.md` — the
artifact(s) it produces and a disposition:

    promote   survives into this run's own Work-branch merge
    evidence  Work-branch record only; removed at finalize

This script reads every stage's declared disposition and applies it
mechanically: `promote`d files are left alone, `evidence`-class and
UNDECLARED files are removed in one closing commit ("silence promotes
nothing" — removed files remain recoverable from Work-branch history, per
convention). It does not decide whether an artifact *should* exist — that
judgment belongs to the stage that wrote it (`docs/icm/convention.md` §1a:
"Judgment about whether an artifact should exist belongs to the stage that
writes it, not to finalize"). If a stage's `output/` holds files but its
`output/README.md` gives this script nothing to go on, that is ambiguity,
and per this codebase's invariant (`CLAUDE.md`: "Ambiguity fails closed ...
never a guess") this script refuses to touch that stage's output at all and
exits nonzero, rather than guess.

This script does NOT touch a generated draft workflow package's own
`output/` directories (e.g. under `.sergeant/drafts/workflows/<candidate>/`)
— those are per-run artifact declarations for that *candidate's future
runs*, not artifacts of the current repo-to-icm run, and are never
populated at draft-materialization time (`docs/icm/convention.md` §1a rule
4: "An output/ directory in the authored tree contains only a README.md").

DISPOSITION DECLARATION FORMAT (what an output/README.md must contain)
------------------------------------------------------------------------
The common case — one artifact, one disposition for the whole stage:

    **Expected artifact:** `contract.md` — ...
    **Disposition:** `promote`

binds `promote` to exactly the filename(s) named after "Expected
artifact:" (`contract.md` here) — NOT to every file the stage happens to
leave under `output/`. A stray scratch file the actor also left behind is
UNDECLARED, not silently covered by the stage's one disposition line.

For a stage that produces more than one artifact with different
dispositions, a table takes precedence over the Expected-artifact/
Disposition pairing above:

    | File | Disposition |
    |---|---|
    | contract.md | promote |
    | scratch-notes.md | evidence |

Any file present under `output/` that neither form names is UNDECLARED and
is removed exactly as `evidence` is — declaring nothing promotes nothing.

USAGE
-----
    python3 finalize.py [WORKFLOW_ROOT]

WORKFLOW_ROOT defaults to this script's own workflow directory (the parent
of `scripts/`) — the ordinary case when invoked by `90-reconcile` in the
materialized run worktree. An explicit argument is accepted for review or
testing against a copy.

Exits 0 on a clean apply (including "nothing to do"), 1 if any stage's
output could not be resolved unambiguously (nothing is modified in that
case), 2 on a usage/environment error.
"""

import os
import re
import subprocess
import sys

DISPOSITION_LINE_RE = re.compile(r"\*\*Disposition:\*\*\s*`([a-zA-Z_-]+)`")
TABLE_ROW_RE = re.compile(r"^\|\s*([^|]+?)\s*\|\s*([a-zA-Z_-]+)\s*\|\s*$")
STAGE_DIR_RE = re.compile(r"^\d")

KNOWN_DISPOSITIONS = {"promote", "evidence"}


ARTIFACT_NAMES_RE = re.compile(
    r"\*\*Expected artifact:\*\*\s*((?:`[^`]+`(?:\s*,\s*|\s+and\s+)?)+)"
)
NAME_TOKEN_RE = re.compile(r"`([^`]+)`")


class StagePlan:
    def __init__(self, stage_id):
        self.stage_id = stage_id
        self.keep = []       # files kept (promote)
        self.remove = []     # files removed (evidence / undeclared)
        self.error = None    # str | None — set means this stage is skipped


def parse_readme(text):
    """Returns (table: dict[filename, disposition] | None,
                declared: dict[filename, disposition]).

    `table` (a `| File | Disposition |` block) wins outright when present —
    it is the only form that can name more than one artifact with different
    dispositions unambiguously.

    Otherwise `declared` is built by binding each filename named in a
    `**Expected artifact:** \`name\` — ...` line to the disposition token in
    exactly one following `**Disposition:** \`x\`` line (the common,
    single-artifact-per-stage shape this workflow's own stages currently
    use). Only backtick tokens captured directly after the "Expected
    artifact:" label are taken as filenames — a description's own
    cross-references (e.g. "per `../CONTEXT.md`") are never mistaken for a
    declared artifact, because the regex stops at the first non-backtick,
    non-separator character (typically " — ").

    A README with more than one `**Disposition:**` line and no table is
    deliberately left unresolved (empty `declared`) rather than guessed at
    — multi-artifact, multi-disposition stages must use a table.
    """
    table = {}
    in_table = False
    for raw in text.splitlines():
        line = raw.rstrip("\n")
        row = TABLE_ROW_RE.match(line)
        if row:
            fname, disp = row.group(1).strip(), row.group(2).strip()
            if fname.lower() not in ("file", "---", ""):
                table[fname] = disp
            in_table = True
            continue
        if in_table and line.strip() == "":
            in_table = False

    if table:
        return table, {}

    names = []
    for m in ARTIFACT_NAMES_RE.finditer(text):
        names.extend(NAME_TOKEN_RE.findall(m.group(1)))

    dispositions = DISPOSITION_LINE_RE.findall(text)

    declared = {}
    if names and len(dispositions) == 1:
        for n in names:
            declared[n] = dispositions[0]
    return None, declared


def plan_stage(stage_dir, stage_id):
    output_dir = os.path.join(stage_dir, "output")
    plan = StagePlan(stage_id)
    if not os.path.isdir(output_dir):
        return plan  # no output/ declared at all: nothing to finalize

    readme_path = os.path.join(output_dir, "README.md")
    entries = sorted(
        f for f in os.listdir(output_dir)
        if f != "README.md" and os.path.isfile(os.path.join(output_dir, f))
    )
    if not entries:
        return plan  # authored-tree shape, or already finalized

    if not os.path.isfile(readme_path):
        plan.error = f"{stage_id}: output/ has {len(entries)} file(s) but no README.md declares a disposition"
        return plan

    with open(readme_path, encoding="utf-8") as f:
        text = f.read()
    table, declared = parse_readme(text)

    if table is None and not declared:
        n_disp_lines = len(DISPOSITION_LINE_RE.findall(text))
        if n_disp_lines > 1:
            plan.error = f"{stage_id}: output/README.md has {n_disp_lines} `**Disposition:**` lines and no table — cannot bind each to an artifact unambiguously"
        else:
            plan.error = f"{stage_id}: output/README.md declares no `**Disposition:**` line and no disposition table"
        return plan

    for fname in entries:
        if table is not None:
            disp = table.get(fname)  # not in table -> undeclared, whatever else the table names
        else:
            disp = declared.get(fname)  # not among named Expected-artifact filenames -> undeclared

        if disp is None:
            plan.remove.append(fname)  # undeclared: silence promotes nothing
            continue
        if disp not in KNOWN_DISPOSITIONS:
            plan.error = f"{stage_id}: output/{fname} declares unknown disposition `{disp}` (expected `promote` or `evidence`)"
            return plan
        if disp == "promote":
            plan.keep.append(fname)
        else:
            plan.remove.append(fname)

    return plan


def find_stage_dirs(root):
    return sorted(
        d for d in os.listdir(root)
        if STAGE_DIR_RE.match(d) and os.path.isdir(os.path.join(root, d))
    )


def git(*args, cwd):
    return subprocess.run(
        ["git", *args], cwd=cwd, capture_output=True, text=True, check=False
    )


def main():
    if len(sys.argv) > 2:
        print("usage: finalize.py [WORKFLOW_ROOT]", file=sys.stderr)
        return 2

    if len(sys.argv) == 2:
        root = os.path.abspath(sys.argv[1])
    else:
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    if not os.path.isfile(os.path.join(root, "workflow.toml")):
        print(f"error: {root} has no workflow.toml — not a workflow root", file=sys.stderr)
        return 2

    stage_dirs = find_stage_dirs(root)
    plans = [plan_stage(os.path.join(root, s), s) for s in stage_dirs]

    errors = [p.error for p in plans if p.error]
    if errors:
        print("REFUSED (fail-closed: nothing modified) — ambiguous disposition:")
        for e in errors:
            print(" -", e)
        return 1

    to_remove = []
    for p in plans:
        for fname in p.keep:
            print(f"keep    {p.stage_id}/output/{fname}  (promote)")
        for fname in p.remove:
            print(f"remove  {p.stage_id}/output/{fname}")
            to_remove.append(os.path.join(p.stage_id, "output", fname))

    if not to_remove:
        print("\nnothing to finalize (no evidence-class or undeclared files present)")
        return 0

    in_git = git("rev-parse", "--is-inside-work-tree", cwd=root)
    use_git = in_git.returncode == 0 and in_git.stdout.strip() == "true"

    if use_git:
        rm = git("rm", "-f", "--", *to_remove, cwd=root)
        if rm.returncode != 0:
            print(f"error: git rm failed:\n{rm.stderr}", file=sys.stderr)
            return 2
        commit = git(
            "commit", "-m",
            "repo-to-icm finalize: apply output/ dispositions (D9)\n\n"
            "Removed evidence-class and undeclared per-run artifacts per each "
            "stage's output/README.md. Kept files remain removable from "
            "Work-branch history.",
            cwd=root,
        )
        if commit.returncode != 0:
            print(f"error: git commit failed:\n{commit.stderr}", file=sys.stderr)
            return 2
        print(f"\nfinalized: removed {len(to_remove)} file(s), committed.")
    else:
        for rel in to_remove:
            os.remove(os.path.join(root, rel))
        print(f"\nfinalized: removed {len(to_remove)} file(s) (no git work tree found — removed directly, not committed).")

    return 0


if __name__ == "__main__":
    sys.exit(main())
