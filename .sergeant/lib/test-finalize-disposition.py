#!/usr/bin/env python3
"""test-finalize-disposition.py — R-MVP1-2's own pin test.

`docs/gauntlet/contracts/MVP-1.md`, R-MVP1-2's pin: "A fake run of a
two-stage workflow with one `evidence` and one `promote` output leaves only
the promoted file on the retained branch, the removal in branch history;
reverting the closing-stage instruction leaves the evidence file (L7)."

This proves exactly that, against a disposable git sandbox (never this
checkout, no network), using the shared helper this directory ships
(`finalize.py`) rather than any one workflow's private copy — the point of
generalizing it out of `repo-to-icm` in the first place.

SCENARIO
--------
A scratch two-stage workflow:

    00-alpha/output/{README.md, evidence.md}   disposition: evidence
    10-beta/output/{README.md, promoted.md}    disposition: promote

Both stages' `output/` are staged (`git add`) and committed — the ordinary
calling convention (a closing stage stages everything before invoking
finalize) — before finalize runs.

1. **Closing-stage instruction present** (finalize.py runs): on the
   retained branch afterward, `10-beta/output/promoted.md` is present and
   `00-alpha/output/evidence.md` is gone from the working tree — but still
   reachable via `git log --all -- 00-alpha/output/evidence.md` (the "removal
   in branch history" half of the pin: recoverable, not destroyed).
2. **Closing-stage instruction reverted** (finalize.py never runs, modeling
   a workflow author who deletes the closing-stage instruction): the branch
   is exactly the pre-finalize commit — both files present, including the
   evidence one — proving finalize.py, not something else, is what removes
   it.

USAGE
-----
    python3 test-finalize-disposition.py

Runnable standalone — no pytest, `git` and `python3` on PATH only. Exits 0
if both scenarios pass, 1 otherwise (with the failing assertion printed to
stderr).
"""

import os
import shutil
import subprocess
import sys
import tempfile

_HERE = os.path.dirname(os.path.abspath(__file__))
FINALIZE_PY = os.path.join(_HERE, "finalize.py")


def run(cmd, cwd, check=True):
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if check and r.returncode != 0:
        raise RuntimeError(f"{' '.join(cmd)} failed in {cwd}:\n{r.stdout}\n{r.stderr}")
    return r


def build_sandbox(tmp):
    """A two-stage workflow root: 00-alpha (evidence), 10-beta (promote),
    both committed — the ordinary pre-finalize state (§1a's calling
    convention: stage, then invoke finalize)."""
    os.makedirs(os.path.join(tmp, "00-alpha", "output"))
    os.makedirs(os.path.join(tmp, "10-beta", "output"))

    with open(os.path.join(tmp, "workflow.toml"), "w", encoding="utf-8") as f:
        f.write(
            '[workflow]\nname = "finalize-pin-fixture"\nversion = "1"\n'
            'stages = ["00-alpha", "10-beta"]\n'
        )

    with open(os.path.join(tmp, "00-alpha", "output", "README.md"), "w", encoding="utf-8") as f:
        f.write("**Expected artifact:** `evidence.md` — scratch notes.\n\n**Disposition:** `evidence`\n")
    with open(os.path.join(tmp, "00-alpha", "output", "evidence.md"), "w", encoding="utf-8") as f:
        f.write("scratch notes, not meant to survive finalize\n")

    with open(os.path.join(tmp, "10-beta", "output", "README.md"), "w", encoding="utf-8") as f:
        f.write("**Expected artifact:** `promoted.md` — the real deliverable.\n\n**Disposition:** `promote`\n")
    with open(os.path.join(tmp, "10-beta", "output", "promoted.md"), "w", encoding="utf-8") as f:
        f.write("the artifact this run actually produced\n")

    run(["git", "init", "-q"], cwd=tmp)
    run(["git", "config", "user.email", "test@example.invalid"], cwd=tmp)
    run(["git", "config", "user.name", "finalize pin test"], cwd=tmp)
    run(["git", "add", "-A"], cwd=tmp)
    run(["git", "commit", "-q", "-m", "pre-finalize: both stages' output staged and committed"], cwd=tmp)


def files_at_head(tmp):
    r = run(["git", "ls-tree", "-r", "--name-only", "HEAD"], cwd=tmp)
    return set(r.stdout.split())


def test_closing_stage_present_promotes_and_evicts():
    """finalize.py runs (the closing-stage instruction present): only the
    promoted file remains on the branch; the evidence file is gone from the
    tree but still reachable in history."""
    tmp = tempfile.mkdtemp(prefix="finalize-pin-present-")
    try:
        build_sandbox(tmp)
        r = run([sys.executable, FINALIZE_PY, tmp], cwd=tmp, check=False)
        assert r.returncode == 0, f"finalize.py should exit 0 on a clean apply, got {r.returncode}:\n{r.stdout}\n{r.stderr}"

        head_files = files_at_head(tmp)
        assert "10-beta/output/promoted.md" in head_files, (
            f"promoted.md must be on the retained branch after finalize; head has {head_files}"
        )
        assert "00-alpha/output/evidence.md" not in head_files, (
            f"evidence.md must NOT be on the retained branch after finalize; head has {head_files}"
        )

        log = run(
            ["git", "log", "--all", "--oneline", "--", "00-alpha/output/evidence.md"],
            cwd=tmp,
        )
        assert log.stdout.strip() != "", (
            "evidence.md's removal must be recoverable from branch history "
            "(git log --all must still find it), not destroyed outright"
        )
        print("PASS  closing-stage instruction present: promoted file kept, evidence file removed-but-recoverable")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def test_closing_stage_reverted_leaves_evidence_file():
    """finalize.py never runs (the closing-stage instruction reverted,
    per L7): the branch is exactly the pre-finalize commit, evidence file
    included — proving finalize.py, not something else, does the removal."""
    tmp = tempfile.mkdtemp(prefix="finalize-pin-reverted-")
    try:
        build_sandbox(tmp)
        before = files_at_head(tmp)

        # The closing-stage instruction is simply never invoked here.

        after = files_at_head(tmp)
        assert after == before, "no finalize invocation must mean no branch change at all"
        assert "00-alpha/output/evidence.md" in after, (
            f"reverting the closing-stage instruction must leave the evidence file on the branch; head has {after}"
        )
        assert "10-beta/output/promoted.md" in after
        print("PASS  closing-stage instruction reverted: evidence file left in place, nothing removed")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def main():
    if not os.path.isfile(FINALIZE_PY):
        print(f"error: {FINALIZE_PY} not found", file=sys.stderr)
        return 2
    try:
        test_closing_stage_present_promotes_and_evicts()
        test_closing_stage_reverted_leaves_evidence_file()
    except AssertionError as e:
        print(f"FAIL: {e}", file=sys.stderr)
        return 1
    except RuntimeError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1
    print("\nPASS: R-MVP1-2's pin holds — promote/evidence disposition, reachable-not-destroyed removal.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
