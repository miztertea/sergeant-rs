# 15-validate — semble parity: the baseline's own command, run against the change

Lane: `/var/tmp/hats6/parity`, branch `hats6/semble-parity`.
Head under validation: `04419f00` (`git -C /var/tmp/hats6/parity rev-parse HEAD`).
Baseline pin it is compared against: `ef66bfe81a3b5e3306711df87f10644d345ff7f9`.
Working tree at validation time: `git status --porcelain` → 0 lines before this
artifact was written.

Full logs, captured verbatim, under `/var/tmp/hats6/parity-validate/output/`:
`ranking-suites-validate.txt`, `coverage-gate-validate.txt`,
`floor-suite-validate.txt`, `fmt-validate.txt`, `clippy-validate.txt`.

**Result: PASS. Nothing failed, so nothing is carried forward as a failure.**

---

## 1. Command A — the baseline's ranking/retrieval suites, verbatim

Run exactly as `05-baseline` §1 recorded it, unmodified:

    cd /var/tmp/hats6/parity && TMPDIR=/var/tmp/sgt-test-tmp CARGO_BUILD_JOBS=6 \
      cargo nextest run --no-fail-fast \
        --test w4_rrf_fusion --test w2_lexical_retrieval \
        --test w3b_semantic_retrieval --test w5_search_surface \
        --test w3_semantic_degradation --test a2_acceptance

Verbatim head and tail of the run:

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
    ────────────
     Nextest run ID 09211606-7fa4-40c6-867a-36008a9bb4c4 with nextest profile: default
        Starting 77 tests across 6 binaries
            PASS [   0.005s] sergeant-rs::a2_acceptance the_documented_walk_table_matches_the_register
    …
            PASS [  17.332s] sergeant-rs::w2_lexical_retrieval a_bounded_answer_reports_true_when_the_posting_scan_is_actually_capped
    ────────────
         Summary [  17.332s] 77 tests run: 77 passed, 0 skipped

Exit code **0**. `grep -n "FAIL"` over the log returns nothing (zero FAIL lines);
77 PASS lines, one per test.

Per binary (`grep -oP '(?<=sergeant-rs::)\w+' | sort | uniq -c`), against the
baseline's own table:

| suite | baseline | now | delta |
|---|---|---|---|
| `w4_rrf_fusion` | 17 | **26** | +9 |
| `w2_lexical_retrieval` | 17 | 17 | — |
| `w5_search_surface` | 15 | 15 | — |
| `w3b_semantic_retrieval` | 9 | 9 | — |
| `w3_semantic_degradation` | 6 | 6 | — |
| `a2_acceptance` | 4 | 4 | — |
| **total** | **68 passed** | **77 passed** | **+9, 0 failed, 0 skipped** |

The whole delta lands in `w4_rrf_fusion` — the suite that owns the fusion and
rerank seams the port touched. Nothing outside it changed count, and nothing
outside it changed colour.

### The four tests `05-baseline` §4 said would have to move
`05-baseline` predicted four committed tests a port must rewrite. What the
validated head actually contains:

| predicted test | fate at `04419f00` | evidence |
|---|---|---|
| `an_exact_name_match_is_promoted_over_a_better_fused_score` | **retained, not deleted** — fixture rewritten so the exact-match boost covers the score gap; still asserts `["a.rs","b.rs"]` after `rerank` | `tests/w4_rrf_fusion.rs:1181-1198` |
| `the_rerank_key_is_a2_section_8s_nine_signals_in_the_contracts_own_order` | **renamed** → `..._as_a_score_adjustment` | `git diff ef66bfe8..HEAD -- tests/w4_rrf_fusion.rs` shows the `-fn …_in_the_contracts_own_order` / `+fn …_as_a_score_adjustment` pair |
| `the_fused_score_is_a2_section_7s_one_expression` | **unchanged and green** — step (b) was reverted at `58a1bb64`, so A2 §7's one expression still holds | `tests/w4_rrf_fusion.rs:141`; PASS in the log |
| `the_retrieval_policy_version_is_pinned_…` | **bumped**, to `rrf-k60+a2s8-score-adjust+semble-boosts/5` | `src/runtime/atlas/trace.rs:109`; the pin test PASSes |

