#!/usr/bin/env bash
# scripts/coverage/common.sh — shared machinery for the S1 coverage harness.
#
# Sourced by every stage script (c0…c4, f1, f2). Nothing here measures
# coverage; it supplies the five things every stage owes the program:
#
#   * a disk pre-flight that refuses to start under 10 GB free (R-S0-6);
#   * a recorded toolchain fingerprint, and a refusal to continue when the
#     recorded one and the live one disagree (R-S0-2 — profdata from two
#     rustc versions must never be merged into one number);
#   * profraw accounting: produced / merged / discarded per stage, with each
#     unmergeable file named rather than silently dropped (§6.3);
#   * the post-stage hygiene sweep — no `sgt` daemon may outlive a stage;
#   * one artifacts directory, one log per stage, small enough to commit.
#
# The stage scripts, not this file, hold the measurement command lines: the
# convention (R-S0-3) is only trustworthy if a reader can see the exact
# `cargo llvm-cov …` invocation in the script that ran it, un-assembled.
#
# Knobs (all optional):
#   COV_ARTIFACTS   where logs and counts land   (default docs/coverage/artifacts-2026-08-10)
#   COV_MIN_FREE_GB disk floor, in GB            (default 10)
#   COV_ALLOW_DRIFT=1  record a toolchain change instead of refusing — for a
#                      deliberate re-baseline only; profdata is NOT reused.

set -euo pipefail

COV_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COV_ARTIFACTS="${COV_ARTIFACTS:-$COV_ROOT/docs/coverage/artifacts-2026-08-10}"
COV_MIN_FREE_GB="${COV_MIN_FREE_GB:-10}"

# cargo-llvm-cov's own collection tree. Measured on 0.8.7, not assumed: a
# managed run (`cargo llvm-cov …`, as opposed to the `show-env` flow) builds
# into and writes its profraws under target/llvm-cov-target/, which keeps the
# instrumented objects out of the plain `target/` the repo's gates use. Two
# build trees total, which is the ceiling R-S0-6 sets.
COV_TARGET_DIR="$COV_ROOT/target/llvm-cov-target"

# The rustup component's tools, never the system LLVM: system LLVM 18 cannot
# read a profraw written by rustc 1.94's LLVM 21.
COV_LLVM_BIN="$(dirname "$(rustc --print target-libdir)")/bin"
COV_PROFDATA="$COV_LLVM_BIN/llvm-profdata"

COV_STAGE=""
COV_LOG=""
COV_BEFORE=""

cov_say() { printf '   %s\n' "$*" | tee -a "${COV_LOG:-/dev/null}"; }
cov_head() { printf '\n\033[1m== %s\033[0m\n' "$*" | tee -a "${COV_LOG:-/dev/null}"; }
cov_fail() {
  printf '\n\033[31mcoverage stage %s failed: %s\033[0m\n' "${COV_STAGE:-?}" "$*" \
    | tee -a "${COV_LOG:-/dev/null}" >&2
  exit 1
}

# --- pre-flight ------------------------------------------------------------

# Refuse to start a stage that could run the disk out mid-collection: a
# truncated profraw is worse than a stage that never ran, because it takes
# the whole report down with it (measured: one unmergeable profraw makes
# `cargo llvm-cov report` exit 1 with "no profile can be merged").
cov_preflight_disk() {
  local free
  free="$(df -BG --output=avail "$COV_ROOT" | tail -1 | tr -dc '0-9')"
  cov_say "disk: ${free} GB free (floor ${COV_MIN_FREE_GB} GB)"
  if [ "${free:-0}" -lt "$COV_MIN_FREE_GB" ]; then
    cov_fail "only ${free} GB free at $COV_ROOT; the floor is ${COV_MIN_FREE_GB} GB (R-S0-6)"
  fi
}

# Record rustc / cargo / cargo-llvm-cov, and refuse to continue if a previous
# stage in this artifacts dir recorded a different set. `rust-toolchain.toml`
# floats on stable by design, so this is the only thing standing between a
# mid-run toolchain update and a number merged out of two compilers.
cov_record_versions() {
  local now="$COV_ARTIFACTS/toolchain.txt" live
  live="$(
    printf '## rustc -vV\n%s\n\n## cargo --version\n%s\n\n## cargo llvm-cov --version\n%s\n' \
      "$(rustc -vV)" "$(cargo --version)" "$(cargo llvm-cov --version)"
  )"
  if [ -f "$now" ]; then
    if ! printf '%s\n' "$live" | diff -q - "$now" >/dev/null; then
      printf '%s\n' "$live" > "$COV_ARTIFACTS/toolchain-drift-$(date -u +%Y%m%dT%H%M%SZ).txt"
      if [ "${COV_ALLOW_DRIFT:-0}" != "1" ]; then
        cov_fail "the toolchain changed since this artifacts dir was started (see \
toolchain-drift-*.txt). Profdata from two toolchains must not be merged (R-S0-2): start a \
fresh artifacts dir, or set COV_ALLOW_DRIFT=1 after cleaning the collection tree."
      fi
      cov_say "toolchain drift accepted by COV_ALLOW_DRIFT=1 — this run is a re-baseline"
    fi
  else
    printf '%s\n' "$live" > "$now"
  fi
  cov_say "toolchain: $(rustc -vV | sed -n 's/^release: //p') / $(cargo llvm-cov --version)"
}

# --- profraw accounting ----------------------------------------------------

cov_profraw_list() {
  # Before the first instrumented build the collection tree does not exist
  # yet, and `find` on a missing directory is a nonzero exit that `set -e`
  # would turn into a stage failure. An empty list is the right answer.
  [ -d "$COV_TARGET_DIR" ] || return 0
  find "$COV_TARGET_DIR" -maxdepth 1 -name '*.profraw' -printf '%f\n' 2>/dev/null | sort
}

