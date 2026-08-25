# 15-validate — validation record

## Note on inputs

`../05-baseline/output/baseline.md` and `../10-implement/output/implementation.md`
were not present in this work surface — only their stage `README.md`
scaffolding exists, and the implementation (commit `f6088192`, "split-hardening
W3: atomic sgt-owned distro trees + route rewrite (#241, #261)") was already
committed on this branch before this stage ran. In the absence of a recorded
`05-baseline` test command, the validation command used here is the one the
task's own brief names explicitly as the decisive check: the new/changed
tests, `cargo test --test f_doctrine_skew` (the citation sweep, now extended
to cover `skills/` and `.sergeant/`), and one full `cargo test` + `cargo fmt
--check` + `cargo clippy -D warnings` pass. This satisfies this stage's J0
fallback (a command is named by the intent) rather than inventing one.

Revision validated: `f60881925e398c3a7e7d9fa457b1275f6b9765a0` (HEAD, branch
`sergeant/01M0VYHVKDH7FXJ3XDTQYYNT21`).

## `cargo test --test f_doctrine_skew` (citation sweep)

```
running 12 tests
test fix_confirmed_context_states_the_commit_imperative ... ok
test close_out_completion_boundary_covers_external_pipeline_runs ... ok
test agents_md_carries_the_intent_section ... ok
test the_proposals_canonical_manifest_example_parses_under_the_current_schema ... ok
test agents_md_states_the_ratified_mutation_surface_contract ... ok
test no_shipped_workflow_or_skill_quotes_the_removed_workspace_flag ... ok
test no_embedded_skill_or_workflow_file_cites_a_removed_or_workspace_only_path ... ok
test no_embedded_workflow_instructs_a_stage_actor_to_run_estate_scoped_commands_from_its_surface ... ok
test embedded_skills_carry_the_real_root_and_preflight_remedies ... ok
test no_readme_contributing_src_test_or_workflow_file_cites_a_removed_path ... ok
test agents_md_dash_c_flag_actually_names_the_estate ... ok
test agents_md_session_start_matches_the_real_root_gate ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.17s
```

PASS — the sweep (now scoped over `skills/` and `.sergeant/` per this
change) is green: no embedded corpus file cites a removed or
workspace-only path.

## `cargo test --workspace` (full suite)

Every suite in the workspace reported `test result: ok` with `0 failed`.
Aggregate: **1706 passed, 0 failed** across 42 test binaries (unit tests +
all integration test files + doctests), with a handful of intentionally
`ignored` measurement-only tests (dogfood-journal benchmarks gated behind
manual runs, unrelated to this change).

Full verbatim per-binary summary lines:

```
test result: ok. 971 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 19.53s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 81 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 5.15s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.27s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.42s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
test result: ok. 77 passed; 0 failed; 17 ignored; 0 measured; 0 filtered out; finished in 10.24s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.54s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.26s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.85s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.34s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.42s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.56s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.11s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.55s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.01s
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 17.28s
test result: ok. 100 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 6.95s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.18s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.14s
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 85.89s
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.15s
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.97s
test result: ok. 69 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out; finished in 3.91s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.92s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.19s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.84s
test result: ok. 0 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.06s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.28s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.83s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

No `FAILED`, no `error[`, no `panicked` lines anywhere in the full run log.

## `cargo fmt --all -- --check`

Exit code 0, no output (no formatting diffs).

## `cargo clippy --workspace --all-targets -- -D warnings`

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s
```

Exit code 0 — no warnings, `-D warnings` gate satisfied.

## Result

**PASS.** All four decisive checks — the citation sweep (`f_doctrine_skew`,
now covering `skills/` and `.sergeant/`), the full workspace test suite,
`cargo fmt --check`, and `cargo clippy -D warnings` — are green against
`f6088192` with no failures, no formatting diffs, and no clippy warnings.
