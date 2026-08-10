#!/usr/bin/env python3
"""test-finalize-evidence-guard.py — proves finalize.py's evidence-preservation
guard (GP-5b; docs/gauntlet/runs/n2-run2/grammar-pressure-report.md GP-5b;
issue #29) against a disposable git sandbox.

WHAT THIS PROVES
-----------------------------------------------------------------------------
Three scenarios, each against its own scratch git repository under a temp
directory (never this checkout, never touches `.sergeant/` here, no
network):

1. **Refuse case.** A stage's `output/` holds an UNDECLARED file that was
   never `git add`ed at all — exactly the shape that produced the original
   GP-5b defect (`20-harvest/output/coverage-note.md` deleted from an
   uncommitted working tree). `finalize.py` run for real against this
   sandbox MUST:
     - exit 1 (REFUSED),
     - print a message naming the file and the word `EVIDENCE-LOSS`,
     - leave the file on disk, untouched,
     - create no new commit (the sandbox's commit count is unchanged).
   Regressing to the pre-guard behavior (silently `git rm`+`git commit`ing
   an untracked file — which real git actually refuses on its own with a
   "did not match any files" error, but only when every removed path in
   the same `git rm` invocation is untracked; mixed with tracked paths in
   the same call it is not so obviously caught) fails this scenario.

2. **Proceed-via-capture-commit case.** The same undeclared file, but
   `git add`ed (staged) — not yet committed — before finalize runs, exactly
   `90-reconcile`'s own ordinary calling convention (stage everything, then
   invoke finalize). `finalize.py` run for real MUST:
     - exit 0,
     - actually remove the file from disk,
     - leave it reachable via `git log --all -- <path>` afterward,
     - and get there via an extra **capture commit** — the sandbox's commit
       count increases by 2 (capture, then the removal commit), not 1 —
       proving finalize actually committed the staged content before
       removing it rather than silently reproducing the original bug.
   This is the common case in production and the one most important not to
   regress: a guard that refuses on this shape would make ordinary runs
   unable to finalize at all.

3. **Proceed-already-committed case.** The same file, committed outright
   before finalize runs (already reachable, no capture needed).
   `finalize.py` run for real MUST exit 0, remove it, leave it reachable,
   and — since no capture commit was needed — increase the commit count by
   only 1. This proves the guard does not force an unnecessary extra commit
   when the precondition it checks already holds.

USAGE
-----
    python3 test-finalize-evidence-guard.py

Runnable standalone — no pytest, no repo state required beyond a `git` and
`python3` on PATH. Exits 0 if all three scenarios pass, 1 otherwise (with
the specific assertion that failed printed to stderr). Wired into
`validate-structure.py` as check `[S15]` (admitted-mode only) once this
script itself passes cleanly.
"""

import os
import shutil
import subprocess
import sys
import tempfile

FINALIZE_PY = os.path.join(os.path.dirname(os.path.abspath(__file__)), "finalize.py")

README_TEMPLATE = """# Output — `00-stage` (sandbox)

**Expected artifact:** `keep.md`

**Disposition:** `promote`
"""


def run(cmd, cwd, check=True):
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if check and r.returncode != 0:
        raise RuntimeError(f"command failed: {cmd!r}\nstdout:\n{r.stdout}\nstderr:\n{r.stderr}")
    return r


def git(args, cwd, check=True):
    return run(["git", *args], cwd=cwd, check=check)


def commit_count(cwd):
    r = git(["rev-list", "--all", "--count"], cwd=cwd, check=False)
    if r.returncode != 0:
        return 0
    return int(r.stdout.strip() or "0")