# Is one profraw readable by the toolchain's own llvm-profdata? This is the
# same merge the report stage will do, run one file at a time so a corrupt
# file is named instead of taking the whole report down anonymously.
cov_profraw_mergeable() {
  "$COV_PROFDATA" merge -sparse -o /dev/null "$1" >/dev/null 2>&1
}

# --- stage lifecycle -------------------------------------------------------

# cov_stage_begin <stage-name>
cov_stage_begin() {
  COV_STAGE="$1"
  mkdir -p "$COV_ARTIFACTS"
  COV_LOG="$COV_ARTIFACTS/$COV_STAGE.log"
  : > "$COV_LOG"
  cov_head "stage $COV_STAGE — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  cov_preflight_disk
  cov_record_versions
  COV_BEFORE="$(cov_profraw_list)"
  cov_say "profraw before: $(printf '%s' "$COV_BEFORE" | grep -c . || true)"
  COV_STAGE_T0="$(date +%s)"
}

# cov_stage_end <min-produced> <why>
#
# Closes the accounting: how many profraws this stage produced, how many of
# them the toolchain can merge, how many it cannot (each named). A stage that
# produced fewer than <min-produced> is a failed measurement, not a fast one —
# the usual cause is that the binary under test never flushed.
cov_stage_end() {
  local min="${1:-1}" why="${2:-the stage must have produced coverage data}"
  local after produced count merged=0 discarded=0 name
  after="$(cov_profraw_list)"
  produced="$(comm -13 <(printf '%s\n' "$COV_BEFORE") <(printf '%s\n' "$after") | grep . || true)"
  count="$(printf '%s' "$produced" | grep -c . || true)"

  # The named-quarantine file exists only when there is something to name; an
  # empty one in the artifacts dir would read as "checked and clean" even for
  # a stage that never got as far as checking.
  rm -f "$COV_ARTIFACTS/$COV_STAGE-profraw-discarded.txt"
  while IFS= read -r name; do
    [ -n "$name" ] || continue
    if cov_profraw_mergeable "$COV_TARGET_DIR/$name"; then
      merged=$((merged + 1))
    else
      discarded=$((discarded + 1))
      printf '%s\n' "$name" >> "$COV_ARTIFACTS/$COV_STAGE-profraw-discarded.txt"
    fi
  done <<< "$produced"

  {
    printf 'stage\t%s\n' "$COV_STAGE"
    printf 'wall_s\t%s\n' "$(( $(date +%s) - COV_STAGE_T0 ))"
    printf 'profraw_before\t%s\n' "$(printf '%s' "$COV_BEFORE" | grep -c . || true)"
    printf 'profraw_produced\t%s\n' "$count"
    printf 'profraw_mergeable\t%s\n' "$merged"
    printf 'profraw_discarded\t%s\n' "$discarded"
    printf 'profraw_total\t%s\n' "$(printf '%s' "$after" | grep -c . || true)"
  } > "$COV_ARTIFACTS/$COV_STAGE-accounting.tsv"
  cov_say "profraw produced=$count mergeable=$merged discarded=$discarded"

  cov_hygiene

  if [ "$count" -lt "$min" ]; then
    cov_fail "produced $count profraw file(s), expected at least $min — $why"
  fi
  if [ "$discarded" -gt 0 ]; then
    cov_fail "$discarded profraw file(s) cannot be merged (named in \
$COV_STAGE-profraw-discarded.txt). One of these is enough to make the report stage exit 1 with \
'no profile can be merged' — quarantine them deliberately and record the loss, never leave them \
in the tree (§6.3)."
  fi
  cov_say "stage $COV_STAGE ok"
}

# The sweep, run after every stage. Quoting matters: an unquoted pattern
# matches the shell that typed it. Inside a script file it cannot — pgrep
# skips itself and the parent's argv is the script's path — but the same line
# pasted into `bash -c` will find itself, which is how this check has been
# fooled before.
# Note that the second pattern is a *superset* of the first — the
# instrumented binary lives at `…/llvm-cov-target/debug/sgt`, which contains
# `target/debug/sgt`. Both are run anyway: the contract names the first, the
# second also catches the uninstrumented census arm, and over-detection is the
# safe direction. The pids are deduplicated before they are reported.
cov_hygiene() {
  local instrumented plain leaked tmp
  instrumented="$(pgrep -f "llvm-cov-target/debug/sgt --data-dir" || true)"
  plain="$(pgrep -f "target/debug/sgt --data-dir" || true)"
  leaked="$(printf '%s\n%s\n' "$instrumented" "$plain" | grep . | sort -un | tr '\n' ' ' || true)"
  tmp="$(find /tmp -maxdepth 1 \( -name 'sgt-demo-*' -o -name '.tmp*' \) 2>/dev/null | wc -l)"
  cov_say "hygiene: leaked daemons [${leaked% }] /tmp entries $tmp"
  printf 'instrumented\t%s\nany_sgt_daemon\t%s\ntmp_entries\t%s\n' \
    "${instrumented//$'\n'/,}" "${plain//$'\n'/,}" "$tmp" \
    > "$COV_ARTIFACTS/$COV_STAGE-hygiene.tsv"
  if [ -n "$leaked" ]; then
    cov_fail "a daemon outlived the stage (pids: ${leaked% }). Its data dir is gone and its \
profile never arrived; reap it and rerun the stage rather than reporting the number."
  fi
}

# Run a measurement command, teeing it into the stage log and returning its
# real exit status through the pipe.
cov_run() {
  cov_say "\$ $*"
  set +e
  "$@" 2>&1 | tee -a "$COV_LOG"
  local rc="${PIPESTATUS[0]}"
  set -e
  return "$rc"
}
