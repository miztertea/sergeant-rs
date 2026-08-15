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

**[v2] Revised in place, same session, before panel dispatch.** §3/§5/§6/§8/§10
corrected: #130 is dropped as a **dependency of Wave 2**. Verified by reading
`.sergeant/workflows/validate-and-ship`'s actual stage files
(`20-select-intent-transport`, `30-start-run`, `40-drive-gates`) — none of
them reference `scripts/gate.sh`; every one invokes `no-mistakes axi run`/
`axi respond`/`axi status` directly. #130 is a bug in the hand-driven
convenience wrapper ADR 0005 already retired, not in the dispatched gate
Work's actual code path, so it was never a real dependency for Wave 2. v1 put
it there because the *hand-driven* attempt earlier this session hit it —
that attempt itself was the process violation this plan exists to correct,
and its symptom got carried into the plan without re-checking against the
correct path.

**[v3] Revised again, same session, before panel dispatch.** The owner
restated the goal as "resolve any mac-specific issues to close this PR" — #130
is a genuine mac-specific finding from this trip, and dropping it from the
gate's dependency graph is not the same claim as dropping it from this
sprint's scope. Restored as **WA**, disjoint-file, non-blocking, parallel to
WB/WC (§5) — in scope for this PR, not a Wave-2 gate dependency. Also added:
**R6**, validate-and-ship is never run with any skip/reduced-profile flag —
every stage runs in full, every time, no exceptions (§6, §7).

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
| R6 | `validate-and-ship` is never deferred, skipped, or run at a reduced profile — every stage runs in full, every time | WD's brief and dispatch carry no `--skip`/reduced-profile flag of any kind; if `no-mistakes` itself offers a "medium profile skips review/document stages" option (seen referenced in `20-select-intent-transport`'s own citations), WD's brief explicitly refuses it |

## 3. Scope