def make_sandbox(root):
    """A one-stage sandbox workflow with a `promote` file and an
    UNDECLARED file (`evidence.md`) under `00-stage/output/`, mirroring the
    exact shape finalize.py scans: a stage dir, its output/README.md, and
    whatever files actually sit under output/ that the README does not
    account for."""
    git(["init", "-q"], cwd=root)
    git(["config", "user.email", "sandbox@example.invalid"], cwd=root)
    git(["config", "user.name", "sandbox"], cwd=root)

    with open(os.path.join(root, "workflow.toml"), "w") as f:
        f.write(
            '[workflow]\nname = "sandbox"\nversion = "1"\n'
            'stages = ["00-stage"]\n'
        )

    stage_output = os.path.join(root, "00-stage", "output")
    os.makedirs(stage_output)
    with open(os.path.join(stage_output, "README.md"), "w") as f:
        f.write(README_TEMPLATE)
    with open(os.path.join(stage_output, "keep.md"), "w") as f:
        f.write("promoted content\n")
    with open(os.path.join(stage_output, "evidence.md"), "w") as f:
        f.write("undeclared, uncommitted content — the GP-5b shape\n")


def scenario_refuse():
    """evidence.md is left uncommitted. finalize.py must REFUSE."""
    tmp = tempfile.mkdtemp(prefix="finalize-guard-refuse-")
    try:
        make_sandbox(tmp)
        # Commit everything EXCEPT evidence.md — the exact GP-5b shape: an
        # undeclared file materialized under output/ that was never staged.
        git(["add", "workflow.toml", "00-stage/output/README.md",
             "00-stage/output/keep.md"], cwd=tmp)
        git(["commit", "-q", "-m", "sandbox baseline (evidence.md deliberately uncommitted)"], cwd=tmp)

        before_commits = commit_count(tmp)
        evidence_path = os.path.join(tmp, "00-stage", "output", "evidence.md")
        assert os.path.isfile(evidence_path), "sandbox setup broken: evidence.md missing before run"

        r = run([sys.executable, FINALIZE_PY, tmp], cwd=tmp, check=False)

        if r.returncode != 1:
            raise AssertionError(
                f"scenario 1 (refuse): expected exit 1, got {r.returncode}\n"
                f"stdout:\n{r.stdout}\nstderr:\n{r.stderr}"
            )
        if "EVIDENCE-LOSS" not in r.stdout:
            raise AssertionError(
                f"scenario 1 (refuse): expected 'EVIDENCE-LOSS' in stdout, got:\n{r.stdout}"
            )
        if not os.path.isfile(evidence_path):
            raise AssertionError("scenario 1 (refuse): evidence.md was removed — guard did not hold")
        after_commits = commit_count(tmp)
        if after_commits != before_commits:
            raise AssertionError(
                f"scenario 1 (refuse): commit count changed ({before_commits} -> "
                f"{after_commits}) — finalize modified the repo despite refusing"
            )
        print("PASS  scenario 1 (refuse): uncommitted undeclared file -> finalize REFUSED, nothing modified")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def scenario_proceed_via_capture():
    """evidence.md is `git add`ed (staged) but NOT committed before finalize
    runs — 90-reconcile's own ordinary calling convention. finalize.py must
    create a capture commit, then proceed to remove it, leaving it
    reachable in history."""
    tmp = tempfile.mkdtemp(prefix="finalize-guard-capture-")
    try:
        make_sandbox(tmp)
        git(["add", "workflow.toml", "00-stage/output/README.md",
             "00-stage/output/keep.md"], cwd=tmp)
        git(["commit", "-q", "-m", "sandbox baseline"], cwd=tmp)
        # Stage evidence.md now (mirroring 90-reconcile's own `git add
        # .../output/` step) but do NOT commit it — the ordinary
        # staged-but-uncommitted shape finalize.py must handle by capturing
        # it first, not by refusing.
        git(["add", "00-stage/output/evidence.md"], cwd=tmp)

        before_commits = commit_count(tmp)
        evidence_path = os.path.join(tmp, "00-stage", "output", "evidence.md")
        evidence_rel = os.path.join("00-stage", "output", "evidence.md")

        r = run([sys.executable, FINALIZE_PY, tmp], cwd=tmp, check=False)

        if r.returncode != 0:
            raise AssertionError(
                f"scenario 2 (proceed via capture): expected exit 0, got {r.returncode}\n"
                f"stdout:\n{r.stdout}\nstderr:\n{r.stderr}"
            )
        if os.path.isfile(evidence_path):
            raise AssertionError("scenario 2 (proceed via capture): evidence.md was NOT removed")
        log = git(["log", "--all", "--oneline", "--", evidence_rel], cwd=tmp)
        if not log.stdout.strip():
            raise AssertionError(
                "scenario 2 (proceed via capture): evidence.md is gone but "
                "unreachable via `git log --all` — the capture commit did not "
                "actually happen"
            )
        after_commits = commit_count(tmp)
        if after_commits != before_commits + 2:
            raise AssertionError(
                f"scenario 2 (proceed via capture): expected exactly 2 new "
                f"commits (capture + removal), got {after_commits - before_commits} "
                f"({before_commits} -> {after_commits})"
            )
        if "capture" not in r.stdout.lower():
            raise AssertionError(
                f"scenario 2 (proceed via capture): expected finalize's own "
                f"output to mention the capture commit, got:\n{r.stdout}"
            )
        print("PASS  scenario 2 (proceed via capture): staged-but-uncommitted file -> "
              "finalize captured it in a commit, then removed it, still recoverable")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def scenario_proceed_already_committed():
    """evidence.md IS committed outright before finalize runs. finalize.py
    must proceed normally with no capture commit needed, and actually
    remove it, leaving it reachable in history."""
    tmp = tempfile.mkdtemp(prefix="finalize-guard-proceed-")
    try:
        make_sandbox(tmp)
        # Commit everything, including evidence.md this time — the
        # precondition the guard checks for is already satisfied outright.
        git(["add", "-A"], cwd=tmp)
        git(["commit", "-q", "-m", "sandbox baseline (evidence.md committed)"], cwd=tmp)

        before_commits = commit_count(tmp)
        evidence_path = os.path.join(tmp, "00-stage", "output", "evidence.md")
        evidence_rel = os.path.join("00-stage", "output", "evidence.md")

        r = run([sys.executable, FINALIZE_PY, tmp], cwd=tmp, check=False)

        if r.returncode != 0:
            raise AssertionError(
                f"scenario 3 (proceed, already committed): expected exit 0, got {r.returncode}\n"
                f"stdout:\n{r.stdout}\nstderr:\n{r.stderr}"
            )
        if os.path.isfile(evidence_path):
            raise AssertionError("scenario 3 (proceed, already committed): evidence.md was NOT removed")
        log = git(["log", "--all", "--oneline", "--", evidence_rel], cwd=tmp)
        if not log.stdout.strip():
            raise AssertionError(
                "scenario 3 (proceed, already committed): evidence.md is gone "
                "but unreachable via `git log --all`"
            )
        after_commits = commit_count(tmp)
        if after_commits != before_commits + 1:
            raise AssertionError(
                f"scenario 3 (proceed, already committed): expected exactly 1 "
                f"new commit (no capture commit needed), got "
                f"{after_commits - before_commits} ({before_commits} -> {after_commits})"
            )
        print("PASS  scenario 3 (proceed, already committed): committed undeclared file -> "
              "finalize removed it directly, no unnecessary capture commit, still recoverable")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def main():
    if not os.path.isfile(FINALIZE_PY):
        print(f"error: {FINALIZE_PY} not found", file=sys.stderr)
        return 2
    which_git = shutil.which("git")
    if not which_git:
        print("error: git not found on PATH", file=sys.stderr)
        return 2

    try:
        scenario_refuse()
        scenario_proceed_via_capture()
        scenario_proceed_already_committed()
    except AssertionError as e:
        print(f"FAIL  {e}", file=sys.stderr)
        return 1
    except RuntimeError as e:
        print(f"FAIL  sandbox setup error: {e}", file=sys.stderr)
        return 1

    print("\nPASS: finalize.py's evidence-preservation guard holds in all three cases.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
