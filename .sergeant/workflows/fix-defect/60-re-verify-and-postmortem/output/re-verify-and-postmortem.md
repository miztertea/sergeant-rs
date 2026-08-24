# Re-verify and postmortem — #234 dirty-patch trailing-newline corruption

## 1. Confirmed findings: fixed or recorded unfixed

`55-refute/output/findings.md` confirmed 11 of 19 findings. Commit
`f464a05e` ("60-re-verify: fix 55-refute's confirmed findings"), landed
by an earlier turn of this same Work ahead of this stage's execution,
disposed of all 11:

| id | axis | disposition |
|---|---|---|
| F-SF-01 | spec-fidelity | positive confirmation (core ask met); no code defect, no action |
| F-SF-02 | spec-fidelity | positive confirmation (test matches issue's acceptance ask); no action |
| F-SF-03 | spec-fidelity | positive confirmation (H2 fixed as byproduct); no action |
| F-SF-04 | spec-fidelity | **fixed** — `retain_dirty_patch_preserves_non_utf8_bytes_verbatim` added (`f464a05e`) |
| F-IV-01 | invariants | positive confirmation (error path unchanged); no action |
| F-IV-03 | invariants | **recorded unfixed, by design** — see below |
| F-IV-04 | invariants | positive confirmation (`write_atomic` coercion is safe); no action |
| F-IV-05 | invariants | **fixed** — `the_git_binary_is_overridable_via_env` extended to cover `git_bytes` (`f464a05e`) |
| F-SI-01 | simplicity | **fixed** — `run()` factored out of `git()`/`git_verbatim()`/`git_bytes()` (`f464a05e`) |
| F-TH-01 | test-honesty | same gap as F-SF-04, **fixed** by the same test |
| F-TH-05 | test-honesty | **fixed** — padding assertion removed from `retain_dirty_writes_a_patch_git_apply_accepts` (`f464a05e`) |

F-SF-01/02/03/IV-01/IV-04 are positive claims the panel confirmed true
about the existing fix, not defects — "confirmed" here means the panel
could not overturn the claim, not that a bug was found. No code change
answers a true claim.

**F-IV-03 (invariants, low, confirmed) is recorded unfixed by design.**
The claim: `.dirty.patch` files (and their `PatchInfo.bytes`) already
written to disk by pre-fix code remain corrupt/mis-sized after upgrade;
nothing in this change detects or migrates them. This Work's own brief
explicitly places "repairing pre-existing corrupt patches on estates" out
of scope as release-note/operator matter, not a code change — there is no
migration path to add without inventing scope beyond #234. No fix commit
addresses it; this document is the record of that decision.

Only confirmed findings were touched. The 8 refuted findings (F-SF-05,
F-IV-02, F-SI-02, F-SI-03, F-SI-04, F-TH-02, F-TH-03, F-TH-04) were left
alone, per `@@fix-confirmed`'s "no opportunistic change" bound.

## 2. Re-attack: defects the fixes introduced

Subject: every fix commit that exists by this point —
`56927a14` (the original fix) and `f464a05e` (this stage's
confirmed-finding fixes). Re-read both diffs in full against current
`src/runtime/git.rs`, `src/runtime/surface.rs`,
`tests/c11_injectable_git.rs`, and `tests/m7_docker_executor.rs`.

Checked specifically:

- **`git_bytes`'s error path**: identical shape to `git()`/`git_verbatim`'s
  after the `run()` factoring — same `GitError::Spawn`/`GitError::Failed`
  construction, same lossy-trim on stderr only (stdout is the one
  byte-exact path, per the doc comment's own contract). No behavior
  divergence introduced by the refactor (`git.rs:116-192`).
- **`write_atomic(&path, &diff)`** taking `&Vec<u8>` where `&[u8]` is
  expected: auto-deref coercion, verified it compiles and the atomic
  write/fsync/rename recipe (`fsutil.rs:105-137`) is untouched.
- **`git apply --check`/`--stat` in a scratch dir with no `git init`**
  (`m7_docker_executor.rs`'s `walk_for_marker`): independently verified
  outside the test suite that `git apply --check` against a plain
  non-repository directory succeeds for a new-file-addition patch (the
  shape `validated.txt` always is) — not a defect, `git apply` does not
  require an enclosing repository for this operation.
  Full run of `m7_docker_executor.rs` (18 tests, 89.71s): all pass.
- **`retain_dirty_patch_preserves_non_utf8_bytes_verbatim`'s claim that
  Git still emits a text diff (not `--binary`'s base85 encoding) for a
  single stray `0xFF` byte with no nearby `NUL`**: re-read the test's own
  captured ground truth (`raw.contains(&0xFFu8)` sanity assertion) — the
  test does not merely assume this, it asserts it before trusting the
  comparison.
- **`the_git_binary_is_overridable_via_env`'s new `git_bytes` assertion**:
  confirmed it exercises the same `command()`/`SGT_GIT_BIN` override path
  as `git()`, not a parallel one that could silently diverge.

No new defect found in either fix commit. Full targeted re-run:

```
cargo test --lib runtime::surface::tests   → 41 passed, 0 failed
cargo test --test c11_injectable_git       → 1 passed, 0 failed
cargo test --test m7_docker_executor       → 18 passed, 0 failed
```

## 3. Test-honesty audit

Every test added or changed across both fix commits:

| test | file | audit |
|---|---|---|
| `retain_dirty_writes_a_patch_git_apply_accepts` | `surface.rs` | Asserts last byte `== b'\n'`, then `git apply --check` and `git apply` against a *second, independent* clean checkout (not the source worktree), then byte-equality of the applied content. Cannot pass on unfixed code — the pre-fix trimmed capture ends in `}`, not `\n`, and `git apply --check` would reject it outright. Confirmed by construction, matches F-SI-03/F-TH-02's refuted objections which already established this. |
| `retain_dirty_patch_preserves_non_utf8_bytes_verbatim` | `surface.rs` | Captures git's own diff bytes independently (`Command::new("git")`, not through any helper under test) as ground truth, sanity-asserts the ground truth itself contains the raw `0xFF` byte, then asserts the captured `.dirty.patch` is byte-identical and contains no U+FFFD substitution sequence. Cannot pass against pre-`git_bytes` code — `String::from_utf8_lossy` would substitute the byte. Proves what it claims. |
| `the_git_binary_is_overridable_via_env` (extended) | `c11_injectable_git.rs` | Added assertion runs `git_bytes` through the same scripted-binary override as the pre-existing `git()` assertion and checks the same echoed-invocation substrings. Not vacuous — a `git_bytes` that failed to read `SGT_GIT_BIN` would produce a `GitError::Spawn` (real `git` binary absent in the harness) and `.expect()` would panic, not silently pass. |
| `walk_for_marker` `git apply --stat`/`--check` addition | `m7_docker_executor.rs` | F-TH-02 raised whether this branch is guaranteed to execute given a sibling short-circuit arm; refuted in `55-refute` with a concrete trace (`retain_dirty` removes the worktree directory before this walk ever runs, so the plain-file arm cannot fire for this scenario). Re-checked that trace against current `surface.rs:1623-1662` (`retain_dirty`) and `m7_docker_executor.rs:1465-1499`(polling for `completed_dirty` before walking) — still holds. |

No vacuous or tautological test found among the changed/added set.

## 4. Closing checklist

- **Original repro no longer reproduces.** `10-reproduce-and-minimize`'s
  shell-level harness showed the pre-fix trimmed capture ends `0x7d` and
  `git apply --check` fails with `error: corrupt patch at line 15`. The
  code-level equivalent, `retain_dirty_writes_a_patch_git_apply_accepts`,
  now passes against current code (real `capture_dirty_patch`, not a
  reimplementation) — confirmed above.
- **Regression test passes.** Confirmed above (both new tests, full
  `runtime::surface::tests`, `c11_injectable_git`, `m7_docker_executor`
  green).
- **Tagged debug instrumentation from `30-instrument` removed.**
  `grep -rn PROBE234 src/ tests/` returns nothing.
- **Throwaway prototypes deleted or marked.** `10-reproduce-and-minimize`'s
  shell-level harness was never committed to the source tree (it ran
  outside the crate, per that stage's own record); `git status --porcelain`
  shows no stray files from it.

## 5. Gates

- `cargo fmt --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test` (full suite) — 10 failures, all in `tests/m2_daemon_api.rs`,
  and they are the exact same 10 test names `40-fix-with-regression-test`
  already recorded and attributed to ambient `SGT_*` environment variable
  interference for a Sergeant-managed checkout of itself running inside
  its own harness (`docs/environments/macbook.md`):
  `concurrent_stale_replacement_leaves_the_surviving_daemon_discoverable`,
  `r_mvp1_7_sgt_turn_cap_env_var_reaches_a_real_spawned_daemon`,
  `resolve_data_dir_falls_back_through_sgt_data_dir_then_xdg_then_home`,
  `retry_success_prints_the_human_readable_line`,
  `stale_descriptor_is_replaced_but_ambiguous_descriptor_fails_closed`,
  `t7_cli_end_to_end_auto_spawn_and_second_daemon_fails_closed`,
  `t7b_cli_status_show_and_cancel_through_the_binary`,
  `t8_two_concurrent_auto_spawns_one_survivor_both_commands_complete`,
  `the_data_dir_guard_reaps_the_daemon_a_client_command_spawns`,
  `work_list_human_form_prints_the_empty_and_populated_branches`. All ten
  fail the same way — a spawned `sgt` daemon subprocess does not report
  healthy within 10s inside this harness — none touch `git.rs`, `surface.rs`,
  or any file this fix or `f464a05e` changed. Re-verified by name match
  against `40-fix-with-regression-test/output/fix.md`'s own recorded list;
  identical set, not a superset or a different failure signature.

No new blocker surfaced in the re-attack. Per `@@bounded-judgment` J0, a
new blocker surviving into the fix commits would require `needs_input`;
none did.

## 6. Root-cause postmortem

**What was actually wrong.** `capture_dirty_patch` (src/runtime/surface.rs)
ran `git diff --cached --binary` through the module's general-purpose
`git()` helper (src/runtime/git.rs), whose contract is "trimmed `String`" —
appropriate for the vast majority of `git()`'s ~50 call sites (rev-parse
output, branch names, single-line answers) where trailing whitespace is
noise. A `git diff` patch is not that kind of output: its trailing `\n` is
part of the format `git apply` parses, not incidental whitespace, and
`String::from_utf8_lossy`'s lossy re-encoding (a second, independent
defect, H2) silently corrupts any non-UTF-8 byte a binary or otherwise
non-ASCII diff might contain. Confirmed hypothesis, from
`20-hypothesize/output/hypotheses.md`: **H1** (the `.trim()` in `git()`
strips the diff's mandatory trailing newline) is the primary and fully
sufficient explanation for #234's reported symptom (corrupt patch, always
at end-of-file, last byte non-newline, 22/22 real-estate patches). **H2**
(lossy UTF-8 re-encoding corrupts non-UTF-8 diff content independently of
trimming) was promoted from "plausible, unexercised" to confirmed-real
during `30-instrument`'s P2 probe, and is a second, narrower defect in the
same call path — not the cause of #234's own reported symptom, but a
latent one the same fix (routing through `git_bytes` instead) happens to
close. Any future debugger should read #234's own symptom as H1-only, and
treat H2 as a second finding piggy-backed on the same root call, not
something #234's original report itself evidenced.

**How it got in.** `git()` is the default, path-of-least-resistance helper
for "run git, get a string" in this module, and `capture_dirty_patch` was
written to fit that pattern without the author registering that a patch's
byte stream, unlike every other `git()` caller's output, is a *format* an
external tool (`git apply`) parses structurally, where trailing bytes and
byte-exactness both matter. The bug is a category error at the call site
— reaching for the general helper instead of asking "what does the
consumer of this output actually require" — not a bug in `git()` itself,
which was and remains correct for what its other ~50 call sites need.

**What would have caught it.** A dedicated test that fed a captured
`.dirty.patch` to a real `git apply` — the actual consumer — rather than
asserting only that the patch's content looked plausible as a string. No
such test existed pre-fix; `retain_dirty_writes_a_patch_git_apply_accepts`
is exactly that missing seam, and it is the regression test committed at
`56927a14`. More generally: any output this codebase captures from an
external tool and *re-feeds to another external tool* (as opposed to
consuming it internally) is a class of call site worth a byte-exactness
test by default, not just a "does it contain the expected substring" test
— the substring can survive a trim or a lossy re-encoding; a downstream
parser's structural requirements often cannot.

## 7. Architectural recommendation

No architectural change is required to prevent recurrence of #234 itself
— `git_bytes` closes the gap with a normal function addition, not a
redesign. One adjacent observation, recorded here per this stage's
contract rather than acted on: `surface.rs:1980`'s `git status --porcelain`
call site still goes through `git()`, whose `.trim()` strips leading
whitespace that is significant for porcelain's status-column format (the
same *class* of defect — "a structured output format run through a helper
built for free-text answers" — as #234's root cause, though a different
symptom: leading-column loss, never fed to `git apply`, so not corrupt
patches). This was explicitly out of scope for this Work (named as a
"separate-issue candidate, not touched here" in `56927a14`'s commit
message and F-SF-05's refutation) and is not touched by this stage either,
per `@@fix-confirmed`'s scope. **Recommendation for Captain/the human**:
file a follow-up issue for `surface.rs:1980` to switch to `git_verbatim`
(already correct for leading-whitespace-significant, single-trailing-
newline-insignificant output) — no architectural work, just the same
category of one-line fix `git_bytes` was for #234, worth tracking so it
does not surface as its own confusing bug report later.

## 8. Coverage disclosure, carried forward

Per `@@panel`/`@@fan-out-evidence`, the same disclosure `50-panel` and
`55-refute` recorded applies to this stage's re-attack and test-honesty
audit too: this was one stage execution reading the fix commits directly
against ground truth, not four isolated seats and not a separate Work.
All four axes `50-panel` raised (spec-fidelity, invariants, simplicity,
test-honesty) were attacked in `55-refute`; none were degraded or missing
there, and this stage's re-attack re-read the same two axes most relevant
to code correctness (spec-fidelity-adjacent: does the fix still do what it
claims; test-honesty: do the tests still prove what they claim) directly
against current code rather than relying on `55-refute`'s prior read.

## 9. Closing packet — promoted artifacts

Every declared `promote` artifact from this package, named by path and
confirmed present in this Work's tree:

| artifact | stage | disposition | present |
|---|---|---|---|
| `.sergeant/workflows/fix-defect/40-fix-with-regression-test/output/fix.md` | 40 | promote | yes (commit `f494dc46`) |
| `.sergeant/workflows/fix-defect/50-panel/output/findings.md` | 50 | promote | yes (commit `4d711b78`) |
| `.sergeant/workflows/fix-defect/55-refute/output/findings.md` | 55 | promote | yes — present in the work surface at this stage's start but not yet committed to the Work branch; committed by this stage's own closing commit, alongside this file |
| `.sergeant/workflows/fix-defect/60-re-verify-and-postmortem/output/re-verify-and-postmortem.md` | 60 (this stage) | promote | yes (this file) |

The panel's own coverage record from `50-panel`, stated honestly: four
axes (spec-fidelity, invariants, simplicity, test-honesty), none missing —
carried forward unchanged from `50-panel/output/findings.md`'s own
"Coverage" section and re-confirmed by `55-refute`'s "All four axes
attacked; none degraded or missing."

## Disposition

All confirmed findings are fixed or recorded unfixed with a stated
reason. Both re-verify passes (re-attack, test-honesty audit) ran over
every fix commit and found nothing new. The closing checklist passes.
The postmortem states H1 as #234's correct hypothesis and H2 as a
piggy-backed second defect. One architectural-adjacent recommendation is
recorded for Captain/the human, not acted on. Every upstream `promote`
artifact is named and confirmed present. This stage is complete.
