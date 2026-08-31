# 10-implement — #334: journal writes silently fail; the invariant is prose

Base `f2b7a3720fe99c3a2d112cf8296339c98fdcec1b` (`00-orient` §1, not re-pinned).
Five commits, one per seam, each red-green-minimal with its red captured in the
commit message it belongs to.

## Seams

Confirmed, not chosen here: the seam list is `00-orient` §6's in-scope boundary,
already settled upstream (J3). `@@test-first`'s J0 — "no test may be written at
an unconfirmed seam" — is satisfied by that, not waived.

| # | Commit | Seam | Red before it |
|---|---|---|---|
| 1 | `4af4959a` | `Core::flush` — the release choke point every lock hold passes through | `SeqMismatch { expected: 2, found: 3 }` on the next commit |
| 2 | `0ab07dbd` | `Core::unabsorbed_holds()` — the breach is counted, not excused | `left: 0, right: 1` |
| 3 | `04326433` | `intelligence_add_source` (`src/api.rs:5751`) + the source-level guard | the guard named `intelligence_add_source` and nothing else |
| 4 | `93ed9ce3` | the issue's cascade, and its recovery | registry answered for 1 of 5 events |
| 5 | `4736f839` | shutdown journals **and publishes** `daemon.stopped` | `got []` — published nothing |

## What the change is

**Two enforcements, deliberately not one.** `absorb_journaled`'s doc comment
said "every direct-journal writer must call this before releasing the hold" and
nothing checked it, which is how `intelligence_add_source` was written without
it.

- **Runtime, unevadable, repairs:** `Core::absorb_before_release`, called from
  `Core::flush`. `flush` is the single point every hold ends at — `CoreGuard::flush`
  and `CoreGuard`'s `Drop` backstop both call it — so a writer that forgets,
  including one written after this change, cannot leave the registry behind.
  Report-only on failure: turning a failed fold into an `Err` there would drop a
  group already on disk unpublished, losing the fan-out as well as the fold.
  `flush`'s own contract (fsync, then publish) is untouched.
- **Source-level, evadable, fails in review:** `every_direct_journal_writer_in_the_api_absorbs_before_releasing_its_hold`
  reads `src/api.rs`'s production region and requires every function handing
  `&mut …journal` to an Atlas writer to also fold what it appended.

Neither alone is the invariant: the backstop is silent (it repairs), the source
guard is evadable (a writer could hold the journal under another name).

The located defect itself (`00-orient` §4) is closed at its own call site too —
`intelligence_add_source` now carries the same three lines the other four
direct-journal writers already carried (R2). The backstop would have covered it;
leaving the writer wrong because the backstop catches it is how the next one
arrives.

## Rungs

- **R2** for the fold itself (`Core::absorb_journaled` already existed — nothing
  new was built to close the defect) and for the source guard's posture
  (`tests/x5_a1a_acceptance.rs`'s `the_trigger_is_reachable_from_production_code_not_only_a_test_module`
  is the in-repo precedent for reading `src/api.rs` as text).
- **R7** for *where* the fold is invoked from — the release path rather than each
  writer — and for the breach counter. Lower rungs checked and why they failed:
  - **R4 (native language feature: privacy + the borrow checker), checked and
    rejected.** The obvious compile-time pin is to make `Core::journal` private
    and hand `&mut Journal` out only inside a scope that folds afterwards. It
    does not work here, for two independent reasons, both read out of the tree
    rather than assumed:
    1. The five production writers hold the journal *across an `await`*:
       `with_atlas_write` (`src/api.rs:4768`) is `async fn` and takes the sync
       closure, so the borrow spans `.await`. A synchronous `Core::with_journal(|j| …)`
       scope cannot wrap it without restructuring every Atlas write path.
    2. Integration tests in `tests/` legitimately need `&mut Journal`
       (`tests/m4_backends.rs:437`, `:448`, `:5877`; `tests/w1_deterministic_filter.rs:398`;
       `tests/w2_lexical_retrieval.rs:968`; `tests/i9_floor_pinning.rs:716`).
       `tests/` is a separate crate, so `#[cfg(test)]` does not reach it and any
       accessor they use is public — which reopens the exact hole the privacy
       was for.
  - **R6 (one line):** the fold is one line; the report and the counter that
    makes it assertable are not, and without them the repair is silent.
