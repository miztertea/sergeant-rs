#!/usr/bin/env bash
# Red-capable, deterministic, fast repro for the W5b defect: a codex actor's
# SECOND turn (an `exec resume`, exactly what `resume_turn_argv` in
# src/backend/codex.rs composes for every stage after the first) never
# re-sends `--sandbox workspace-write --add-dir <common-dir>`. The code
# assumes codex-cli's session state persists the turn-1 sandbox grant across
# `exec resume` — that assumption is false, empirically, on codex-cli
# 0.149.0: the resumed turn's OS sandbox reverts to a narrower scope, and any
# git operation that touches the worktree's git-common-dir (loose objects,
# refs, and critically `<common>/worktrees/<name>/index.lock`) fails with
# exactly the symptom the real fresh-estate proof recorded:
#   fatal: Unable to create '.../.git/worktrees/<name>/index.lock': Read-only file system
#
# Mirrors the real production surface shape exactly: worktree under
# <estate>/data/surfaces/<work>/<repo>, git-common-dir under
# <estate>/repos/<repo>/.git — two genuinely separate top-level trees, NOT
# nested under one shared temp root the way the existing unit test
# (`real_worktree` in tests/codex_backend.rs) builds its fixture. That
# nesting is why the existing stub-driven --add-dir test and the existing
# live test (which also nests source/worktree under one `data_dir`) never
# caught this: they assert on turn-1's argv/behavior only, and never
# exercise a second (resumed) turn at all.
#
# Usage: bash /tmp/codex_resume_add_dir_repro.sh
# Requires: a working `codex` CLI, logged in, network access (spends a few
# cents of real tokens: two short turns, gpt-5.6-luna, one-word-bounded).
# Deterministic: fixed prompts, pinned model, fresh scratch dirs each run.
# Exit 0 = the commit landed (bug fixed / not present). Exit 1 = red — the
# resumed turn could not commit, matching the production symptom.
set -euo pipefail

SCRATCH="$(mktemp -d /tmp/codex-resume-repro.XXXXXX)"
trap 'rm -rf "$SCRATCH"' EXIT

SOURCE="$SCRATCH/estate/repos/solo"
WORKTREE="$SCRATCH/data/surfaces/W1/solo"
mkdir -p "$SOURCE" "$(dirname "$WORKTREE")"

git init -q -b main "$SOURCE"
git -C "$SOURCE" config user.email a@b.c
git -C "$SOURCE" config user.name test
echo hi > "$SOURCE/README.md"
git -C "$SOURCE" add README.md
git -C "$SOURCE" commit -q -m init
git -C "$SOURCE" worktree add -q -b sergeant/w1 "$WORKTREE"

COMMON="$SOURCE/.git"

echo "== turn 1 (first_turn_argv shape: -C \$WORKTREE --sandbox workspace-write --add-dir \$COMMON) ==" >&2
TURN1_LOG="$SCRATCH/turn1.jsonl"
timeout 90 codex exec --json --skip-git-repo-check -C "$WORKTREE" \
  --sandbox workspace-write --add-dir "$COMMON" -m gpt-5.6-luna \
  "Append the line 'edit1' to README.md using a shell command. Do not run git commands. Report only the word done." \
  > "$TURN1_LOG" 2>&1
THREAD="$(grep -o '"thread_id":"[^"]*"' "$TURN1_LOG" | head -1 | cut -d'"' -f4)"
if [ -z "$THREAD" ]; then
  echo "FAIL(harness): turn 1 never announced thread.started — see $TURN1_LOG" >&2
  exit 2
fi

echo "== turn 2 (resume_turn_argv shape: exec resume \$THREAD, NO -C, NO --sandbox, NO --add-dir) ==" >&2
TURN2_LOG="$SCRATCH/turn2.jsonl"
( cd "$WORKTREE" && timeout 90 codex exec resume "$THREAD" --json --skip-git-repo-check -m gpt-5.6-luna \
    "Run \`git add README.md\` then \`git commit -m repro\`. Report only the word done when finished, or the exact error text if it fails." \
    > "$TURN2_LOG" 2>&1 )

AFTER="$(git -C "$WORKTREE" rev-parse HEAD)"
BEFORE_COUNT="$(git -C "$SOURCE" log --oneline sergeant/w1 | wc -l)"

if git -C "$WORKTREE" log -1 --format=%s | grep -q '^repro$'; then
  echo "PASS: resumed turn committed successfully (HEAD=$AFTER)" >&2
  exit 0
else
  echo "RED: resumed turn did not commit. Turn 2 transcript:" >&2
  cat "$TURN2_LOG" >&2
  echo >&2
  echo "worktree status:" >&2
  git -C "$WORKTREE" status --short >&2
  exit 1
fi