**In scope** — the trip's three genuinely open mac-specific findings, all
inside `sergeant-rs` itself (confirmed by reading each issue in full, not
its title). Only two of the three sit in the dispatched gate's own code
path; the third (#130) is scoped in for the PR without being a gate
dependency — see the per-issue notes below:

| Issue | File(s) | Shape |
|---|---|---|
| #130 | `scripts/gate.sh` | `daemon_env_ok()`'s `/proc/<pid>/environ` grep has no macOS equivalent; the issue's own body sketches a candidate fix (skip the check on the direct-fork/non-systemd path, keep it where a supervisor could silently drop env). **[v3] In scope, not a gate dependency** — see below |
| #129 | `src/runtime/recovery.rs`, `src/runtime/engine.rs`, `tests/m3_execution.rs` | Deterministic extra `OBSERVE` in restart reconciliation (3 vs. expected 2), pure engine logic, no OS process involved — genuine engine-logic debugging |
| #128 | `tests/m2_daemon_api.rs`, submit-path code (`src/runtime/engine.rs`/`surface.rs`), `docs/perf/` | Submission throughput floor measures ~5 works/s vs. a 12 works/s floor on this hardware — real profiling needed, not a guessed fix; may end in a code fix, a justified floor revision, or neither if the cause resists this pass |

**[v3]** #130 is in scope for this PR (a genuine mac-specific finding from
this trip, and the owner's restated goal is "resolve any mac-specific issues
to close this PR") but is **not a Wave-2 dependency** — `validate-and-ship`
never invokes the script #130 lives in (§v2 note above), so nothing about
WD needs WA to land first. It runs in Wave 1, parallel to WB/WC, on its own
disjoint file.

**Then, once WB and WC land** (WA does not gate this): a dispatched
`validate-and-ship` gate Work, landing on `integration/macbook-arrival-2026-08-15`,
followed by the orchestrator updating PR #126's body to reflect this plan,
its adjudication, and the gate's outcome.

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
`AGENTS.md`'s "When NOT to use `sgt`" and the `estate-navigation` skill).
[v2] Already run this session:**

```sh
sgt repo add sergeant-rs --origin https://github.com/miztertea/sergeant-rs.git
git -C repos/sergeant-rs fetch origin integration/macbook-arrival-2026-08-15
git -C repos/sergeant-rs checkout integration/macbook-arrival-2026-08-15
```

**[measured]** `sgt repo list` now shows `sergeant-rs` declared, `sgt doctor`
reports `estate: 1 repositories declared, all present on disk`; the estate
clone's `git log -1` is `471963a` (pre-plan tip) as of Wave 0 — it must be
re-synced to this plan's own commit before Wave 1 dispatches, since the plan
and contract files themselves only exist on the integration branch as of a
later commit than Wave 0 fetched.

Every Work below cuts its own branch from the **estate clone's** current
HEAD (`src/runtime/surface.rs:332,431-441`, cited in the prior sprint's own
plan) — so the estate clone must sit on `integration/macbook-arrival-2026-08-15`
before Wave 1 dispatches, or Works would cut from `main` and never see R4's
prior fixes.

## 5. Waves

| Wave | Work | Scope | Cut from | Depends on |
|---|---|---|---|---|
| 0 | *(orchestrator, not dispatched)* | Register `sergeant-rs`, point estate clone at the integration branch | — | — (**[v2] done**) |
| 1 ‖ | **WA · gate-script portability** | #130: fix or narrow `daemon_env_ok()` for macOS | estate-clone tip, re-synced to this plan's own commit | Wave 0 — **not** a Wave-2 dependency |
| 1 ‖ | **WB · restart-reconciliation bug** | #129: root-cause and fix the extra `OBSERVE` | estate-clone tip, re-synced to this plan's own commit | Wave 0 |
| 1 ‖ | **WC · submission-throughput profiling** | #128: profile the submit path on this hardware; fix if a clear culprit is found, otherwise document and, only with measured justification, propose a floor revision | estate-clone tip, re-synced to this plan's own commit | Wave 0 |
| 2 | **WD · gate** | `validate-and-ship`, dispatched (ADR 0005), full pipeline, no skipped stages (R6) | Wave-1 tip (WB+WC merged; WA merged too if already landed, but not waited on) | WB, WC both `completed` and merged to the integration branch |
| 3 | *(orchestrator)* | Adjudicate WD's findings; update PR #126's body with full provenance | — | WD |

WA/WB/WC touch disjoint file sets (`scripts/gate.sh` ·
`src/runtime/{recovery,engine}.rs` + `tests/m3_execution.rs` ·
`tests/m2_daemon_api.rs` + `src/runtime/{engine,surface}.rs` + `docs/perf/`)
— each Work gets its own isolated worktree by construction (`sgt`'s
one-Work-one-surface model), so parallel dispatch is safe; residual risk is
an ordinary merge conflict if WC's profiling touches the same submit-path
lines WB's fix does, flagged in §8.

## 6. Dispatch

`--repo sergeant-rs --backend claude --turns 40 --ceiling-secs 5400`
(matching the prior sprint's envelope sizing precedent — no Work in that
sprint used more than 2 turns against a well-scoped brief, so 40 is
deliberate slack, not an estimate). **[v2] `--profile sonnet` dropped and
`--backend claude` added instead** — tried at Wave 0 and refused: `422: no
profile named "sonnet" in this workspace (has: )`. This estate's
`sergeant.toml` (§4) has no `[profile.*]` sections and no `default_backend`
at all — it is a fresh scaffold, not a copy of the prior sprint's manifest —
so an explicit `--backend claude` on every dispatch is the correct
replacement, not a manifest edit invented to route around the refusal.

Each brief states: the issue's full body (not just its title) as **settled**
prior art; the exact acceptance test (the issue's own repro command where one
exists); a `Fixes #NN` trailer requirement, argued for, not assumed; evidence-
vs-belief labels (**L15**); and — for WA/WB specifically, since they touch
`scripts/` and `src/runtime/` respectively — WA gets the bash-3.2-clean
constraint (`docs/handoff/path-to-mac.md` step 8) and WB gets the
adjacent-append crash-window check (**L6**, `docs/DEVELOPMENT.md:41`). WC's
brief states explicitly that "profiled, no safe fix found, documented
instead" is an acceptable stage-30 outcome, not a failed Work — its own
issue text says so.

`--workflow implement` for WA/WB/WC (bounded, single-outcome code fixes,
each with a named acceptance test — `implement`'s `10-implement-with-tdd`
stage delegates to `tdd`, `30-review` delegates to `code-review`);
`--workflow validate-and-ship` for WD, **with no `--skip`/reduced-profile
flag of any kind** (R6) — the full seven-stage pipeline runs every time.

**Dispatch mechanics, so this doesn't get bungled on the actual command
line:** each brief is a separate Markdown file written to this session's
scratchpad directory first (not inlined in the shell command — briefs are
long, and issue text/code citations can contain backticks and `$(...)`-like
sequences that would otherwise be reinterpreted by the shell if typed
inline). Each Work dispatches as:

```sh
sgt run "$(cat /path/to/scratchpad/<work>-brief.md)" --repo sergeant-rs \
  --backend claude --workflow <name> --turns 40 --ceiling-secs 5400 --json
```

`"$(cat file)"` reads the brief's raw bytes and substitutes them as one
already-quoted argument — the shell does not re-parse backticks or `$(...)`
sequences *inside* the substituted text as new syntax, only the outer
command line's own quoting matters. This is the same pattern the prior
sprint's plan used (`docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md` §6:
"Briefs are files, passed `"$(cat …)"`"), adapted to this session's
scratchpad path per this harness's own convention rather than `/var/tmp`
(no build artifacts or `CARGO_TARGET_DIR` are involved here — only text —
so the repo's `/var/tmp` build-dir placement rule doesn't apply to brief
files specifically).

