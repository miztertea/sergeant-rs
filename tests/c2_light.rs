//! S2 V1c: consolidated harness for the smallest, cheapest C2 (non-spawning)
//! suites — ten single-purpose test files that each paid the full per-binary
//! link cost (and its own copy of `support`'s compile) for a handful of
//! tests. Every test remains addressable exactly as before, just through
//! this binary's name: `cargo test --test c2_light <old_suite_name>::<test>`
//! (nextest: `cargo nextest run -E 'test(<test>)'` or by the same
//! `c2_light::<old_suite_name>::` path). No test was deleted, weakened, or
//! reworded — each module below is its original file, moved verbatim except
//! for the `mod support;` → `use crate::support;` swap made necessary by the
//! move (this file is the crate root now, not each individual suite, so
//! `support` is declared once, here).
//!
//! Heavier C2/C3 suites (m4_backends, m3_execution, codex_backend,
//! opencode_backend, agy_backend, m2_daemon_api, m6_surfaces, ...) are
//! deliberately NOT folded in here: measurement (v1c baseline) showed the
//! full-workspace relink cost after a shared-source change is ~19s across
//! all 45 binaries — already small next to the ~5-9 minute test run itself —
//! so merging suites whose own compile time already dwarfs their link
//! overhead would add real intra-process hygiene risk for a link-time saving
//! that was never the bottleneck. This binary is the bounded, measured pilot
//! named in the v1c brief's "keeping heavyweight outliers separate if
//! measurement says the link win is already realized" clause.
mod support;

#[path = "c2_light/agy_routing.rs"]
mod agy_routing;
#[path = "c2_light/codex_routing.rs"]
mod codex_routing;
#[path = "c2_light/coverage_stage_membership.rs"]
mod coverage_stage_membership;
#[path = "c2_light/docs_contract.rs"]
mod docs_contract;
#[path = "c2_light/e_periodic_sweep.rs"]
mod e_periodic_sweep;
#[path = "c2_light/m10_harness.rs"]
mod m10_harness;
#[path = "c2_light/opencode_routing.rs"]
mod opencode_routing;
#[path = "c2_light/t2_workflow_catalog.rs"]
mod t2_workflow_catalog;
#[path = "c2_light/w2fix_probe_ordering.rs"]
mod w2fix_probe_ordering;
#[path = "c2_light/w5_cutover_rehearsal.rs"]
mod w5_cutover_rehearsal;
