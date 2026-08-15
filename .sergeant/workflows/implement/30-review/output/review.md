# 30-review output — WB: restart-reconciliation bug (issue #129)

**Reviewed commit:** `581b85a tests/m3_execution: fix t8 extra-OBSERVE assertion to be timing-robust`  
**Diff scope:** `tests/m3_execution.rs` only (one file, 33 lines changed)  
**Review effort:** high (three-angle code-review: line-by-line correctness, removed-behavior audit, cross-file tracer)

---

## Summary

The implementation is correct. The fix is accepted with one observation noted for the record.

---

## Root cause (established by stage 10)

`t8`'s assertion counted *total* OBSERVEs on the survivor's `first_execution`
(expected 2 = 1 pre-restart + 1 from reconciliation). On macOS, the daemon's
200 ms completion driver (`drive_completions`) fires during the first daemon's
lifetime before shutdown — the HTTP submits and git worktree creation takes
> 200 ms on this host — so the total can be 3 instead of 2. This is a
scheduler-pacing difference between hosts; the reconciliation logic in
`src/runtime/recovery.rs` and `src/runtime/engine.rs` (`reconcile_work`,
`Resumed` branch + `run_inline`) correctly observes exactly once.

---

## What the fix does

Captures `obs_before_reconcile` immediately after `handle.shutdown()` (the
first daemon's completion driver is stopped at that point) and asserts that
the *delta* `obs_after_reconcile - obs_before_reconcile == 1`. This isolates
what reconciliation itself contributed, independent of how many times the
first daemon's completion driver polled `first_execution` during its lifetime.

The assertion still catches both failure directions:
- 0 → reconciliation skipped its OBSERVE entirely
- ≥ 2 → genuine double-observe in the reconciliation path

---

## Code-review findings (scoped to the diff)

Three angle agents ran. Their findings about files outside the diff (the
broader macOS portability pass committed in parallel waves WA and WC) are out
of scope here. One finding applies to my diff:

**Candidate: unsigned `usize` subtraction could underflow (Angle A)**  
`obs_after_reconcile - obs_before_reconcile` where both are `usize`.  
Verdict: **CONFIRMED SAFE** in this context.  
`FakeBackend.observations` is a `Vec<String>` with push-only access
(`src/backend/fake.rs:906` — only `push`, never cleared or truncated). Between
the two snapshots, no code path removes entries. `obs_after_reconcile >=
obs_before_reconcile` is guaranteed by the append-only semantics of the fake
backend. The concern is PLAUSIBLE in general but does not apply here. A
`checked_sub` guard would add defensive noise to a test that deliberately
relies on the fake backend's invariants.

---

## Test verification

Ran 3 times, isolated (`--test-threads=1`), all passed:

```
--- Run 1 ---
test t8_restart_resumes_unambiguous_work_and_blocks_ambiguous_work ... ok
--- Run 2 ---
test t8_restart_resumes_unambiguous_work_and_blocks_ambiguous_work ... ok
--- Run 3 ---
test t8_restart_resumes_unambiguous_work_and_blocks_ambiguous_work ... ok
```

Pre-fix (on `integration/macbook-arrival-2026-08-15` before this commit):
deterministic failure with `left: 3, right: 2`.

---

## L6 adjacent-append audit

No journal-appending code was changed. `src/runtime/recovery.rs` and
`src/runtime/engine.rs`'s reconciliation path are untouched. This is a
test-precision fix only. No adjacent-append crash window is introduced or
resolved by this change.

---

## Disposition

**Accept.** The commit carries `Fixes #129` and is justified: the root cause
is diagnosed (scheduler-pacing difference surfaces a pre-existing counting
error in the test assertion), the fix is minimal and correct (delta assertion
rather than absolute total), and the invariant it pins is stronger — it
catches both the zero case and the double-observe case explicitly.