The brief's "do not delete `an_exact_name_match_is_promoted_over_a_better_fused_score`
without saying so in the same breath" is satisfied by not deleting it: the test
survives at `tests/w4_rrf_fusion.rs:1181` with an adjusted fixture, and its
doc-comment states the new mechanism.

`a2_acceptance` — the suite whose `every_named_check_exists_in_the_suite_it_names`
`05-baseline` flagged as coupled to those renames — is 4/4 PASS, so no cited
check was left dangling by the rename.

### The load-bearing greens `05-baseline` §4 named
All present and PASS in the log: `the_fused_order_does_not_depend_on_the_callers_limit`
(`00-orient` J0 #2's tripwire — **green**, so the limit-independence pin held and
the discarded `top_k*5` over-fetch never landed),
`the_search_cli_exposes_a2_section_14s_selectors_and_no_weight_knob`,
`a_test_path_loses_to_a_canonical_one_at_an_equal_fused_score`,
`the_three_filter_shaped_signals_are_uniform_because_admissibility_already_applied_them`,
the determinism pair (`the_same_query_…_returns_an_identical_answer` /
`…_identical_fused_answer`), and the `w3_semantic_degradation` six.

## 2. Command B — the #231 wiring gate, verbatim

    cd /var/tmp/hats6/parity && TMPDIR=/var/tmp/sgt-test-tmp CARGO_BUILD_JOBS=6 \
      cargo nextest run --test c2_light -E 'test(coverage_stage_membership)'

Verbatim, in full:

        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
    ────────────
     Nextest run ID 97c146e9-fc9e-40d5-96b0-d11b8d264b30 with nextest profile: default
        Starting 1 test across 1 binary (27 tests skipped)
            PASS [   0.005s] sergeant-rs::c2_light coverage_stage_membership::every_suite_is_wired_or_explicitly_allowlisted
    ────────────
         Summary [   0.005s] 1 test run: 1 passed, 27 skipped

Exit code **0**. Same shape as the baseline's `1 test run: 1 passed, 27 skipped`.

## 3. The wave's new suite — outside the baseline command, run anyway

`w4a_retrieval_floor` did not exist at the pin, so the baseline's command A
cannot name it. It is the wave's own guard and is run here rather than left
unvalidated:

    cd /var/tmp/hats6/parity && TMPDIR=/var/tmp/sgt-test-tmp CARGO_BUILD_JOBS=6 \
      cargo nextest run --no-fail-fast --test w4a_retrieval_floor

Verbatim, in full:

        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
    ────────────
     Nextest run ID 80d6d7e3-88dd-4f1e-ad45-438b321145c0 with nextest profile: default
        Starting 3 tests across 1 binary
            PASS [   0.004s] sergeant-rs::w4a_retrieval_floor the_committed_question_set_keeps_its_shape
            PASS [  31.561s] sergeant-rs::w4a_retrieval_floor the_scanned_corpus_is_this_repository
            PASS [  59.414s] sergeant-rs::w4a_retrieval_floor precision_at_one_over_the_committed_question_set_does_not_regress
    ────────────
         Summary [  59.414s] 3 tests run: 3 passed, 0 skipped

Exit code **0**. Grand total across all three commands: **81 tests run, 81
passed, 0 failed.**

---

## 4. Close-out — all four, with evidence

### 4.1 Every new suite is wired into `scripts/coverage/` (#231) — DONE
The wave adds exactly one suite (`git diff --name-only ef66bfe8..HEAD` lists one
new `tests/*.rs`: `tests/w4a_retrieval_floor.rs`). Grep of `scripts/` for it:

    $ grep -rn "w4a_retrieval_floor" scripts/
    scripts/coverage/c2-suites.sh:418:cov_stage_begin c2-w4a_retrieval_floor
    scripts/coverage/c2-suites.sh:419:cov_run cargo llvm-cov --no-report --test w4a_retrieval_floor --locked || cov_fail "w4a_retrieval_floor failed under instrumentation"
    scripts/coverage/c2-suites.sh:420:cov_stage_end 1 "the w4a_retrieval_floor test binary must write its own profile"