## 7. Orchestrator duties

- Watchers armed in the same response as each dispatch, backgrounded
  (`sgt --json watch <id>` / `--follow` for the parallel Wave 1 trio).
- No `completed` believed on its own: verify `git log base..sergeant/<id>`
  is non-empty and the named acceptance test actually passes, independently,
  the same discipline the prior sprint's #94 finding demands.
- A Wave-1 Work landing `failed`: retry once against the same base. For WB/WC
  a second failure blocks Wave 2 and escalates to the owner. For WA (not a
  Wave-2 dependency) a second failure is recorded and escalated but does not
  block the gate — WA can retry independently on its own schedule.
- WD's findings: `auto-fix` on the Work's own judgment for mechanical/
  low-risk items, `no-op` recorded, `ask-user` relayed to the owner verbatim
  and never resolved autonomously (ADR 0005, part of D1).
- **R6 enforcement**: before dispatching WD, the orchestrator confirms its
  brief carries no skip/reduced-profile instruction, and after dispatch
  confirms (via `sgt work show`/`no-mistakes axi status`) that every stage
  actually ran — a stage silently skipped is a defect to report, not an
  efficiency to accept.
- No merge to `main` (R1).

## 8. Risks

1. **WC (#128) may not resolve to a fix at all.** The issue's own text says
   profiling, not a guessed fix, is the actual scope — a Work here may
   legitimately end in "documented, not fixed," and that is an acceptable
   outcome per R-S0-1-style honesty, not a failed Work.
2. **WD depends on both WB and WC** (not WA), so a stuck WB or WC blocks the
   gate entirely — mitigated by the retry-once-then-escalate rule in §7, not
   by loosening the dependency.
3. **The estate clone is genuinely new** (§4) — first contact with `sgt repo
   add`/dispatch on this exact host carries the same "anticipate
   unanticipated findings" risk the arrival trip itself named; anything this
   plan didn't foresee gets recorded, not silently routed around.
4. **WC/WB file overlap risk** (§5) — real but small; a merge conflict here
   is an ordinary git event, not a process failure, and is handled at
   Wave-2 cut time.
5. **[v2] #130 may still surface inside WD anyway**, if `validate-and-ship`'s
   pipeline turns out to shell out to `scripts/gate.sh` somewhere this
   plan's read of `20-select-intent-transport`/`30-start-run`/`40-drive-gates`
   didn't catch — those three stages are what's been read; `50-reconcile-
   custody`/`60-close-out` and `no-mistakes`' own internals were not. If WD
   hits #130's exact failure mode, that is new evidence this plan's v2
   correction was itself incomplete, not a WD-owned bug — file it, don't
   force past it.

## 9. How this plan was reviewed

*Filled in after the panel runs — see `critics/`, `refuters/`, and
`adjudication.md` in this same directory.*

## 10. What this sprint closes

- #130, #129, #128 — closed by fix, or (for #128 specifically) closed by a
  measured, justified disposition if a fix does not materialize. #130 closes
  independently of the gate (§3, §5) — its landing is not a precondition for
  WD or for PR #126 being ready to merge.
- A gate Work's outcome recorded against `integration/macbook-arrival-2026-08-15`,
  dispatched per ADR 0005, not hand-driven.
- PR #126's body rewritten to cite this plan, its adjudication, and every
  Work ID that executed it — the "proper PR body" that only exists once the
  process that produced it is real.
