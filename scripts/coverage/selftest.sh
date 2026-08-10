#!/usr/bin/env bash
# scripts/coverage/selftest.sh — the coverage harness's own accounting, tested.
#
#   scripts/coverage/selftest.sh
#
# An instrument nobody has tried is a claim (the rule this repo's reapers are
# already held to, applied to the thing that grades them). Every check in
# `common.sh` is a claim about what would fail, and two of them turned out not
# to be true when someone tried:
#
#   * the profraw accounting computed additions only, so a stage that deleted
#     every earlier stage's profile still printed "stage ok" and exit 0.
#
# The fix is pinned here rather than by a real collection run, which costs ten
# minutes and a cold DuckDB build. The cases run against a scratch
# COV_TARGET_DIR full of empty files with `.profraw` names, with COV_PROFDATA
# stubbed to `/bin/true` — the mergeability classifier is a separate claim
# with its own evidence (README measured claim 6) and is not what these cases
#
# Runs in seconds, needs no instrumented build, and is executed by
# `tests/m6_surfaces.rs` so the repo's ordinary gate covers it.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SELF="$HERE/$(basename "${BASH_SOURCE[0]}")"

# --------------------------------------------------------------- the cases
#
# Each runs in its own process (a `cov_fail` is an `exit 1`, which is exactly
# what half of these are checking for), with the environment the driver sets.

seed_profraws() {
  local name
  for name in "$@"; do : > "$COV_TARGET_DIR/$name.profraw"; done
}

case_loss_undeclared() {
  seed_profraws old-1 old-2
  cov_stage_begin loss-undeclared
  rm -f "$COV_TARGET_DIR"/old-1.profraw "$COV_TARGET_DIR"/old-2.profraw
  seed_profraws new-1 new-2
  cov_stage_end 2 "the stage produced two profraws and destroyed two"
}

case_loss_declared() {
  seed_profraws old-1 old-2
  cov_stage_begin loss-declared
  cov_expect_profraw_loss "the selftest's stand-in for F2's clean --profraw-only"
  rm -f "$COV_TARGET_DIR"/old-1.profraw "$COV_TARGET_DIR"/old-2.profraw
  seed_profraws new-1 new-2
  cov_stage_end 2 "declared loss must be recorded, not fatal"
}

case_no_loss() {
  seed_profraws old-1 old-2
  cov_stage_begin no-loss
  seed_profraws new-1 new-2
  cov_stage_end 2 "the pooling case: nothing before disappeared"
}

if [ "${1:-}" = "--case" ]; then
  # shellcheck source=common.sh
  . "$HERE/common.sh"
  # The hygiene sweep is a different claim with different evidence, and it
  # cannot run from here: `cargo test` executes this script *inside* the m6
  # suite, which has live `sgt` daemons of its own in parallel, so a real
  # sweep would report the suite that invoked it and fail every case. Stubbed
  # loudly rather than silently skipped — the sweep's own behavior (it does
  # fail a stage on a leaked daemon) is exercised by the stages themselves.
  cov_hygiene() { cov_say "hygiene: not swept — the selftest runs inside the test suite"; }
  "case_${2//-/_}"
  exit 0
fi

# -------------------------------------------------------------- the driver

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
FAILURES=0

say() { printf '   %s\n' "$*"; }
fail() {
  printf '\033[31mselftest: %s\033[0m\n' "$*" >&2
  FAILURES=$((FAILURES + 1))
}

# run <case> — in a scratch tree whose path contains digits, on purpose.
run() {
  # Two statements, not one `local a=… b=…`: bash expands every word of a
  # `local` before the builtin runs, so `$name` would be unbound in `$dir`.
  local name="$1"
  local dir="$SCRATCH/$name/sub1234"
  mkdir -p "$dir/target" "$dir/artifacts"
  set +e
  env COV_TARGET_DIR="$dir/target" \
      COV_ARTIFACTS="$dir/artifacts" \
      COV_PROFDATA=/bin/true \
      COV_MIN_FREE_GB=0 \
      "$SELF" --case "$name" > "$SCRATCH/$name.out" 2>&1
  RC=$?
  set -e
  OUT="$(cat "$SCRATCH/$name.out")"
  TSV="$dir/artifacts"
}

# field <tsv-file> <key>
field() { sed -n "s/^$2\t//p" "$1"; }

printf '\n\033[1m== coverage harness selftest\033[0m\n'

# 1. Undeclared loss must fail the stage. Before this check existed, this
#    exact shape printed "stage ok" and exited 0.
run loss-undeclared
if [ "$RC" -eq 0 ]; then
  fail "a stage that destroyed two earlier profraws exited 0 — the loss half of the \
accounting is not enforced (R-S0-6)"
elif ! printf '%s' "$OUT" | grep -q 'profraw file(s) that existed when this stage began are gone'; then
  fail "the stage failed, but not with the loss message: $OUT"
else
  say "undeclared profraw loss fails the stage (exit $RC)"
fi
if [ "$(field "$TSV/loss-undeclared-accounting.tsv" profraw_lost)" != "2" ]; then
  fail "the accounting tsv must record profraw_lost 2, got \
'$(field "$TSV/loss-undeclared-accounting.tsv" profraw_lost)'"
fi
if [ "$(wc -l < "$TSV/loss-undeclared-profraw-lost.txt")" != "2" ]; then
  fail "each lost profraw must be named in loss-undeclared-profraw-lost.txt"
fi

# 2. Declared loss is recorded with its reason and does not fail — F2's case.
run loss-declared
[ "$RC" -eq 0 ] || fail "a declared loss must not fail the stage: $OUT"
[ "$(field "$TSV/loss-declared-accounting.tsv" profraw_lost)" = "2" ] \
  || fail "a declared loss must still be counted"
printf '%s' "$(field "$TSV/loss-declared-accounting.tsv" profraw_loss_expected)" \
  | grep -q "clean --profraw-only" \
  || fail "a declared loss must carry its reason into the committed tsv"
[ -f "$TSV/loss-declared-profraw-lost.txt" ] \
  || fail "a declared loss must still name the files"
say "declared profraw loss is recorded with its reason and does not fail"

# 3. The pooling case — nothing lost — stays green and says so.
run no-loss
[ "$RC" -eq 0 ] || fail "the no-loss case must pass: $OUT"
[ "$(field "$TSV/no-loss-accounting.tsv" profraw_lost)" = "0" ] \
  || fail "the no-loss case must record profraw_lost 0"
[ "$(field "$TSV/no-loss-accounting.tsv" profraw_loss_expected)" = "no" ] \
  || fail "a stage that declared nothing must say so"
if [ -e "$TSV/no-loss-profraw-lost.txt" ]; then
  fail "no lost-file list may exist when nothing was lost — an empty one reads as \
'checked and clean'"
fi
say "the pooling case is green and records profraw_lost 0"

if [ "$FAILURES" -gt 0 ]; then
  printf '\n\033[31mselftest: %s check(s) failed\033[0m\n' "$FAILURES" >&2
  exit 1
fi
printf '\n\033[1mselftest: all checks passed\033[0m\n'
