#!/usr/bin/env bash
# C4 — one report over everything C1–C3 pooled.
#
#   scripts/coverage/c4-report.sh
#
# Three exports off one merge: the summary table (the baseline's numbers), an
# lcov file (what a findings pass reads region-by-region), and the HTML tree
# (what a human reads). The lcov and HTML land outside the repo, under
# target/llvm-cov-reports/ — they are regenerable from the profdata in
# seconds and would be megabytes of committed noise. What gets committed is
# the summary, the counts, and the checksums that say which report the
# baseline was read from.
#
# THE TRAP THIS STAGE GUARDS AGAINST (measured on 0.8.7, see the README):
# with no profraw files present, `cargo llvm-cov report` does not fail and
# does not merge — it re-prints the numbers from a **stale** .profdata left
# by an earlier run, exit 0, no warning. A report that silently describes a
# different tree is worse than no report, so this stage refuses to run
# without profraws and proves the merge actually happened by watching the
# profdata change.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

COV_REPORTS="${COV_REPORTS:-$COV_ROOT/target/llvm-cov-reports}"

cov_stage_begin c4
mkdir -p "$COV_REPORTS"

PROFRAWS="$(cov_profraw_list | grep -c . || true)"
cov_say "profraws to merge: $PROFRAWS"
[ "$PROFRAWS" -gt 0 ] || cov_fail "there are no profraw files in $COV_TARGET_DIR. Reporting now \
would re-print a previous run's .profdata as if it were this run's measurement (measured \
behavior, not a hypothesis). Run C1–C3 first."

PROFDATA="$(find "$COV_TARGET_DIR" -maxdepth 1 -name '*.profdata' -printf '%p\n' | head -1)"
BEFORE_STAMP="$([ -n "$PROFDATA" ] && stat -c %y "$PROFDATA" || echo none)"

# ---- the convention's report command lines, verbatim ----------------------
cov_say "\$ cargo llvm-cov report --summary-only"
cargo llvm-cov report --summary-only 2>&1 \
  | tee "$COV_ARTIFACTS/c4-summary.txt" | tee -a "$COV_LOG"
[ "${PIPESTATUS[0]}" -eq 0 ] || cov_fail "the summary report failed"

cov_run cargo llvm-cov report --lcov --output-path "$COV_REPORTS/lcov.info" \
  || cov_fail "the lcov export failed"

cov_run cargo llvm-cov report --html --output-dir "$COV_REPORTS/html" \
  || cov_fail "the html report failed"

# ---- proof that this report describes this run ----------------------------
PROFDATA="$(find "$COV_TARGET_DIR" -maxdepth 1 -name '*.profdata' -printf '%p\n' | head -1)"
[ -n "$PROFDATA" ] || cov_fail "no .profdata exists after reporting — nothing was merged"
AFTER_STAMP="$(stat -c %y "$PROFDATA")"
[ "$AFTER_STAMP" != "$BEFORE_STAMP" ] || cov_fail "the .profdata did not change across the \
report ($PROFDATA). The merge was skipped and these numbers are the previous run's."

MERGED="$(cov_profraw_list_lines)"
{
  printf 'profraw_present\t%s\n' "$PROFRAWS"
  printf 'profraw_merged\t%s\n' "${MERGED:-unknown}"
  printf 'profdata\t%s\n' "$PROFDATA"
  printf 'profdata_bytes\t%s\n' "$(stat -c %s "$PROFDATA")"
  printf 'lcov_bytes\t%s\n' "$(stat -c %s "$COV_REPORTS/lcov.info")"
  printf 'lcov_sha256\t%s\n' "$(sha256sum "$COV_REPORTS/lcov.info" | cut -d' ' -f1)"
  printf 'lcov_path\t%s\n' "$COV_REPORTS/lcov.info"
  printf 'html_path\t%s\n' "$COV_REPORTS/html/index.html"
} > "$COV_ARTIFACTS/c4-report-provenance.tsv"
cov_say "merged $MERGED of $PROFRAWS profraw file(s) into $PROFDATA"
cov_say "summary: $COV_ARTIFACTS/c4-summary.txt · lcov+html: $COV_REPORTS"

# Reporting runs no tests and must add no profiles of its own.
cov_stage_end 0 "C4 only merges and exports"
