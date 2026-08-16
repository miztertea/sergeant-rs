# T-series build sprint — plan

Authored 2026-08-16 (Cerberus) by the orchestrating session (Captain), owner
present and directing at the start, then delegated: *"You are the captain...
my expectation is to go watch a movie and return to a PR ready for my review
to merge to main."*

## 1. Destination

Land `sgt tui` as the work-centered operator cockpit `reference/
proposal-tui-t-series.md` describes, on `integration/t-series-2026-08-15`,
behind head PR #131, gated by one final `validate-and-ship` run — never
merged to `main` by this session (owner's explicit instruction).

## 2. How this plan is reviewed

Per the PATH-TO-MAC-1 precedent (`docs/gauntlet/runs/path-to-mac-2026-08-15/
plan.md` §9): a multi-Work implementation sprint's *plan* gets graded, not
each individual Work. That grading already happened here — **T-SERIES-1's
enactability axis graded `reference/proposal-tui-t-series.md` §20's T0-T4
program shape directly** (`docs/gauntlet/runs/t-series-1/critics/
enactability.md`, `refuters/enactability.md`), found and corrected the phase-
assignment gap (4 tabs unassigned — now fixed), the ambiguous T0 verbs (fixed),
and the unowned #120 gate dependency (fixed, tracked below). This plan
document is the dispatch mechanics for an already-graded program, not a new
artifact requiring its own panel.

## 3. Scope, this sprint

Built here: T1 (cockpit foundation), T2 (workflow discovery), T3 (Estate),
the pre-T4 fix for #120, T4 (close-out), one final gate.

Not re-litigated: the T2-14 client-boundary ruling (owner-resolved,
2026-08-16, already merged), the proposal's own content (T0 already closed).

## 4. Waves

Sequential unless marked `‖` (parallel). Each wave cuts from the *previous
wave's* integration-branch tip, per `src/runtime/surface.rs`'s cut-from-HEAD
behavior (same reasoning as PATH-TO-MAC-1 §5) — advancing the branch between
waves means later Works never inherit a stale base.

| Wave | Work | Scope | Cut from |
|---|---|---|---|
| 1 | **T1a · shell** | Restructure `src/tui.rs` (2,614 lines, currently flat) into a module tree; `Home / Fleet / Workflows / Estate` navigation; App state machine; §8.10's locked visual tokens wired in | T0 tip |
| 2 | **T1b · canonical Work surface** | Thread, Workflow rail, Evidence, Graph, Details tabs — read-only views over already-shipped API reads | T1a tip |
| 3 | **T1c · mutation & composer** | Output/envelope/`completed_dirty`, respond/retry/extend/cancel, `ratatui-textarea` composer (T0-admitted), responsive Fleet | T1b tip |
| 4 ‖ | **T2 · workflow discovery** | `GET /v1/workflows` route (§11.2's T0-finalized schema) + `ApiClient` method + Home's `@` chooser + live-vs-pinned rail | T1c tip |
| 4 ‖ | **T3 · estate** | The 7 estate/doctor routes from the T2-14 ruling + Estate/Health screens + retained/reap consumption | T1c tip |
| 5 | **T3-fix · #120** | `no-mistakes`'s empty-diff-base false-green (blocks T4's own acceptance item 57) | merge(T2, T3) tip |
| 6 | **T4 · close-out** | Responsive fixtures, geometry tests, real screenshots, README/help, ledger/lessons/ADR, handoff note | T3-fix tip |
| 7 | **Gate** | `validate-and-ship`, whole branch, Captain-run | integration tip after T4 merges |

Wave 4's two lanes are disjoint by construction: T2 touches `src/api.rs`
(new route) and the Workflows/Home surfaces T1a already scaffolded; T3
touches `src/api.rs` (different new routes) and the Estate surface T1a
already scaffolded. Both append to `src/api.rs`'s route table — a small,
mechanical merge-order risk (not a semantic conflict), accepted rather than
serialized, consistent with PATH-TO-MAC-1's own W2/W3 precedent of accepting
disjoint-but-same-file parallelism.

## 5. Dispatch conventions

`--repo sergeant-rs --turns 40 --ceiling-secs 5400` — the same generous
envelope PATH-TO-MAC-1 used, because this is real implementation work, not a
proposal-editing pass (those ran fine at 20-24 turns; building and testing a
new TUI screen does not).

Every brief: cites the exact proposal sections it implements; states prior
art as settled (T0's locked tokens/schema, the T2-14 ruling, what already
ships) rather than re-deriving it; instructs `cargo fmt --check && cargo
clippy --all-targets -- -D warnings && cargo test` before considering the
Work done, per `docs/DEVELOPMENT.md`; names the exact files in scope and
forbids touching anything else; uses `tdd` (vertical slices, one seam at a
time) for anything net-new, `direct-implementation`-shaped instruction for
route/schema wiring that mirrors an existing pattern exactly.

`workflow`: `tdd` for T1a/T1b/T1c/T2/T3 (net-new, judgment-bearing feature
work); default (`software-change`) for the #120 fix (`diagnose-bug`-shaped
but small) and T4's doc-only bullets Captain doesn't do directly.

## 6. Orchestrator (Captain) duties

- Watcher armed before each dispatch, not after.
- **No "completed" believed on its own**: verify `git diff <base>..sergeant/
  <id> --stat` is non-empty and touches only the files the brief scoped,
  before merging. Read the actual diff, not just the Work's own summary.
- `cargo build`/`cargo test` re-verified by Captain after each merge into the
  integration branch, not just trusted from the Work's internal claim.
- **A wave Work landing `failed`:** retry once against the same base; a
  second failure blocks the wave and is a genuine escalation to the owner —
  not silently worked around.
- No merge to `main`. Every Work's branch merges into `integration/
  t-series-2026-08-15` via its own sub-PR against head PR #131, exactly as
  every prior stage of this sprint has.

## 7. Risks, named rather than smoothed

1. **Review is Captain-serial**, same limitation PATH-TO-MAC-1 named for
   itself — no per-Work critic panel, only diff-read verification plus one
   final whole-branch gate. Accepted for the same reason: the *program*
   already went through full adversarial grading (T-SERIES-1); the Works
   executing it are implementing an already-validated design, not making
   new architectural calls.
2. **`src/tui.rs`'s restructuring (T1a) is the highest-leverage, highest-risk
   Work in this sprint** — everything else depends on the module shape it
   picks. If T1a's own diff looks structurally wrong on Captain review, the
   right move is to retry it with a sharper brief, not to let T1b start
   against a shaky foundation.
3. **`m6_surfaces.rs` t5/t5b must still pass after every wave** — the
   client-boundary invariant (T2-14's ruling) is exactly what these tests
   guard; any Work whose diff adds a `tui.rs` import outside `crate::api`
   fails closed at `cargo test`, which Captain re-runs post-merge per §6.
4. **#120 (shipping-gate false-green) is the one dependency this sprint does
   not control the timeline of** — wave 5 exists specifically so T4's gate
   run (wave 7) is trustworthy rather than a skipped false pass.
