#!/usr/bin/env bash
# F1 — the control arm of the flake census: N uninstrumented full-suite runs.
#
#   scripts/coverage/f1-control-census.sh [N]        # N defaults to 10
#
# This arm exists so that "fails only under instrumentation" can be said at
# all (R-S0-7): the claim is only available with a green control arm, and a
# test that flakes in both arms is a flake, not an instrument artifact. Ten
# runs is the M3 precedent — the M3-era parallel-flake class showed up at
# roughly one run in twenty, so a single green run has never been evidence
# here.
#
# Default parallel harness, no `--test-threads` pinning, no `--ignored`: the
# arm has to be the same shape as the run everyone else does, or it measures
# a different program. Each run's failures are recorded by name; the full
# output is kept only for runs that failed, because ten green transcripts are
# ten copies of nothing.
#
# Knobs: COV_CENSUS_CEILING_S (default 14400 — the ~4 h ceiling; a run that
# would cross it is not started, and the shortfall is recorded with its
# reason rather than left as a silent gap).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

N="${1:-10}"
CEILING="${COV_CENSUS_CEILING_S:-14400}"

cov_stage_begin f1
FAILURES="$COV_ARTIFACTS/f1-failures.tsv"
RUNS="$COV_ARTIFACTS/f1-runs.tsv"
printf 'run\ttest\n' > "$FAILURES"
printf 'run\toutcome\twall_s\tfailed\n' > "$RUNS"

# The suite is built once, outside the timed loop: a census of *test* flakes
# should not have a cold compile in run 1 and a warm one in runs 2-10.
cov_run cargo test --no-run || cov_fail "the suite does not build"

started="$(date +%s)"
completed=0
for run in $(seq 1 "$N"); do
  elapsed=$(( $(date +%s) - started ))
  if [ "$run" -gt 1 ] && [ "$elapsed" -ge "$CEILING" ]; then
    cov_say "ceiling reached after $completed run(s) (${elapsed}s ≥ ${CEILING}s) — stopping short"
    break
  fi
  out="$COV_ARTIFACTS/f1-run-$run.log"
  t0="$(date +%s)"
  set +e
  cargo test > "$out" 2>&1
  rc=$?
  set -e
  wall=$(( $(date +%s) - t0 ))
  failed="$(grep -c '^test .* FAILED$' "$out" || true)"
  grep '^test .* FAILED$' "$out" | sed "s/^test /$run\t/; s/ \.\.\. FAILED$//" >> "$FAILURES" || true
  if [ "$rc" -eq 0 ]; then
    printf '%s\tok\t%s\t0\n' "$run" "$wall" >> "$RUNS"
    rm -f "$out"          # a green transcript is not evidence of anything
  else
    printf '%s\tfailed\t%s\t%s\n' "$run" "$wall" "$failed" >> "$RUNS"
    cov_say "run $run FAILED ($failed test(s)) — transcript kept at $out"
  fi
  completed=$((completed + 1))
  cov_say "run $run: rc=$rc wall=${wall}s failed=$failed"
  cov_hygiene
done

printf 'arm\tcontrol (uninstrumented)\nrequested\t%s\ncompleted\t%s\nceiling_s\t%s\ntotal_s\t%s\n' \
  "$N" "$completed" "$CEILING" "$(( $(date +%s) - started ))" \
  > "$COV_ARTIFACTS/f1-census.tsv"
[ "$completed" -eq "$N" ] \
  || cov_say "SHORTFALL: $completed of $N runs completed — recorded, not hidden (R-S0-7)"

cov_stage_end 0 "the control arm is uninstrumented and must produce no profiles"
