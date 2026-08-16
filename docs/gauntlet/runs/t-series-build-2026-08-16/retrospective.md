# Session retrospective — T-series build sprint

Written 2026-08-16 at the owner's direction, alongside PR #131. Covers the
whole session from proposal intake through the shipping gate: the
`T-SERIES-1` proposal gauntlet (8 Works), the owner's T2-14 client-boundary
ruling, the ten-Work build sprint (`docs/gauntlet/runs/
t-series-build-2026-08-16/plan.md`), and the shipping gate that found four
more real defects after all of it.

Owner's framing, carried forward from the Path-to-Mac retrospective because
it kept paying there too: **friction is evidence of leakage or ambiguity,
not junk. Missed steps usually mean unclear instructions; leftover residue
is usually a leak.**

`GAUNTLET.md`'s T-SERIES-BUILD entry and `docs/gauntlet/runs/t-series-1/
adjudication.md` record the gauntlet's own verdicts and are not repeated
here. This document carries what only a whole-session view shows.

---

## 1. Residue sweep

Run against the shipped tip `4c4b4dc`, fleet quiet (65 Works, zero
nonterminal).

| Residue | Measure | Cause | Disposition |
|---|---|---|---|
| `.sergeant/data` | **180 MB**, **0 retained surfaces** | 18 Works this session, all clean teardown | healthy |
| `/tmp` | 2% of 16 GB, clean | — | clean |
| orphan daemons | **0** beyond the one live daemon | — | clean |
| `completed_dirty` Works | 2 | both pre-date this session (2026-08-13) | not this sprint's residue |
| local scratch branches (custom-named, this session) | **17**, all merged | one per reviewed Work, created for diff inspection before merge | **swept this pass** — deleted after confirming ancestry into the integration branch |
| remote branches on GitHub (this session) | **0** | repo's auto-delete-on-merge setting, same as Path-to-Mac's finding | clean without action |
| `sergeant/<work-id>` branches (engine-retained, this session) | 14 of 18 present locally | **§1.1 — 4 missing, unexplained** | open finding, no data lost |
| `/var/tmp/sgt-rescue`, `sgt-rs-tests` | 156 KB, 4 KB | pre-existing, prior sprints | untouched, negligible |
| outer checkout (`repos/sergeant-rs`'s `origin`) branch | left on `main` | deliberate — see §1.2 | intentional, not residue |

### 1.1 Four of eighteen engine-retained branches have no local ref, with no data loss

`sergeant/01M03TVN35S8XXQFFF7FDBNK29` and its three sibling T-SERIES-1
fidelity/invariants/assumptions/enactability **critic** branches (not the
refuter branches — those were separately recovered, see LESSONS L25) do not
appear in `git branch` today, though every commit they pointed at is still
a real object (`git cat-file -t` confirms all four) and `git reflog --all`
shows no matching deletion event for any of them. Root cause not
identified: this checkout carries 150+ historical `sergeant/<id>` branches
from the repo's whole lifetime, and losing exactly these four amid that
volume was noticed only while gathering this retrospective's own numbers,
after the wildcard-fetch incident that L25 documents had already been
investigated and closed against a different four (the refuters). No data
was lost either way — the commits are recoverable by SHA if ever needed —
but the *mechanism* by which these four specifically lost their ref is an
open question this entry is deliberately not papering over.

### 1.2 The outer checkout was moved to `main` on purpose, and left there

Running the shipping gate required moving `repos/sergeant-rs`'s `origin`
remote (a live, non-bare working copy) off `integration/t-series-2026-08-15`
and onto `main`, or issue #120's empty-diff condition fires immediately —
which it did, once, exactly as designed, before the fix. Left on `main`
rather than switched back: any other branch this estate's `origin` sits on
is a live #120 hazard for the *next* gate run, and `main` is the one branch
where a genuine diff against it is guaranteed for as long as any other
branch in this estate is ahead of it. This is a workaround for #120's root
cause, not a fix for it — see §7.3.

---

## 2. Product defects filed

**None, filed separately.** Every defect this session's review surfaced —
the four review findings the shipping gate found, the stale
`docs/DEVELOPMENT.md` fact the document step found, the ratatui-textarea
spike gaps T-SERIES-1's enactability axis found — was fixed in the same run
that found it, not deferred to a tracked issue. The two genuinely deferred
items (the slash palette, the Workflows `@` chooser half) are tracked
in-code (`Overlay::SlashPalette::owner()`) and in-proposal, not as GitHub
issues, since neither is a defect — they were never built, by explicit,
named scope decision.

---

## 3. Instruction and brief defects — these are mine

**A wildcard git fetch refspec deleted branches it had no remote counterpart
for.** `git fetch github 'refs/heads/sergeant/*:refs/heads/sergeant/*'`,
issued to sync one thing, overlaid remote state onto every local ref
matching the pattern — including four freshly-created, never-pushed refuter
branches the remote had no way to know about yet. Recovered in full from
each Work's own `teardown.bindings[].final_sha` (LESSONS L25), but the
command should never have been wildcarded against branches that might not
exist on the remote yet.

**I used Bash's background-notify for a `sgt watch --follow` that never
exits.** `run_in_background` notifies on process exit; an estate-wide
`--follow` watch has no exit condition by design. The result: it ran
correctly and captured every event, but nothing surfaced until the user
asked what a sample event looked like and I went looking at the raw output
file — 35 KB of transitions had already streamed past silently. Switching
to the `Monitor` tool (per-line notification) fixed it; the fix should have
been the first choice, not a correction after being asked to show a sample.

**I edited the proposal directly, then was told to dispatch it instead.**
Mid-way through applying T-SERIES-1's adjudicated corrections, I started
hand-editing `reference/proposal-tui-t-series.md` in-session rather than
dispatching the mechanical work as a Work — three edits landed before the
owner's correction ("Shouldn't you use an agent... you are the captain").
Reverted cleanly (nothing was committed yet) and the rest of the sprint
followed the dispatch discipline throughout. The instinct to just do a
small, obviously-correct edit directly is exactly the instinct this
session's own standing instruction ("follow the process no matter how
minor") exists to override.

**One Work ID transcription slip.** Dispatched T1a, then referred to it by
a different, wrong ID (apparently copied from a truncated JSON preview)
until the completion notification carried the real one and the mismatch
surfaced on the first `sgt work show` call. Caught immediately, cost
nothing, but is exactly the kind of small friction this section exists to
record rather than wave off.

---

## 4. Tool-call patterns

**Recorded because the corpus should carry confirmed approaches, same as
Path-to-Mac §5:**

- **Never trust a Work's own "done" claim — read the diff, then build, then
  test, independently, every time.** Applied to all eighteen Works this
  session with zero exceptions. It caught real things: a flaky PTY test
  that needed a second run to distinguish from a regression (twice), and
  gave the confidence to merge two parallel Works' conflicting diffs by
  hand rather than re-dispatching a third Work to resolve them.
- **`git -C <path>` for anything touching more than one checkout.** This
  session ran two clones of the same repo (the outer working checkout and
  the daemon's own estate mount) side by side for its entire duration.
  Ambient-cwd `cd`-then-run commands produced one real confusion (a
  fetch/merge that appeared to succeed against the wrong checkout); every
  `git -C` invocation after that point was unambiguous by construction.
- **A dispatched Work's journaled `final_sha` outlives its local git
  ref** (LESSONS L25) — the single highest-leverage recovery fact this
  session used, twice: once for the wildcard-fetch incident, and implicitly
  every time a Work's branch was recreated from `sgt work show`'s teardown
  record rather than assumed lost.
- **Check a workflow's actual stage shape before dispatching against it.**
  `to-spec` looked like the right fit for T0 by its one-line catalog
  description alone; reading its actual stages revealed a GitHub-issue-
  publishing shape that didn't match "amend this proposal in place." Caught
  before dispatch, not after — the five minutes spent reading
  `workflow.toml`/`CONTEXT.md` for `to-spec`, `tdd`, and `diagnose-bug`
  before committing to a workflow choice paid for itself by avoiding a
  wasted dispatch.

---

## 5. What worked, measurably

- **The proposal gauntlet paid for itself before a single line of product
  code existed.** T-SERIES-1's four-axis panel found 19 findings, 18
  confirmed, and one — the Estate/Doctor client-boundary bypass — would
  have failed `cargo test` on T1a's very first run had it shipped as
  drafted. The panel caught it at the proposal stage, for the cost of 8
  Work dispatches against prose, not against a rewrite of already-built
  code.
- **Parallel dispatch across a genuinely disjoint wave worked, including
  the conflict.** T2 and T3 were dispatched from the same base, landed in
  either order, and collided exactly where the plan predicted (shared
  scaffolding: `src/api.rs`'s imports, `src/tui/{app,mod,overlay}.rs`'s
  enums). The conflict was real but entirely mechanical — sibling entries
  in the same list, never a semantic disagreement — and resolved by hand in
  under the time a third coordinating Work would have cost to dispatch and
  verify.
- **The shipping gate found four more real bugs after everything else had
  already reviewed the code.** T-SERIES-1's panel, ten independently
  Captain-verified Works, and a full green test suite at every merge still
  left a double-submit race, a false-failure-on-success response, an
  overflow, and a navigation regression for `no-mistakes`'s own review step
  to catch. Same finding as Path-to-Mac §5/§6, reproduced on a completely
  different piece of work: **the gate is not a formality after review has
  already happened — it finds things review does not.**
- **The self-authored gate-guard protected its own sprint.** The #120
  pre-flight fix, dispatched mid-sprint specifically because T4's own gate
  run would need it, fired exactly once, on this exact sprint's own gate
  run, for exactly the reason it was written. Not a hypothetical
  precaution — measured, same day.

---

## 6. Calibration

**Every Work used exactly its stage count in turns, regardless of turn
budget.** All four `software-change`-workflow Works (4 stages) used
**exactly 4 turns**; all six `tdd`-workflow Works (2 stages) used **exactly
2 turns** — no exceptions across eighteen dispatches. Budgets were sized at
`--turns 24` and `--turns 40` respectively, out of the same #90-adjacent
caution Path-to-Mac's calibration section names. Realized cost: **6× and
20× over-provisioned**, the second figure matching Path-to-Mac's own "20×"
finding for `implement`-shaped Works almost exactly, on an unrelated
workflow (`tdd`) and an unrelated repo surface (a TUI, not platform code).
Two data points now agree: **in this repo, turn count is bounded by
workflow stage count, not by task complexity** — a `tdd` Work restructuring
2,614 lines into a twelve-file module tree cost the same 2 turns as one
adding a single client method. Future sizing should start near stage count
plus a small margin, not a round number chosen from wedge-avoidance
anxiety.

**Cost, whole session:** 18 Works dispatched (8 gauntlet + 1 ruling + 1
corrections + 8 build), **zero failures, zero retries, zero `needs_input`**
— the plan's own "retry once, escalate" policy was never invoked because
nothing ever failed once. 60 turns total (32 gauntlet/ruling/corrections +
28 build). Test count: 654 (measured at PR #126, before this session) →
**832** across the full suite at the shipped tip, zero `SKIPPED-ENV`
throughout.

---

## 7. Proposed, not made — the owner's to rule

Deliberately not enacted; each changes governing text, product behavior, or
spends real effort this entry does not authorize on its own.

1. **Fix #120 at its root, not just its symptom.** This sprint's gate-guard
   refuses loudly instead of silently false-passing, and this entry's own
   §1.2 shows the workaround (moving `origin`'s checked-out branch) still
   works but has to be remembered. The issue's own second fix candidate —
   estate scaffolding gives `no-mistakes` a bare mirror instead of a live
   non-bare working copy — would close the class of hazard instead of
   guarding one instance of it. **Recommend filing as the follow-up.**
2. **Close review-5's TOCTOU window**, or rule it acceptable as-is. The
   gate-guard's own base-detection snapshot and `no-mistakes`'s later
   internal rebase are not atomic; the gap is sub-second and requires
   another actor pushing this exact branch to `origin` in that window, but
   it is real and the gate's own review step named it rather than staying
   silent about it.
3. **Root-cause §1.1's four missing branch refs.** Not urgent — nothing was
   lost — but an unexplained residue in a system whose whole architecture
   is "the journal is the only truth, everything else is disposable" is
   worth understanding rather than shrugging off, especially since it
   surfaced by accident while writing this retrospective rather than by
   deliberate audit.
4. **Real terminal PNG screenshots**, not `TestBackend` text dumps. T4's
   own scope note (and this session's own environment) had no real
   terminal session available to capture from; `docs/tui-screenshots/*.txt`
   is honest and current but README's `docs/img/tui-*.png` pair is still
   the M6-era interface until someone captures the new one from a real
   terminal.
5. **Build the two named T-series gaps** (§15.3's slash palette, §15.4's
   Workflows-screen `@` chooser half) — deliberately deferred by T1c and
   never picked up by any later Work this sprint, tracked but not
   scheduled.
