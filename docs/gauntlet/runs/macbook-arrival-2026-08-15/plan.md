# MacBook arrival — closeout sprint plan

Authored 2026-08-15 (on the MacBook Pro M3 Pro itself) by the orchestrating
session (Captain), after the owner ruled the session's own prior conduct
this trip **draft evidence, not delivered process** — no plan was filed, no
workflow was named, no dispatched Work tracked the effort, and the shipping
gate was hand-driven in direct contradiction of ADR 0005. This plan exists
to correct that: everything below is dispatched through named workflows via
`sgt run`, reviewed by a four-axis panel before any dispatch, and grouped/
ordered by dependency rather than executed as one undifferentiated pass.

Marks used below: **[measured]** = measured this session; a citation = read
from that artifact; anything else states it is a belief (**L15**).

---

## 1. Destination

Close out what the MacBook arrival trip (`docs/handoff/path-to-mac.md`) left
open, **through the process this repo actually documents**: a plan, reviewed
by a blind four-axis panel, then dispatched as `sgt` Work items on named
workflows, landing on `integration/macbook-arrival-2026-08-15`, gated by a
properly dispatched `validate-and-ship` Work (ADR 0005) — not a hand-run
`scripts/gate.sh` — and ending in a PR body whose provenance is this plan,
its adjudication, and the Work IDs that executed it, ready for the owner's
merge to `main`.

## 2. Owner rulings

Established this conversation, not invented by the orchestrator:

