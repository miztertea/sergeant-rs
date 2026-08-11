# BS2 — Turn-completion settle + permission-mode profile config (#46, #47)

Bug Sprint 2, Cerberus, 2026-08-11. Governing evidence:
`docs/gauntlet/runs/runB/run-manifest.md` (both attempts),
`docs/gauntlet/notes/cerberus-ask-grammar-remeasurement-2026-08-11.md`,
issues #46/#47. Loop: full multi-axis per R-S0-12 at Bug Sprint scale
(builders → critics → refuter → independent prober → fixer → shipping gate).

## The measured defect (read before designing)

Run B, both attempts, stage `00-contract` sat `active` indefinitely after
its turn ended — 34m58s (attempt 1, envelope-less root refusal) and 45m28s
(attempt 2, clean envelope whose derivation the adapter's own unit test
pins as `StageCompleted`). Orchestrator's root-cause reading (verified in
source this session, 2026-08-11): the turn thread records
`TurnState::Finished` and journals `conversation.turn.ended` (+
`usage.updated`), but **nothing ever takes another observation** — the
engine observes only at launch-settle (when a real turn is still
`InFlight` → `Running` → parked), SEND-settle, recovery resume, and
client-request cranks. The fake backend finishes turns inside the launch
effect, so every deterministic test settles at launch and the gap is
structurally invisible to the current suite. `observe_in_memory` already
classifies the envelope-less case `native: Unknown` (→ engine blocks,
fail-closed) and the clean case `StageCompleted` — the classifications are
right and starved.

## Outcomes

1. **A turn that ends, settles.** When a real-backend turn finishes after
   launch-settle, the daemon takes the observation and drives the stage —
   completion cascades, envelope-less blocks — with **no client call
   required**. Bounds: observation taken and settled off the core lock
   (§22.6 instrumentation stays green; the performer/crank machinery is
   the existing home). Settle must be currency-checked (§14.5): a late or
   duplicate observation of a superseded execution/attempt is subordinate
   to durable state — it must not double-complete, must not resurrect a
   canceled work, and must not race a concurrent SEND/interrupt settle.
   Candidate lower rung (builder decides, rung-logged): a daemon-side
   completion driver over active executions — the adapter's `observe` is
   an in-memory read, so a modest poll cadence is not external I/O; an
   adapter→daemon completion notification is the higher rung and an R7
   choosing it names why polling failed. Recovery's restart path already
   re-observes; do not duplicate it — pin it (Outcome 3).
2. **Both Run B shapes proven end-to-end through a real daemon.** Contract
   tests drive the REAL turn-end path: a fake-CLI stub (S-series pattern)
   whose process outlives launch-settle, through the spawned daemon
   (DataDir guard), asserting journal + work state:
   - stub emits complete stream-json with a clean result envelope, then
     exits → stage completes, next stage enters, no client crank in the
     interval (the test may poll `sgt work show` for the *transition*, but
     the transition must not be caused by a mutating client call);
   - stub exits pre-envelope with stderr (attempt-1 shape) → stage lands
     `blocked` with stderr in evidence within the same no-client-crank
     bound. The classifier-in-isolation unit tests are not acceptance.
3. **Crash windows (L6/§22.5).** Daemon dies after
   `conversation.turn.ended` is journaled but before settle → restart
   re-derives the outcome (recovery resume observes `TurnState` is gone;
   it must land per its fail-closed ladder, never silently `active`
   forever). Pin the window with the existing crash-injection style. Any
   new adjacent-append pair the driver introduces gets the same analysis.
4. **#47 — permission mode is profile config.** `permission_mode` in the
   profile, passed through as the CLI's own vocabulary (`default` |
   `acceptEdits` | `bypassPermissions` | `dontAsk` | `plan`).
   Unspecified → **no permission flag at all** (the CLI's own default),
   never a silent `--dangerously-skip-permissions`. `bypassPermissions`
   is explicit profile opt-in. Doctor/capability surfaces the effective
   mode per profile. Unknown mode strings fail closed at profile load.
   Tests: flag-construction per mode (stub-level, deterministic) + ONE
   bounded opt-in real-CLI test pinning what a default-mode headless turn
   can and cannot do (measured, not assumed — L1; keep it to one haiku
   turn). Measured fact available: skip flag is viable under uid 1001
   (`docs/environments/cerberus.md`).
5. **a5 probe-gates** per the re-measurement note's disposition 2: the
   driven turn's transcript carries `post_turn_summary` → assert the
   NeedsInput mapping; carries none → assert the withdrawal fired
   (capability lowered + `conversation.turn.grammar_unmeasured`
   journaled) and skip the mapping assertion loudly (`SKIPPED-ENV`).

## Non-goals

No journal group commit (#44, next sprint). No Docker. No further ask
grammar work beyond Outcome 5. No retention. No TUI/dashboard surfaces
beyond what doctor already shows.

## Unknowns (state, don't fake)

- Whether a poll cadence interacts with the m6 #45 flake class (dropped
  daemon under load) — if the driver changes daemon shutdown ordering,
  say so.
- The CLI-default-mode headless consequence matrix (Outcome 4's opt-in
  test measures it; whatever it finds is the record).

## Gate

- `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` green; suite counts move only upward.
- Every fix commit separable (L10), pinned (L7), guard-mapped for the
  independent prober; `Fixes #46` / `Fixes #47` on exactly the commits
  that close them.
- Bracketed-pgrep leak check clean after suites.
- `scripts/gate.sh` run green before push (shipping gate).