- **R1** for seams 4 and 5: no new mechanism, existing code driven over the state
  the issue's daemon was in.
- **J3** throughout: cause, boundary, and the §3a/§3b corrections are `00-orient`'s,
  read at source.
- **J5** where it bit: the test for issue item 3 asserts the stop event is
  published, not merely journaled, because H1 §6 makes the journal the authority
  and A1-01 makes the derived surfaces answerable to it — a test asserting only
  the journal half passes against the live defect.

## Deliberately not asserted

`Core::commit` appends before it folds, so a failed commit's event is already in
the journal and the gap widens by one per failure (`00-orient` §3a — the issue's
`expected 146, got 149` is three prior failed commits, not three concurrent
appenders). Seam 4 pins the **recovery** and not the widening: the ordering is
inside the J0 `00-orient` §7 escalated, and pinning it in a test would settle a
first-principle contract by accident.

Also untouched, per `00-orient` §6's out-of-scope list: the `failed to journal …`
wording at `src/daemon.rs:1485`/`:1647` (which §3b shows is the wrong claim about
the wrong layer), any retry/serialisation redesign, and the scan front door.

## Non-vacuity

Every test here was watched red.

- Seams 1, 2, 3: red on the tree as it stood, before the code that fixes them.
  Seam 3's red is the brief's own "add a writer that skips it, watch the guard go
  red" — taken against the real writer that skipped it, and the four writers that
  do absorb are matched by the same pattern and pass it, which is what validates
  the pattern instead of trusting a count of zero.
- Seams 4 and 5 were green on arrival (seam 1 already repairs the state), so
  their red was taken by probe: `self.absorb_before_release();` commented out of
  `Core::flush`, the test run, the line restored by `git checkout`. Both reds are
  in their commit messages verbatim.
- Seam 2 is a matched pair — a writer that skips and the identical fixture that
  does not — so it measures the absorb rather than the fixture.

## #231 — coverage membership, wired at birth and proven live

`tests/f334_journal_integrity.rs` is wired into `scripts/coverage/c2-suites.sh`
in the same commit that created it (`4af4959a`). The membership guard was run
against the unwired suite first and went red naming it:

```
new orphaned suite(s) wired into neither c2-suites.sh nor c3-spawning-suites.sh,
and not named in this test's ALLOWLIST with a reason: ["f334_journal_integrity"]
```

then green once wired:

```
$ cargo nextest run --test c2_light -E 'test(coverage_stage_membership)'
1 test run: 1 passed, 27 skipped
```

Run, not reasoned about.

## Verification (this stage's own; the panel is separate — R-S0-12)

| Command | Result |
|---|---|
| `cargo nextest run --no-fail-fast --test f334_journal_integrity --test m1_event_core --test m5_projections --test w1b_overlay_lifecycle_trigger --test y5_external_git_triggers --test x5_a1a_acceptance --test w3_allowlist_equivalence` | `73 tests run: 73 passed` (baseline's 67 + this suite's 6) |
| `cargo nextest run --test m2_daemon_api -E 'test(shutdown) + test(t11d) + test(t11e)'` | `3 tests run: 3 passed` |
| `cargo nextest run --test c2_light -E 'test(coverage_stage_membership)'` | `1 test run: 1 passed` |
| `cargo nextest run --test w3_prune_engine` | `6 tests run: 6 passed` — not in the baseline subset; added because `Core::flush` is a global choke point and prune is the code that moves the journal's floor under it |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --locked -- -D warnings` | clean — `05-baseline` recorded this as *not* baselined, so it is new work here, not pre-cleared state |

The full suite was not run locally: CI by SHA is the exhaustive pass.

## Conflicts

None. No merge or rebase was needed; `@@resolve-conflicts` did not fire.

## Still blocked, unchanged

Issue item 2 — *is a failed journal write ever acceptable?* — remains escalated
at `00-orient` §7 with its recommendation and evidence. Nothing in these five
commits decides it, and none of them depended on the answer.
