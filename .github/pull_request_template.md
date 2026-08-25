# What & why

<!-- What changes and why it's the right shape. If this resolves a design
     decision, name its Ponytail rung (R1–R7). -->

## Evidence

<!-- The gates, run locally, with outcomes:
     cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
     Paste the test-result line(s). For behavior changes: what you ran or
     measured that shows the new behavior. Claims without tool output are
     prose. -->

## Tests

<!-- Every fix ships with the test that fails when the fix is reverted (L7).
     Name the pinning test(s) here, or say why none is possible. New
     backend capability flags need a contract test (L8). -->

## Record

<!-- Tick what applies; the ledger is append-only.
- [ ] No deviation from the proposal, or the deviation is registered in the workspace knowledge library's deviation register (D-row)
- [ ] Deferred findings landed as backlog rows or GitHub issues, not silence
- [ ] After the suites: `pgrep -f "debug/sgt [-]-data-dir"` finds nothing
-->
