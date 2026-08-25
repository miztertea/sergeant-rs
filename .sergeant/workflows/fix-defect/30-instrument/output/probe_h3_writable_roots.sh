#!/usr/bin/env bash
# [W5B-H3] Probe for 20-hypothesize's H3: does resume's `-c` override
# mechanism reach the sandbox layer the way turn 1's `--add-dir` does?
#
# One variable changed vs. the baseline repro
# (00-build-feedback-loop/output/codex_resume_add_dir_repro.sh): turn 2
# (the resumed turn) additionally carries
#   -c sandbox_workspace_write.writable_roots=["<common-dir>"]
# — the `writable_roots` field name is not guessed; it was confirmed
# present in the installed codex-cli 0.149.1 binary's own struct field
# strings (`strings ~/.local/bin/codex | grep writable_roots`), adjacent
# to `network_access`/`exclude_tmpdir_env_var`/`exclude_slash_tmp` under
# `sandbox_workspace_write`, matching the config table name `-c` already
# uses for #262's `network_access` override at codex.rs:1112.
#
# If turn 2 now commits, H3 is confirmed (narrowest fix: one function,
# resume_turn_argv). If codex-cli rejects/ignores the key or the commit
# still fails identically, H3 is falsified.
#
# Exit 0 = resumed turn committed (H3 confirmed). Exit 1 = still red (H3
# falsified). Exit 2 = harness failure.
set -euo pipefail

SCRATCH="$(mktemp -d /tmp/codex-h3-probe.XXXXXX)"
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

echo "[W5B-H3] turn 1 (first_turn_argv shape, unchanged) ==" >&2
TURN1_LOG="$SCRATCH/turn1.jsonl"
timeout 90 codex exec --json --skip-git-repo-check -C "$WORKTREE" \
  --sandbox workspace-write --add-dir "$COMMON" -m gpt-5.6-luna \
  "Append the line 'edit1' to README.md using a shell command. Do not run git commands. Report only the word done." \
  > "$TURN1_LOG" 2>&1
THREAD="$(grep -o '"thread_id":"[^"]*"' "$TURN1_LOG" | head -1 | cut -d'"' -f4)"
if [ -z "$THREAD" ]; then
  echo "[W5B-H3] FAIL(harness): turn 1 never announced thread.started — see $TURN1_LOG" >&2
  exit 2
fi

echo "[W5B-H3] turn 2 (resume_turn_argv shape PLUS -c sandbox_workspace_write.writable_roots=[\"$COMMON\"]) ==" >&2
TURN2_LOG="$SCRATCH/turn2.jsonl"
( cd "$WORKTREE" && timeout 90 codex exec resume "$THREAD" --json --skip-git-repo-check \
    -c "sandbox_workspace_write.writable_roots=[\"$COMMON\"]" \
    -m gpt-5.6-luna \
    "Run \`git add README.md\` then \`git commit -m repro\`. Report only the word done when finished, or the exact error text if it fails." \
    > "$TURN2_LOG" 2>&1 )

if git -C "$WORKTREE" log -1 --format=%s 2>/dev/null | grep -q '^repro$'; then
  echo "[W5B-H3] RESULT: resumed turn COMMITTED successfully with the -c override (H3 confirmed)" >&2
  exit 0
else
  echo "[W5B-H3] RESULT: resumed turn did NOT commit even with the -c override (H3 falsified). Transcript:" >&2
  cat "$TURN2_LOG" >&2
  echo "[W5B-H3] worktree status:" >&2
  git -C "$WORKTREE" status --short >&2
  exit 1
fi
