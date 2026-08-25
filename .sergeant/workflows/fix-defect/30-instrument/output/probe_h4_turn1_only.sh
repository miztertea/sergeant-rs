#!/usr/bin/env bash
# [W5B-H4] Probe for 20-hypothesize's H4 (rule-out): is turn 1 itself
# under-granted, independent of resume at all?
#
# One variable changed vs. the baseline repro
# (00-build-feedback-loop/output/codex_resume_add_dir_repro.sh): the SAME
# turn 1 launch (first_turn_argv shape: -C $WORKTREE --sandbox
# workspace-write --add-dir $COMMON) is asked to edit AND git add/commit,
# in one turn — no resume at all. If this succeeds, H4 is falsified (turn
# 1's grant is correct; the failure is specific to resume, as H1-H3
# assume). If it fails identically, H4 is confirmed.
#
# Exit 0 = turn 1 alone can commit (H4 falsified). Exit 1 = turn 1 alone
# cannot commit either (H4 confirmed). Exit 2 = harness failure.
set -euo pipefail

SCRATCH="$(mktemp -d /tmp/codex-h4-probe.XXXXXX)"
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

echo "[W5B-H4] turn 1 only: -C \$WORKTREE --sandbox workspace-write --add-dir \$COMMON, edit+add+commit in one turn" >&2
TURN1_LOG="$SCRATCH/turn1.jsonl"
timeout 90 codex exec --json --skip-git-repo-check -C "$WORKTREE" \
  --sandbox workspace-write --add-dir "$COMMON" -m gpt-5.6-luna \
  "Append the line 'edit1' to README.md using a shell command, then run \`git add README.md\` then \`git commit -m repro\`. Report only the word done when finished, or the exact error text if it fails." \
  > "$TURN1_LOG" 2>&1
THREAD="$(grep -o '"thread_id":"[^"]*"' "$TURN1_LOG" | head -1 | cut -d'"' -f4)"
if [ -z "$THREAD" ]; then
  echo "[W5B-H4] FAIL(harness): turn 1 never announced thread.started — see $TURN1_LOG" >&2
  exit 2
fi

if git -C "$WORKTREE" log -1 --format=%s 2>/dev/null | grep -q '^repro$'; then
  echo "[W5B-H4] RESULT: turn 1 alone COMMITTED successfully (H4 falsified — turn 1's grant is correct)" >&2
  exit 0
else
  echo "[W5B-H4] RESULT: turn 1 alone did NOT commit (H4 confirmed). Transcript:" >&2
  cat "$TURN1_LOG" >&2
  echo "[W5B-H4] worktree status:" >&2
  git -C "$WORKTREE" status --short >&2
  exit 1
fi
