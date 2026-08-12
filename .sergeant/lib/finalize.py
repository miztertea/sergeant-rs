#!/usr/bin/env python3
"""finalize.py — the shared D9 disposition finalize helper (R-MVP1-2).

Layer 3 (`_config`-adjacent) deterministic machinery. R-MVP1-2
(`docs/gauntlet/contracts/MVP-1.md`) ruled that promote/finalize EXECUTION
is workflow content, not an engine concern: "the workflow's closing actor
stage invokes a deterministic shared finalize helper; the engine learns no
output vocabulary." This is that helper — generalized out of
`repo-to-icm`'s own `scripts/finalize.py` (its first consumer, which now
delegates here — see that file) so any workflow whose stages declare
`output/` artifacts can invoke the identical, already-proven logic from its
own closing stage, by passing its own workflow root as the one argument.

WHAT THIS DOES AND DOES NOT DO
-------------------------------
At the end of a run, every stage of the *calling* workflow has its own
`output/` directory materialized in the run worktree, each declaring — in
that stage's `output/README.md` — the artifact(s) it produces and a
disposition (`docs/icm/convention.md` §1a):

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
runs*, not artifacts of the calling workflow's own run, and are never
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
    python3 finalize.py [--dry-run] WORKFLOW_ROOT

`WORKFLOW_ROOT` is the calling workflow's own root directory (the parent of
its stage directories) — always pass it explicitly; this shared copy has no
workflow of its own to default to (a workflow-local caller, such as
`repo-to-icm/scripts/finalize.py`, may still supply its own default before
delegating here).

`--dry-run` computes and prints the exact same keep/remove plan (including
any REFUSED ambiguity) but performs no filesystem or git mutation — no
`git rm`, no `git commit`, no `os.remove`. Its exit code matches what a real
run would return (0 clean-apply-or-nothing-to-do, 1 refused-ambiguous, 2
usage/environment error), so it is safe to sanity-check the disposition
plan against a real or scratch worktree before the closing stage actually
applies it.

Exits 0 on a clean apply (including "nothing to do"), 1 if any stage's
output could not be resolved unambiguously, or if the evidence-preservation
guard below refuses (nothing is modified in either case), 2 on a
usage/environment error.

EVIDENCE-PRESERVATION GUARD (GP-5b; docs/gauntlet/runs/n2-run2/
grammar-pressure-report.md GP-5b; issue #29)
-----------------------------------------------------------------------------
`docs/icm/convention.md` §1a's D9 disposition rule promises that
`evidence`-class and undeclared files "remain recoverable from Work-branch
history" after finalize removes them. That promise is only true if the file
was actually committed to some tree before this script's own `git rm` runs.
A `git add` immediately followed by a `git rm`, with no commit landing in
between, writes a blob object into `.git/objects` but no tree or commit ever
references it (measured: `git commit` after such an add+rm reports "nothing
to commit" for that path) — `git log --all -- <path>` finds nothing, the
object is unreachable, and it is eventually garbage-collected.

Before removing anything, this script verifies every file about to be
removed is already reachable in at least one committed tree (`git log
--all -- <path>` non-empty — equivalently, `git cat-file -e
<commit>:<path>` succeeds for some commit). The ordinary calling convention
is: a closing stage stages (`git add`) every stage's `output/` before
invoking this script, so the ordinary case is: a to-be-removed file is
*staged* but not yet committed. For exactly that case, this script performs
a **capture commit** first — committing whatever is currently staged,
verbatim, before touching anything — which makes every staged file
(including the ones about to be removed) genuinely reachable, and then
proceeds with the ordinary removal-and-commit sequence below. This is
mechanical, not judgment: it commits what the caller already staged, it
does not decide what *should* be staged (this script still never runs
`git add` itself).

If, after that capture commit (or with nothing to capture), a file slated
for removal is **still** not reachable — it was never staged at all —
this script REFUSES outright (fail-closed, nothing modified beyond the
capture commit already described, which by construction only ever adds
content, never removes any) rather than deleting unrecoverable evidence.
A calling workflow's own evidence-guard test (see, for the pattern,
`repo-to-icm/scripts/test-finalize-evidence-guard.py`, and R-MVP1-2's own
pin test, `test-finalize-disposition.py` in this directory) proves this
against a scratch git sandbox, runnable standalone.
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
    single-artifact-per-stage shape). Only backtick tokens captured directly
    after the "Expected artifact:" label are taken as filenames — a
    description's own cross-references (e.g. "per `../CONTEXT.md`") are
    never mistaken for a declared artifact, because the regex stops at the
    first non-backtick, non-separator character (typically " — ").

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


def _reachable(root, rel):
    """True iff `rel` is present in at least one commit reachable from any
    ref (`git log --all -- rel` non-empty) — meaning its content would
    survive even after this script's own `git rm` (GP-5b: the D9
    disposition policy's "recoverable from Work-branch history" promise,
    checked rather than assumed)."""
    log = git("log", "--all", "--oneline", "--", rel, cwd=root)
    return log.returncode == 0 and log.stdout.strip() != ""


def _is_staged(root, rel):
    """True iff `rel` is currently in the git index (staged via `git add`),
    whether or not it has ever been committed — `git ls-files --stage --
    rel` non-empty. Distinguishes "about to be captured by the next commit"
    from "genuinely never touched," which `_reachable` alone cannot tell
    apart (both read as unreachable in `git log`)."""
    ls = git("ls-files", "--stage", "--", rel, cwd=root)
    return ls.returncode == 0 and ls.stdout.strip() != ""


def main():
    args = sys.argv[1:]
    dry_run = "--dry-run" in args
    positional = [a for a in args if a != "--dry-run"]

    if len(positional) != 1:
        print("usage: finalize.py [--dry-run] WORKFLOW_ROOT", file=sys.stderr)
        return 2

    root = os.path.abspath(positional[0])

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

    plan_lines = []   # ("keep", printable) | ("remove", relpath) in stage order
    to_remove = []
    for p in plans:
        for fname in p.keep:
            plan_lines.append(("keep", f"keep    {p.stage_id}/output/{fname}  (promote)"))
        for fname in p.remove:
            rel = os.path.join(p.stage_id, "output", fname)
            plan_lines.append(("remove", rel))
            to_remove.append(rel)

    in_git = git("rev-parse", "--is-inside-work-tree", cwd=root)
    use_git = in_git.returncode == 0 and in_git.stdout.strip() == "true"

    # Evidence-preservation guard (GP-5b) — see module docstring. Checked
    # before anything is printed or modified. Two cases for a to-be-removed
    # file that is not yet reachable in a committed tree:
    #   - never staged at all (the literal GP-5b shape) -> REFUSE outright,
    #     nothing modified.
    #   - staged but not yet committed (the ordinary case, since the calling
    #     stage's own `git add` runs before this script) -> commit whatever
    #     is currently staged first (a "capture commit" — this script still
    #     never runs `git add` itself, it only commits what the caller
    #     already staged), which makes it reachable, then proceed.
    # Only meaningful inside a git worktree; outside one there is no
    # "Work-branch history" to preserve in the first place.
    capture_note = None
    if use_git and to_remove:
        unreachable = [rel for rel in to_remove if not _reachable(root, rel)]
        if unreachable:
            staged = [rel for rel in unreachable if _is_staged(root, rel)]
            never_staged = [rel for rel in unreachable if rel not in staged]

            if never_staged:
                print("REFUSED (fail-closed: nothing modified) — evidence preservation:")
                for rel in never_staged:
                    print(
                        " -",
                        f"EVIDENCE-LOSS: {rel} is about to be removed but is "
                        f"not staged and not committed anywhere (`git log "
                        f"--all -- {rel}` and `git ls-files --stage -- {rel}` "
                        f"are both empty) — its removal would be "
                        f"unrecoverable, not 'recoverable from Work-branch "
                        f"history' as the D9 disposition policy promises "
                        f"(GP-5b). `git add -- {rel}` it first, then re-run "
                        f"finalize.",
                    )
                return 1

            # Every unreachable file IS staged: a capture commit makes it,
            # and everything else already staged, reachable before removal.
            if dry_run:
                capture_note = (
                    f"would first create a capture commit for {len(staged)} "
                    f"currently-staged file(s) not yet in any commit "
                    f"(evidence-preservation guard, GP-5b); nothing modified "
                    f"(dry-run)"
                )
            else:
                commit = git(
                    "commit", "-m",
                    "finalize: capture staged per-run artifact(s) before "
                    "disposition (evidence-preservation guard, GP-5b)\n\n"
                    "Committed whatever this run's stages had already staged, "
                    "before removing any evidence-class/undeclared file, so "
                    "every removal below is a recoverable deletion rather "
                    "than an unrecorded one.",
                    cwd=root,
                )
                if commit.returncode != 0:
                    print(f"error: capture commit failed:\n{commit.stderr}", file=sys.stderr)
                    return 2
                still_unreachable = [rel for rel in to_remove if not _reachable(root, rel)]
                if still_unreachable:
                    print("REFUSED — evidence preservation still fails after capture commit:")
                    for rel in still_unreachable:
                        print(" -", f"EVIDENCE-LOSS: {rel} still unreachable after the capture commit")
                    return 1
                capture_note = (
                    f"captured {len(staged)} staged file(s) in a commit before removal"
                )

    for kind, val in plan_lines:
        if kind == "keep":
            print(val)
        else:
            verb = "would remove" if dry_run else "remove"
            print(f"{verb}  {val}")

    if capture_note:
        print(f"\n{capture_note}")

    if not to_remove:
        print("\nnothing to finalize (no evidence-class or undeclared files present)")
        return 0

    if dry_run:
        print(f"\ndry-run: would finalize {len(to_remove)} file(s); nothing modified.")
        return 0

    if use_git:
        rm = git("rm", "-f", "--", *to_remove, cwd=root)
        if rm.returncode != 0:
            print(f"error: git rm failed:\n{rm.stderr}", file=sys.stderr)
            return 2
        commit = git(
            "commit", "-m",
            "finalize: apply output/ dispositions (D9)\n\n"
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
