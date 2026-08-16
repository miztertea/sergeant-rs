# ADR 0012: Estate and Doctor are daemon API surface, not a second TUI reach path

**Status:** Accepted, 2026-08-16.

## Context

`reference/proposal-tui-t-series.md`'s Decision T2-14 originally proposed
that `tui.rs` consume repo/group and Doctor behavior through "narrow typed
local operations extracted from the current CLI implementation" — the CLI
and TUI both calling `crate::domain::manifest` and `mod doctor`'s
Check-producing functions directly, in-process, formatting the same
outcomes differently. The stated reason: "Repo/group commands are
deliberately local manifest operations. Doctor must work when no daemon
is running. Forcing them through new daemon routes solely for UI purity
would violate their existing lifecycle semantics."

T-SERIES-1's invariants axis (`docs/gauntlet/runs/t-series-1/critics/
invariants.md`, Finding `inv-estate-doctor-bypasses-client-boundary`,
confirmed unchanged by adversarial refutation) found this unenactable as
written: `docs/DEVELOPMENT.md`'s "clients are equal" invariant — "The CLI
and TUI reach state only through the loopback HTTP/SSE API via
`ApiClient`... enforced by tests, not convention" — is a hardened
structural gate (`tests/m6_surfaces.rs`'s `t5_the_tui_is_a_client_like_
any_other` and `t5b_the_structural_scan_sees_every_spelling_of_a_path`,
the latter added specifically because an earlier disposable-copy
experiment fooled a less careful version of the same scan). T2-14 as
drafted would add a second crate-internal reach path into `tui.rs` beyond
`crate::api`, without ever naming the test it collides with.

The critic's key finding: the proposal's own justification conflates two
different clients' constraints. Doctor's no-daemon requirement is real,
but it is the *CLI's* requirement (`sgt doctor` must diagnose an
installation with no daemon running at all), not the TUI's. Decision
T2-16, settled earlier in the same proposal, already commits `sgt tui` to
refusing outright without a live, reachable daemon (ADR 0009, no
exceptions). Nothing about the TUI's own behavior needs, or benefits
from, a no-daemon code path.

## Decision

Repo/group and Doctor behavior reaches `tui.rs` exclusively through new
authenticated daemon API routes, consumed via `ApiClient` — the same
boundary every other daemon-owned fact already crosses:

```text
GET    /v1/estate/repos
POST   /v1/estate/repos
DELETE /v1/estate/repos/{name}
GET    /v1/estate/groups
POST   /v1/estate/groups
DELETE /v1/estate/groups/{name}
GET    /v1/doctor
```

Each is a thin daemon-side wrapper over an already-existing function —
`crate::domain::manifest::{add_repo, remove_repo, add_group,
remove_group}` and `mod doctor`'s `Check`-producing functions plus
`Report::to_json` — never a second computation or a second validation
path. `src/api.rs`'s `doctor_report` handler is exactly `doctor::run(&state
.data_dir).await; Json(report.to_json())`: the daemon serializes the same
`Report` the CLI prints, per Decision T2-58.

The CLI's existing local, no-daemon calls to these same functions are
**unchanged** — `sgt repo add`, `sgt group add`, and `sgt doctor` keep
working with no daemon running at all, which `sgt init` and pre-daemon
diagnosis both depend on. This is a separate, deliberate exception scoped
to the CLI's own no-daemon requirement, not a second sanctioned reach
path for `tui.rs`. `tests/m6_surfaces.rs`'s `t5`/`t5b` are unweakened —
confirmed by every T-series Work built against this decision, and by this
ADR's own review of the merged diffs.

## Alternatives considered

**Ship T2-14 as originally drafted** (shared local functions). Rejected:
fails `t5`/`t5b` on the first `cargo test` run, or requires silently
weakening a hardened, purpose-built architecture test the proposal never
disclosed it was asking for.

**Revise `t5`/`t5b` to permit a second sanctioned reach path for
estate-local, non-daemon operations.** Considered and rejected. The
refuter (`docs/gauntlet/runs/t-series-1/refuters/invariants.md`) found a
2026-08-11 design note (`docs/gauntlet/notes/estate-manifest-design-2026-
08-11.md`, "three pens, one file") that appears to anticipate exactly
this shape — "TUI later = the same verbs with a screen" — but that note
predates `t5b`, added specifically to close a regression where an earlier,
less careful version of the scan was fooled by this exact pattern.
Reviving it would revert a fix already earned by a measured incident, for
a client (the TUI) that has no actual no-daemon requirement to justify
the exception.

## Consequences

- New daemon-owned mutation surface: the daemon process now writes
  `sergeant.toml` on a client's behalf (`POST`/`DELETE /v1/estate/repos`,
  `/v1/estate/groups`), which previously only the CLI did, synchronously,
  in-process. Not a new privilege — the daemon already has full
  filesystem access to the estate — but a new entry point to that write
  path, worth naming rather than treating as free.
- `src/cli.rs`'s `mod doctor` gained `pub(crate)` visibility (from
  private) so the new `GET /v1/doctor` handler in `src/api.rs` can call
  its `Check`-producing functions. No logic in `mod doctor` changed.
- Estate/Doctor parity tests (`tests/m6_surfaces.rs`, T3) prove the CLI's
  local path and the TUI's API path produce identical structured results
  against the same fixtures — the parity claim is pinned by test, not
  merely asserted.
- The equal-client boundary holds without exception for every T-series
  screen: Home, Fleet, Workflows, Estate, and the canonical Work surface
  all reach daemon-owned facts only through `crate::api`.

## Open questions

None. This ruling was made explicitly, by the proposal's owner, in
dialogue with the orchestrating session, after the two live options above
were laid out — not inferred or defaulted.
