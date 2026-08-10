#!/usr/bin/env bash
# C0 — the instrument's own environment, checked before any of it is trusted.
#
#   scripts/coverage/c0-show-env.sh
#
# One Unknown from the proposal (§14) is resolved here and nothing else is:
# whether the `LLVM_PROFILE_FILE` pattern cargo-llvm-cov sets is **absolute**.
# It has to be. Two of this repo's suites run the `sgt` client with its cwd
# set to a `TempDir` that is deleted at the end of the test; a relative
# profraw pattern would scatter every subprocess profile into those directories
# and then delete them, and the only symptom would be numbers that are quietly
# too low. A hard stop here is cheaper than a baseline nobody can explain.
#
# Recorded, not asserted from documentation (doctrine 1 / L1 at the tool
# boundary): the exact `show-env` output goes into the artifacts dir verbatim.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

cov_stage_begin c0

# ---- the convention's environment, verbatim -------------------------------
cov_say "\$ cargo llvm-cov show-env"
cargo llvm-cov show-env > "$COV_ARTIFACTS/c0-show-env.txt" 2>&1 \
  || cov_fail "cargo llvm-cov show-env failed (output in c0-show-env.txt)"
tee -a "$COV_LOG" < "$COV_ARTIFACTS/c0-show-env.txt" > /dev/null

PATTERN="$(sed -n "s/^LLVM_PROFILE_FILE='\(.*\)'$/\1/p" "$COV_ARTIFACTS/c0-show-env.txt")"
[ -n "$PATTERN" ] || cov_fail "show-env printed no LLVM_PROFILE_FILE (output in c0-show-env.txt)"
cov_say "LLVM_PROFILE_FILE=$PATTERN"

VERDICT=absolute
case "$PATTERN" in
  /*) ;;
  *) VERDICT=relative ;;
esac
[ "$VERDICT" = absolute ] || cov_fail "the profraw pattern is RELATIVE ($PATTERN). Every \
subprocess launched with a temporary cwd would write its profile into a directory that is \
about to be deleted. Stop: the baseline cannot be collected until this is pinned down."

# %p keeps two processes from writing the same file. Without it the daemon and
# the client that spawned it would race for one path, and the loss would look
# like uncovered code rather than a clobbered file.
case "$PATTERN" in
  *%p*) cov_say "pattern is per-process (%p present)" ;;
  *) cov_fail "the pattern has no %p ($PATTERN): concurrent processes would overwrite \
each other's profiles" ;;
esac

# What show-env reports is the *base* target dir; a managed run (the flow this
# harness uses — `cargo llvm-cov --no-report …`, never the show-env flow)
# writes its objects and profraws one level down, in target/llvm-cov-target/.
# Measured on 0.8.7, recorded here so the difference is on the record rather
# than a surprise at C1, where the profraws are counted in that directory.
cov_say "show-env target dir : $(sed -n 's/^CARGO_LLVM_COV_TARGET_DIR=//p' "$COV_ARTIFACTS/c0-show-env.txt")"
cov_say "managed collection  : $COV_TARGET_DIR (where C1–C3 count profraws)"

{
  printf 'pattern\t%s\n' "$PATTERN"
  printf 'absolute\t%s\n' "$VERDICT"
  printf 'per_process\tyes\n'
  printf 'managed_collection_dir\t%s\n' "$COV_TARGET_DIR"
} > "$COV_ARTIFACTS/c0-verdict.tsv"

# C0 runs no tests, so it produces no profraws — and that is the assertion.
cov_stage_end 0 "C0 inspects the environment and must not run anything"
