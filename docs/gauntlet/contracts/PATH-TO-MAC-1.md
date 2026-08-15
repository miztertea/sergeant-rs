# PATH-TO-MAC-1 — contract

**Unit type:** a *plan* graded, not an implementation. Second instance of this
shape in the ledger; **FOUNDATION-1** is the precedent and its adjudication
(`docs/gauntlet/runs/foundation-1/adjudication.md`) is the model for this
unit's output.

**Artifact under grade:** `docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md`
at commit `d86885f`.

**Method:** `reference/notes/gauntlet-pattern.md` — BLIND CRITIC PANEL →
ADVERSARIAL VERIFY (batched per axis) → ADJUDICATE. Written by the
orchestrator per that document's model-assignment section ("Fable owns the
contract-writing and the tie-breaking").

## Bounded outcome

Decide whether the plan may govern a six-Work sprint as written, and produce
the corrections it needs first. Three outcomes are legitimate, and the panel
must not be steered toward any of them:

- **validated** — governs as written;
- **validated with findings** — governs after local corrections (FOUNDATION-1's
  outcome);
- **sent back** — a decision in the plan is wrong, not merely under-argued.

A truthful "this section cannot be graded from the evidence available" is a
successful result, not a failure to produce findings.

## Why this unit exists

`LESSONS.md` **L19**: a document that directs what gets built is executable
through the program that obeys it, so every governing artifact takes
fresh-context review before it governs. The plan was authored in a single pass
by the orchestrating session from a live owner interview, with **no prior
review** — the same provenance FOUNDATION-1 named as the honest explanation for
its own unusually high confirm rate (12/13). Panels should expect this artifact
to be softer than a Work's output, which passes a `tdd` stage, a `30-review`
stage and a shipping gate before the orchestrator sees it.

A narrower `code-review` pass (Work `01M01XGMY8JJ4M4RJDN1RSZXJC`, two axes) was
run first and is **input to this unit, not a substitute for it** — the owner's
ruling. It is available to the adjudicator; critics do not read it, because a
blind panel that reads a prior review is no longer blind.

## Axes

Four blind seats, one axis each, fresh context, dispatched as separate Works so
blindness is structural rather than promised. Adapted for a document exactly as
FOUNDATION-1 adapted them: `test-honesty` has no meaning for a plan and is
replaced by **enactability**; `simplicity` folds into **invariants** via the
Ponytail ladder to hold the panel at four.

| Axis | Grades |
|---|---|
| **fidelity** | The plan against the sources it cites. Does a cited document say what the plan says it says? |
| **invariants** | The plan against `NORTH-STAR.md`'s ownership boundaries and `docs/DEVELOPMENT.md`'s architecture invariants; Ponytail rung for every addition the plan proposes |
| **enactability** | Can a Work execute each section, or does confident prose hide an undecided question? |
| **assumptions** | Every factual and measured claim. FOUNDATION-1's `assumptions` axis caught "closed six issues" against a tracker where they were still open |

## Acceptance

- Each axis produces one cited Markdown findings file under
  `docs/gauntlet/runs/path-to-mac-2026-08-15/critics/<axis>.md`.
- Every finding carries: the exact plan text at issue, the governing text it
  contradicts **with file and line**, an argued severity (error / warning /
  info), and what a correction would be.
- Every claim distinguishes **verified in-session** from **believed** (L15).
- A finding that can be neither confirmed nor refuted is recorded `PLAUSIBLE`,
  never dropped (`gauntlet-pattern.md`, "Rules that outrank the loop").
- Adversarial verify: one refuter per axis, batched over that axis's findings
  (economy revision 2026-08-08), **each given a specific line of attack** —
  FOUNDATION-1's method note records that the axes given a concrete thing to
  try produced its only refutation and all three severity downgrades.
- Refuters never edit the artifact under review (**L5**); any mutation probe
  runs in a disposable worktree on `/var/tmp` and is reported.

## In scope

The plan's sections 1–10 as committed at `d86885f`.

## Explicit non-goals

- **The nine owner rulings in plan §2 are not re-litigated.** They are
  decisions, not derivations. A finding that a *different* decision would be
  better is out of scope. A finding that a ruling **as written down**
  misrepresents the source it cites, or contradicts a governing document, is in
  scope and is a fidelity question.
- No code is written or edited by this unit.
- No gate run. `scripts/gate.sh`/no-mistakes belong to a separate Work.
- The crate measurements in plan §4 are not re-derived. A critic who doubts one
  names it and names the command that would settle it.

## Unknowns

Named rather than papered over, per the method doc's requirement that a
contract state what is genuinely unresolved:

1. **Whether a four-axis document panel is the right instrument twice.**
   FOUNDATION-1 answered its own Unknown 3 ("does a code gauntlet degrade into
   style commentary on prose") with *no*. One unit is not a trend, and this
   artifact is a plan rather than a proposal — shorter, more operational, and
   denser in measured claims.
2. **Whether an all-Sonnet panel holds on a judgment-dense document.** The
   owner ruled Sonnet for every seat. `gauntlet-pattern.md`'s model assignment
   reserves Opus for "blind adversarial review" and calls cross-model diversity
   "an independence measure, not a cost measure." FOUNDATION-1 nonetheless ran
   8 all-Sonnet seats and produced a refutation and three downgrades. This unit
   is the second data point, and the result is worth recording either way.
3. **Whether `research` is the right workflow for a critic seat.** Chosen for
   its durable outcome — "primary sources only, every claim traced; one cited
   Markdown findings file" — which is a critic's contract almost verbatim. It
   was not built for adversarial grading, and any friction is a finding about
   the workflow library worth carrying to the ledger.

## Outputs

- `docs/gauntlet/runs/path-to-mac-2026-08-15/critics/{fidelity,invariants,enactability,assumptions}.md`
- `docs/gauntlet/runs/path-to-mac-2026-08-15/refuters/{fidelity,invariants,enactability,assumptions}.md`
- `docs/gauntlet/runs/path-to-mac-2026-08-15/adjudication.md` — orchestrator,
  with a verdict table in FOUNDATION-1's shape (axis / findings / refuted /
  confirmed / severity moves) and every correction applied to the plan.
- A `GAUNTLET.md` ledger entry with both scorecards.