| # | Ruling | Consequence |
|---|---|---|
| R1 | Only the owner merges to `main`; the orchestrator merges freely against its own integration branch | PR #126 stays the head PR; no direct push to `main` |
| R2 | Gating is a dispatched Work (ADR 0005) — `scripts/gate.sh`/`no-mistakes` is never hand-driven by the orchestrating session again | The gate is Wave 2's own dispatched `validate-and-ship` Work, not a Bash invocation |
| R3 | "Task tool" means the harness's own built-in task/todo tracker, not `sgt run` and not a generic subagent | Withheld by default for this session's model (Sonnet 5) per Claude Code's own docs (`Task tool availability`); needs `CLAUDE_CODE_ENABLE_TODO_TOOLS=1` at next restart, which is a settings change only the owner makes. Tracked here as a plain-text checklist meanwhile |
| R4 | Prior in-session work (macbook.md's measurement, the #18/#81/#82/#95/#127 fixes already committed to `integration/macbook-arrival-2026-08-15`, PR #126) is **evidence**, independently re-verified against live git/GitHub state — not a substitute for this process | Cited as settled fact in the Works below; not redone from scratch, and not treated as needing this panel's re-approval since it is not this plan's output |
| R5 | This plan itself is the thing under review, not the code it will produce | Panel grades `plan.md`, per the same shape as `PATH-TO-MAC-1` |

## 3. Scope

**In scope** — the trip's three genuinely open findings, all inside
`sergeant-rs` itself (confirmed by reading each issue in full, not its
title):

| Issue | File(s) | Shape |
|---|---|---|
| #130 | `scripts/gate.sh` | `daemon_env_ok()`'s `/proc/<pid>/environ` grep has no macOS equivalent; the issue's own body sketches a candidate fix (skip the check on the direct-fork/non-systemd path, keep it where a supervisor could silently drop env) |
| #129 | `src/runtime/recovery.rs`, `src/runtime/engine.rs`, `tests/m3_execution.rs` | Deterministic extra `OBSERVE` in restart reconciliation (3 vs. expected 2), pure engine logic, no OS process involved — genuine engine-logic debugging |
| #128 | `tests/m2_daemon_api.rs`, submit-path code (`src/runtime/engine.rs`/`surface.rs`), `docs/perf/` | Submission throughput floor measures ~5 works/s vs. a 12 works/s floor on this hardware — real profiling needed, not a guessed fix; may end in a code fix, a justified floor revision, or neither if the cause resists this pass |

**Then, once all three land:** a dispatched `validate-and-ship` gate Work,
landing on `integration/macbook-arrival-2026-08-15`, followed by the
orchestrator updating PR #126's body to reflect this plan, its adjudication,
and the gate's outcome.

**Explicitly out of scope:** re-deriving or re-gating #18/#81/#82/#95/#127
(R4); rebuilding `no-mistakes`' stages as native ICM review (ADR 0005's own
deferred item); anything on `docs/environments/macbook.md`'s own probe-tooling
defect (noted there, not this sprint's job).

## 4. Prerequisite — the estate has never been set up on this host

**[measured]** `sgt doctor` on this checkout reports `0 repositories
declared`, and `sgt status`/the journal show **nothing has ever run in this
data dir** — this Mac has never had a working self-hosted estate for
sergeant-rs to dispatch Work against. `sgt repo add` clones a **fresh** copy
of `--origin` into `repos/<name>`; it does not adopt this primary checkout in
place.

**Wave 0 (orchestrator, not a dispatched Work — CLI scaffolding per
`AGENTS.md`'s "When NOT to use `sgt`" and the `estate-navigation` skill):**

```sh
sgt repo add sergeant-rs --origin https://github.com/miztertea/sergeant-rs.git
git -C repos/sergeant-rs fetch origin integration/macbook-arrival-2026-08-15
git -C repos/sergeant-rs checkout integration/macbook-arrival-2026-08-15
```

Every Work below cuts its own branch from the **estate clone's** current
HEAD (`src/runtime/surface.rs:332,431-441`, cited in the prior sprint's own
plan) — so the estate clone must sit on `integration/macbook-arrival-2026-08-15`
before Wave 1 dispatches, or Works would cut from `main` and never see R4's
prior fixes.

## 5. Waves

| Wave | Work | Scope | Cut from | Depends on |
|---|---|---|---|---|
| 0 | *(orchestrator, not dispatched)* | Register `sergeant-rs`, point estate clone at the integration branch | — | — |
| 1 ‖ | **WA · gate portability** | #130: fix or narrow `daemon_env_ok()` for macOS | estate-clone tip (Wave 0) | Wave 0 |
| 1 ‖ | **WB · restart-reconciliation bug** | #129: root-cause and fix the extra `OBSERVE` | estate-clone tip (Wave 0) | Wave 0 |
| 1 ‖ | **WC · submission-throughput profiling** | #128: profile the submit path on this hardware; fix if a clear culprit is found, otherwise document and, only with measured justification, propose a floor revision | estate-clone tip (Wave 0) | Wave 0 |
| 2 | **WD · gate** | `validate-and-ship`, dispatched (ADR 0005) | Wave-1 tip (all three merged) | WA, WB, WC all `completed` and merged to the integration branch |
| 3 | *(orchestrator)* | Adjudicate WD's findings; update PR #126's body with full provenance | — | WD |

WA/WB/WC touch disjoint file sets (`scripts/gate.sh` · `src/runtime/{recovery,engine}.rs` + `tests/m3_execution.rs` · `tests/m2_daemon_api.rs` + `src/runtime/{engine,surface}.rs` + `docs/perf/`) — each Work gets its own isolated worktree by construction (`sgt`'s one-Work-one-surface model), so parallel dispatch is safe; residual risk is an ordinary merge conflict if WC's profiling touches the same submit-path lines WB's fix does, flagged in §8.

## 6. Dispatch

`--repo sergeant-rs --profile sonnet --turns 40 --ceiling-secs 5400`
(matching the prior sprint's envelope sizing precedent — no Work in that
sprint used more than 2 turns against a well-scoped brief, so 40 is
deliberate slack, not an estimate). No `--backend` flag: `sergeant.toml`
should already declare `default_backend` per the prior sprint's §13
correction — **[to verify at Wave 0]**, since this is a fresh estate on a
fresh host and that setting lives in `sergeant.toml`, not in the code.

Each brief states: the issue's full body (not just its title) as **settled**
prior art; the exact acceptance test (the issue's own repro command where one
exists); a `Fixes #NN` trailer requirement, argued for, not assumed; evidence-
vs-belief labels (**L15**); and — for WB specifically, since it touches
`src/runtime/` — the adjacent-append crash-window check (**L6**,
`docs/DEVELOPMENT.md:41`).

`--workflow implement` for WA/WB/WC (bounded, single-outcome code fixes with
a named acceptance test each); `--workflow validate-and-ship` for WD.

## 7. Orchestrator duties

- Watchers armed in the same response as each dispatch, backgrounded
  (`sgt --json watch <id>` / `--follow` for the parallel Wave 1 trio).
- No `completed` believed on its own: verify `git log base..sergeant/<id>`
  is non-empty and the named acceptance test actually passes, independently,
  the same discipline the prior sprint's #94 finding demands.
- A Wave-1 Work landing `failed`: retry once against the same base; a second
  failure blocks Wave 2 and escalates to the owner rather than silently
  dropping that issue's scope.
- WD's findings: `auto-fix` on the Work's own judgment for mechanical/
  low-risk items, `no-op` recorded, `ask-user` relayed to the owner verbatim
  and never resolved autonomously (ADR 0005, part of D1).
- No merge to `main` (R1).

## 8. Risks

1. **WC (#128) may not resolve to a fix at all.** The issue's own text says
   profiling, not a guessed fix, is the actual scope — a Work here may
   legitimately end in "documented, not fixed," and that is an acceptable
   outcome per R-S0-1-style honesty, not a failed Work.
2. **WD depends on all of WA/WB/WC**, so a stuck Wave-1 Work blocks the gate
   entirely — mitigated by the retry-once-then-escalate rule in §7, not by
   loosening the dependency.
3. **The estate clone is genuinely new** (§4) — first contact with `sgt repo
   add`/dispatch on this exact host carries the same "anticipate
   unanticipated findings" risk the arrival trip itself named; anything this
   plan didn't foresee gets recorded, not silently routed around.
4. **WC/WB file overlap risk** (§5) — real but small; a merge conflict here
   is an ordinary git event, not a process failure, and is handled at
   Wave-2 cut time.
5. **`no-mistakes` inside WD may still hit #130's exact failure** if WA's fix
   is incomplete — WD's brief should treat a repeat of #130's failure mode as
   a `WA regression` finding to route back, not a WD-owned bug.

## 9. How this plan was reviewed

*Filled in after the panel runs — see `critics/`, `refuters/`, and
`adjudication.md` in this same directory.*

## 10. What this sprint closes

- #130, #129, #128 — closed by fix, or (for #128 specifically) closed by a
  measured, justified disposition if a fix does not materialize.
- A gate Work's outcome recorded against `integration/macbook-arrival-2026-08-15`,
  dispatched per ADR 0005, not hand-driven.
- PR #126's body rewritten to cite this plan, its adjudication, and every
  Work ID that executed it — the "proper PR body" that only exists once the
  process that produced it is real.