Same three-line shape as its neighbour `c2-w4_rrf_fusion` at `:404-406`. The
independent check is command B above, which is exactly the "orphaned suite"
detector and is green.

### 4.2 `cargo fmt` and `cargo clippy --all-targets` — DONE, both clean
`CARGO_BUILD_JOBS=6 cargo fmt --check` → exit **0**, **0 lines of output**
(`fmt-validate.txt` is empty).
`TMPDIR=/var/tmp/sgt-test-tmp CARGO_BUILD_JOBS=6 cargo clippy --all-targets` →
exit **0**, tail `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in
0.19s`, and `grep -c '^warning\|^error'` → **0**.

### 4.3 Docs the brief names — NAMED AS NOT APPLICABLE, with the reason
The brief (`brief-semble-parity.md`) names **no documentation deliverable**. Its
only doc-shaped instruction is line 83: *"record that as a contract amendment
with the measured delta behind it and escalate; do not silently diverge"* — an
amendment to `A2-RETRIEVAL-INTELLIGENCE.md`, which lives in the **knowledge
library**, not in this lane's tree, and which `10-implement` carried as
escalation #1. Writing it here would be this stage taking a decision that is the
owner's (J0), and would put a knowledge-library edit in a product lane. Not done,
deliberately.

Verified that the lane holds no product doc contradicted by the change:
`grep -rln "rrf-k60\|RETRIEVAL_POLICY_VERSION\|retrieval policy" docs/ *.md`
returns **no hits** — the policy string is documented only in its own Rust
doc-comment (`src/runtime/atlas/trace.rs:56`), which moved with it.

### 4.4 No clock decides correctness anywhere the wave touched — CONFIRMED
The wave's nine changed files (`git diff --name-only ef66bfe8..HEAD`) grepped for
`Instant::now|SystemTime::now|Duration::from|elapsed()|sleep|timeout|BUDGET|deadline`:

- `src/runtime/atlas/fusion.rs`, `src/runtime/atlas/trace.rs`,
  `tests/w4_rrf_fusion.rs`, `tests/w4a_retrieval_floor.rs`,
  `tests/a2_acceptance.rs` — **zero matches**.
- `src/runtime/atlas/db.rs` — the diff hunks contain no clock construct at all
  (`git diff … -- src/runtime/atlas/db.rs | grep -i "now()\|elapsed\|sleep\|deadline\|Duration"` → no output).
- `tests/w5_search_surface.rs:117` — one hit, `deadline: Duration::from_secs(30)`,
  a `WorkerRuntime` field. **Not introduced by this wave**: the file's diff
  against `ef66bfe8` contains no `deadline` line. It is the anydoc worker's hang
  guard (`src/runtime/atlas/worker.rs:56`: *"The deadline above is a HANG guard"*),
  it bounds a child process rather than asserting anything, and no assertion in
  the file reads a wall clock or a ratio.

No deadline loop, no `Instant::now() + BUDGET`, no wall-clock or ratio assertion
was added. The relevance floor (`w4a_retrieval_floor`) asserts a **p@1 count**,
not a duration.

---

## 5. Carried forward

Nothing failed, so this stage carries no failure. Two things the panel should
still see, both raised by earlier stages and neither resolved by a green run:

1. `10-implement`'s three J0 escalations stand unchanged — A2 §8 signal 5 no
   longer influencing order; the p@1 floor threshold `.57` being a **new
   acceptance criterion the owner must set**; and the `RETRIEVAL_POLICY_VERSION`
   bump. A green suite is not the owner's ruling on any of them.
2. The measured outcome is **not parity**: p@1 `.404 → .538` against semble's
   `.731` upper bound. The suites passing says the port is correct, not that the
   wave's objective is met.
