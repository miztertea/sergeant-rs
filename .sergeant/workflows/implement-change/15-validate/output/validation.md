# Validation — split-hardening W5-fix2 (#262)

## Change under test

HEAD `bf537e1d` — "fix(estate): validate network_access at load, surface it
in doctor (#262)". Adds estate-load-time validation of `network_access`
mirroring `permission_mode`'s existing check (`src/domain/estate.rs`, same
two call sites, same `InvalidNetworkAccess`-shaped error), and a
`network_access` doctor row beside `permission_mode_check`
(`src/cli.rs`). Tests added in `src/domain/estate.rs` (unit) and
`tests/m6_surfaces.rs` (integration, doctor rows).

## Test command

Per the fix's stated gates:

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --lib domain::estate::
cargo test --lib cli::
cargo test --lib backend::codex::
cargo test --test m6_surfaces
cargo test --workspace
```

## Results (verbatim, condensed)

### `cargo fmt --check`
Exit 0. No output — nothing to reformat.

### `cargo clippy --all-targets -- -D warnings`
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.28s
```
Exit 0. Zero warnings.

### `cargo test --lib domain::estate::`
```
running 38 tests
...
test domain::estate::tests::a_profile_with_an_unknown_network_access_is_refused_at_load ... ok
...
test domain::estate::tests::a_profile_with_an_unknown_permission_mode_is_refused_at_load ... ok
...
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 898 filtered out; finished in 0.27s
```
The new `network_access`-at-load test passes alongside its
`permission_mode` sibling, confirming the mirrored shape.

### `cargo test --lib cli::`
```
running 13 tests
...
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 923 filtered out; finished in 0.04s
```

### `cargo test --lib backend::codex::`
```
running 66 tests
...
test result: ok. 66 passed; 0 failed; 0 ignored; 0 measured; 870 filtered out; finished in 0.02s
```
Confirms the codex argv-regression tests (stub-based) are untouched and
still passing, per the "do not remove" note.

### `cargo test --test m6_surfaces`
```
running 52 tests
...
test t3d_doctor_reports_permission_mode_has_no_effect_on_a_codex_profile ... ok
test t3e_doctor_reports_network_access_has_no_effect_on_a_claude_profile ... ok
test t3b_doctor_reports_the_effective_permission_mode_per_profile ... ok
test t3e_doctor_reports_the_effective_network_access_per_profile ... ok
...
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.38s
```
The new `network_access` doctor rows land beside the existing
`permission_mode` rows, matching the fix's stated goal.

### `cargo test --workspace`

First attempt (default parallel test-threads): 1 failure —
`t5_disabled_export_runs_no_exporter_machinery` in `tests/m5_projections.rs`
panicked with `bind a stand-in collector on 127.0.0.1:4318: Address already
in use (os error 98)`. Isolated re-run of that binary alone
(`cargo test --test m5_projections -- --test-threads=1`) passed all 21
tests in 103.5s, confirming this is a pre-existing port-collision race
between two OTLP-collector-binding tests running concurrently in the same
binary — unrelated to `network_access`/`permission_mode`/estate/doctor
code touched by this change.

Full clean re-run of `cargo test --workspace` (all binaries, default
settings):

```
test result: ok. 935 passed; 0 failed; 1 ignored; ... (lib)
...
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.31s   (m5_projections)
...
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.61s   (m6_surfaces)
...
```
Every test binary in the workspace reported `0 failed` on this run.

## Verdict

**Pass.** `cargo fmt --check` and `cargo clippy -D warnings` are clean.
Targeted estate/cli/codex/m6_surfaces tests pass, including the two new
tests this fix adds (`a_profile_with_an_unknown_network_access_is_refused_at_load`
in estate.rs, `t3e_doctor_reports_the_effective_network_access_per_profile`
in m6_surfaces.rs). The full `cargo test --workspace` pass is green;
one test in the run showed a flaky, load-dependent port-binding collision
in an unrelated OTLP export test (`m5_projections.rs`), reproduced as
pre-existing and independent of this change by isolating and re-running
that binary and by a subsequent clean full-suite run.
