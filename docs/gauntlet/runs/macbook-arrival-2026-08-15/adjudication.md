# MACBOOK-ARRIVAL-1 — adjudication

## Outcome: **validated with findings**

No section sent back. Every finding that survived adversarial verify is a
local correction to the plan's text — a citation misattribution, a
self-contradictory word choice, a missing per-Work instruction — not a wrong
decision. The two structurally alarming findings (E1, F2) were each partially
refuted: the mechanism the critic feared does not exist as described, and a
narrower, real, INFO-level risk remains in its place.

## Verdicts

13 findings across four axes: 0 outright refutations of a correct finding, 2
partial refutations (both narrowing a WARNING to INFO), 11 confirmed as
written, 2 severity moves (both down).

| Axis | Findings | Refuted | Confirmed | Moves |
|---|---|---|---|---|
| fidelity | 5 (1 + 4 PLAUSIBLE) | 0 | 5 (all PLAUSIBLE resolved to confirmed-accurate on further investigation) | none |
| assumptions | 2 | 0 | 2 | none |
| enactability | 3 | 1 (structural arm of E1) | 3, one narrowed | E1: WARNING → INFO |
| invariants | 3 | 1 (structural arm of F2) | 3, one narrowed | F2: WARNING → INFO |

## The citation error three axes converged on

Fidelity (F1), enactability (E2), and invariants (F2) each independently
found the same misattribution: the plan's §2 R6 cites the medium-profile
skip behavior as coming from `20-select-intent-transport`'s own citations.
It is actually `30-start-run/CONTEXT.md` lines 66–68 (`BU-P1-042`). Three
independent seats landing on the same file:line is the strongest kind of
signal this method produces — fixed below, no further debate warranted.

The invariants refuter went further and found the citation was pointing at
the wrong *tool* as well as the wrong stage: `BU-P1-042` describes
`reference/sergeant-upstream/bin/sgt-validate`'s own hardcoded
`SKIP="review,document"` default — a separate upstream coordinator-beside-
worker script that `validate-and-ship`'s actual stages never invoke. Stage
30's real behavior contract is "call `no-mistakes axi run --intent \"...\"`
directly," which carries no default skip. The plan's R6 was worried about a
mechanism that isn't in its own dispatch path.

## What R6 actually needs to guard against, narrowed by both refuters

Both `E1` and `F2` had a structural arm (a default behavior would silently
skip stages) that got refuted, and a narrower arm that survived:

- **`no-mistakes axi run --skip <steps>` is a real, confirmed flag**
  (invariants refuter, verified via `no-mistakes axi run --help`). Nothing
  in `validate-and-ship`'s six stage files mechanically prevents an executor
  from passing it. The only guard is the brief's own instruction not to.
- This is an **executor-honesty risk, not a default-path trap** — it fires
  only if WD's dispatched Work actively chooses to pass `--skip` against an
  explicit brief prohibition. That is genuinely lower-severity than a
  default behavior silently doing the wrong thing, which is why both
  refuters moved it to INFO rather than confirming WARNING or refuting it
  outright.
- §7's existing post-dispatch check ("confirms every stage actually ran")
  is an adequate operational guard at this severity — it was already in the
  plan before this panel ran.

## WC's acceptance-criterion question, resolved

E1 asked whether `implement`'s TDD-completion machinery structurally blocks
WC's documented-no-fix outcome. The refuter read `implement/10-implement-
with-tdd/CONTEXT.md` and `implement/30-review/CONTEXT.md` directly: no
behavior unit in either stage requires a green test as a completion gate;
`30-review`'s durable outcome is "the change is reviewed" against
whatever the brief establishes as the seam, and WC's brief already
establishes "profiled, no fix, documented" as an acceptable seam outcome.
The real defect is narrower: §7's blanket "verify the acceptance test
passes" line isn't explicitly reconciled with §6's WC carve-out. Fixed
below by making the carve-out explicit in §7, not by changing WC's
workflow — the refuter separately confirmed dispatching WC via
`--workflow research` instead (the critic's alternative correction) would
be wrong, since `research`'s single `00-investigate` stage has no
implementation capacity and WC's primary expected outcome is still a code
fix or floor revision.

## Corrections applied to the plan (in place, `[v4]`)

1. **§2 R6** rewritten: the "medium profile" framing removed (it named the
   wrong tool as well as the wrong stage file); replaced with the actual
   confirmed risk (`no-mistakes axi run --skip`) at the severity both
   refuters settled on.
2. **§5**: "disjoint file sets" → "largely non-overlapping file sets," with
   the assumptions refuter's specific finding that the two issues' plausible
   fix regions in `engine.rs` (WB ≈ lines 2383–2800, WC ≈ 1089–1692 +
   2848–3183) do not share a function, so the correction is neither an
   undersell nor an oversell of the real risk.
3. **§6**: the adjacent-append crash-window check (L6) added to WC's brief
   too, conditioned on WC actually landing a code change in
   `src/runtime/engine.rs`/`surface.rs` — WB was the only one carrying it
   before, and WC's file scope includes the same files with a live
   possibility of a code fix.
4. **§7**: an explicit WC carve-out added, reconciling the blanket
   acceptance-test-passes rule with §6's documented-disposition allowance.
5. **§4**: a one-line honesty note that Wave 0's `git fetch`/`checkout`
   steps are not literally documented by the estate-navigation skill's two
   named paths (`sgt repo add`, `git pull --ff-only`) — functionally
   correct, just outside what the skill currently writes down.

## Method notes for the ledger

- **Three-axis convergence on one citation** is the strongest single
  signal this run produced — worth recording as a pattern to watch for
  again: when independent blind seats land on the identical file:line
  without being told to look there, that finding needs no further
  adversarial pressure.
- **Both refuters given a specific line of attack moved something** —
  consistent with `FOUNDATION-1` and `PATH-TO-MAC-1`'s own recorded
  pattern; a fourth data point for treating this as standing practice
  rather than a per-unit choice.
- **`--workflow research` served the critic *and* refuter seats fine for a
  document/citation-shaped review**, including seats whose job was to run
  `--help` output and read source files, not just prose — no friction
  observed at this unit's scale, unlike `PATH-TO-MAC-1`'s own note about
  `research` carrying no built-in verdict vocabulary (this unit's briefs
  supplied that vocabulary explicitly in the prompt, which resolved it
  without needing a dedicated seat workflow).
- All eight dispatched Works (4 critics, 4 refuters) completed on the first
  attempt, zero `needs_input`, zero failures — verified independently via
  `git log`/`git diff --stat` on each Work's branch before being trusted,
  not accepted on the Work's own "completed" self-report alone.
