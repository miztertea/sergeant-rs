# 35-re-verify — network_access load-validation fix (#262)

## Subject

Fix commits from `30-fix-confirmed` (fixes.md was not materialized in this
run; the fix's own commit trailer names the finding, so the subject is
taken directly from history):

- `bf537e1d` — `fix(estate): validate network_access at load, surface it in doctor (#262)`
  (`src/domain/estate.rs`, `src/cli.rs`, `tests/m6_surfaces.rs`)
- `1c4eaf5d` — `15-validate: record gate run for network_access load-validation fix (#262)`
  (evidence only, no code)
- `3033091f` — purges the removed workspace-selection flag from #262
  validation evidence (evidence wording only, no code)

Only `bf537e1d` touches code; the re-attack and test-honesty audit below
are against that commit.

## Pass 1 — re-attack for fixer-introduced defects

Read the full diff of `bf537e1d` against `src/domain/estate.rs`,
`src/cli.rs`, and `src/domain/profile.rs`'s pre-existing `network_access()`.

- `EstateError::InvalidNetworkAccess` is structurally identical to the
  established `InvalidPermissionMode` (same fields, same `#[error]` shape)
  and is raised from the same two call sites — `from_config_impl` and
  `from_config_impl_structural` — that `InvalidPermissionMode` already
  uses. No third loader (`declared_repos`, `from_config_structural`,
  `from_config_allow_empty`) needed a new call site: they all funnel
  through one of the two `_impl` functions.
- `network_access_check` (`src/cli.rs`) mirrors `permission_mode_check`
  line for line: estate-root threading, the `profiles.is_empty()` early
  return, the "configured but the backend doesn't read it" branch worded
  so it never reads as "in effect" (checked against the actual wording —
  distinct from the "in effect" branch's format string, same discipline
  `permission_mode_check` already applies).
- `backend_consumes_network_access` correctly inverts
  `backend_consumes_permission_mode`'s answer (`codex` reads
  `network_access`, `claude` reads `permission_mode`) — matches
  `codex.rs`'s `launch_config` (`network_access` field) and `claude.rs`
  having no equivalent sandbox knob.
- The one inefficiency found: `network_access_check` re-parses the
  manifest via a second `Estate::from_config_allow_empty` call, on top of
  the one `permission_mode_check` already made. Not a correctness defect
  (both reads are of the same immutable file within a single `doctor`
  invocation) and out of this fix's declared scope ("touch nothing
  else") — recorded here, not filed as a finding.
- No new `unwrap`/`expect`/panic path introduced; `p.network_access().ok().flatten()`
  in the doctor check is documented and correct — the value is provably
  `Ok` at that point because a `Err` would have already refused the
  estate at load, per the very validation this fix adds.

**Result: no new finding.** The fix's shape is a faithful mirror of the
existing, already-reviewed `permission_mode` path; it doesn't introduce
new call shapes, new panics, or new inconsistency between "configured"
and "in effect."

## Pass 2 — test-honesty audit

Three tests were added or changed by `bf537e1d`.

### `a_profile_with_an_unknown_network_access_is_refused_at_load` (`src/domain/estate.rs`)

Asserts a bad `network_access` value is refused at `Estate::from_config`
with `EstateError::InvalidNetworkAccess`, and that valid values still
parse. This test cannot be applied to pre-fix code unmodified — it
names `EstateError::InvalidNetworkAccess`, a variant the fix itself
introduces, so pre-fix code fails to *compile*, not just to pass. That is
still real evidence, of a stronger kind than a runtime assertion failure:
the test is mechanically incapable of passing without the exact new
variant existing and being returned from the exact code path exercised.
Not independently re-run against pre-fix code (compilation is the
proof); confirmed by reading the diff that the variant did not exist
before this commit (`git show bf537e1d -- src/domain/estate.rs` shows it
added, not modified).

### `t3e_doctor_reports_the_effective_network_access_per_profile` (`tests/m6_surfaces.rs`)

Asserts `sgt doctor --json` reports a `network_access` check naming
every profile and its effective value. Re-run against pre-fix code
(worktree checked out at `9ded5dfa`, the fix's parent, with only the
`tests/m6_surfaces.rs` hunk from `bf537e1d` applied — no production
code from the fix): **fails**, because `named_check(&report,
"network_access")` finds no such row in the pre-fix doctor output.
Confirmed empirically, not assumed.

### `t3e_doctor_reports_network_access_has_no_effect_on_a_claude_profile` (`tests/m6_surfaces.rs`)

Same treatment, same result: **fails** against pre-fix code for the same
reason (no `network_access` row exists yet).

Empirical run against `9ded5dfa` + test-only diff:

```
test t3e_doctor_reports_network_access_has_no_effect_on_a_claude_profile ... FAILED
test t3e_doctor_reports_the_effective_network_access_per_profile ... FAILED
test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 49 filtered out
```

(The one pass in that run is an unrelated pre-existing doctor test in the
same file, filtered in by the `t3e_doctor` substring match.)

**Result: all three tests are honest.** Two fail at runtime against
pre-fix code by direct re-run; the third fails to compile against
pre-fix code because it names a symbol the fix adds, which is a stronger
(not weaker) form of the same guarantee.

## Independent gate re-run

Re-ran the fix's stated gates myself against the current branch tip
(`3033091f`), rather than trusting the recorded `15-validate` evidence:

- `cargo fmt --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test --lib estate::` — 51 passed.
- `cargo test --lib cli::` — 13 passed.
- `cargo test --test m6_surfaces network_access` — 2 passed
  (`t3e_doctor_reports_network_access_has_no_effect_on_a_claude_profile`,
  `t3e_doctor_reports_the_effective_network_access_per_profile`).
- `cargo test` (full suite) — all green, 0 failed across every test
  binary.

## Outcome

No new finding. This is a positive result: the fix was re-attacked and
its new tests were audited for honesty rather than taken on the fixer's
word, and both passes came back clean. No `blocker` was found; no
`needs_input` escalation is warranted.
