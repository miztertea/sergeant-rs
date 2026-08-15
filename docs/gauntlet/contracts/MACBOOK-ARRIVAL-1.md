# MACBOOK-ARRIVAL-1 — contract

**Unit type:** a *plan* graded, not an implementation. Third instance of this
shape in the ledger; `FOUNDATION-1` and `PATH-TO-MAC-1` are the precedents —
`PATH-TO-MAC-1`'s adjudication (`docs/gauntlet/runs/path-to-mac-2026-08-15/adjudication.md`)
is the closer model, since both plans govern a small multi-Work sprint on
this same repo.

**Artifact under grade:**
`docs/gauntlet/runs/macbook-arrival-2026-08-15/plan.md` at the commit it is
first checked in at (recorded in this run's `adjudication.md` once graded).

**Method:** `reference/notes/gauntlet-pattern.md` — BLIND CRITIC PANEL →
ADVERSARIAL VERIFY (batched per axis) → ADJUDICATE. Orchestrator-authored
(this session, Sonnet 5, acting as Captain — not the Fable-orchestrator seat
`gauntlet-pattern.md`'s model-assignment section describes; noted as a
deviation, not hidden).

## Bounded outcome

Decide whether the plan may govern its three-Work sprint (plus the dependent
gate Work) as written, and produce the corrections it needs first. Three
outcomes are legitimate, and the panel must not be steered toward any of
them:

- **validated** — governs as written;
- **validated with findings** — governs after local corrections
  (`PATH-TO-MAC-1`'s outcome, and `FOUNDATION-1`'s);
- **sent back** — a decision in the plan is wrong, not merely under-argued.

A truthful "this section cannot be graded from the evidence available" is a
successful result, not a failure to produce findings.

## Why this unit exists

`LESSONS.md` **L19**: a document that directs what gets built is executable
through the program that obeys it, so every governing artifact takes fresh-
context review before it governs. This plan was authored in a single pass by
the orchestrating session, from a live correction the owner gave in-session
(not a `grilling` interview transcript) — softer provenance than a Work's own
output, same class of softness `PATH-TO-MAC-1` and `FOUNDATION-1` both named
for themselves.

## Axes

Four blind seats, one axis each, fresh context, dispatched as separate
subagents so blindness is structural rather than promised — same four axes
`PATH-TO-MAC-1` adapted from the code-gauntlet's original set, for the same
reason (`test-honesty` has no meaning for a plan; `simplicity` folds into
`invariants` via the Ponytail ladder):

| Axis | Grades |
|---|---|
| **fidelity** | The plan against the sources it cites — do `docs/DEVELOPMENT.md`, ADR 0005, the estate-navigation skill, and issues #128/#129/#130 actually say what the plan says they say? |
| **invariants** | The plan against `NORTH-STAR.md`'s ownership boundaries, `docs/DEVELOPMENT.md`'s architecture invariants, and `AGENTS.md`'s "When NOT to use `sgt`" boundary; Ponytail rung for anything the plan proposes building |
| **enactability** | Can a Work execute each Wave-1/Wave-2 section as written, or does confident prose hide an undecided question — e.g. §6's "[to verify at Wave 0]" `default_backend` flag, or WC's open-ended acceptance criterion |
| **assumptions** | Every factual and measured claim, especially §4's "0 repositories declared" / "nothing has ever run in this data dir" claim and §5's file-disjointness claim across WA/WB/WC |

## Acceptance

- Each axis produces one cited Markdown findings file under
  `docs/gauntlet/runs/macbook-arrival-2026-08-15/critics/<axis>.md`.
- Every finding carries: the exact plan text at issue, the governing text it
  contradicts **with file and line**, an argued severity (error / warning /
  info), and what a correction would be.
- Every claim distinguishes **verified in-session** from **believed** (L15).
- A finding that can be neither confirmed nor refuted is recorded
  `PLAUSIBLE`, never dropped (`gauntlet-pattern.md`, "Rules that outrank the
  loop").
- Adversarial verify: one refuter per axis, batched over that axis's
  findings, each given a specific line of attack (`PATH-TO-MAC-1`'s method
  note: the axes given a concrete thing to try produced its only refutation
  and all its severity downgrades).
- Refuters never edit the artifact under review (**L5**); any mutation probe
  runs in a disposable worktree on `/var/tmp`, never this checkout, and is
  reported.

## In scope

The plan's sections 1–10 as committed.

## Explicit non-goals

- **The five owner rulings in plan §2 are not re-litigated.** They are
  decisions the owner made in this conversation, not derivations. A finding
  that a *different* ruling would be better is out of scope. A finding that a
  ruling **as written down** misrepresents what was actually said, or
  contradicts a governing document, is in scope and is a fidelity question.
- No code is written or edited by this unit.
- No gate run. The gate is Wave 2's own dispatched Work, not this review's.
- Issue bodies for #128/#129/#130 are not re-derived — a critic who doubts
  the plan's characterization of one names the exact issue text it
  misrepresents.

## Unknowns

1. **Whether a single-orchestrator, all-Sonnet panel holds a third time.**
   `FOUNDATION-1` and `PATH-TO-MAC-1` both ran all-Sonnet seats and both
   produced real refutations/downgrades. This is a third data point on the
   same repo, with a Sonnet-5 (not Fable) orchestrator — an additional
   deviation from `gauntlet-pattern.md`'s original model table, worth
   recording either way.
2. **Whether a plan this small (three disjoint bug-fix Works plus one gate
   Work) still earns four full axes**, versus the "small diffs batch into the
   next larger panel" economy revision. Run as a full panel here on the
   `PATH-TO-MAC-1`/`FOUNDATION-1` precedent (both plans, both reviewed in
   full); worth naming as a candidate for that economy revision on the next
   plan-shaped unit if this one comes back thin.

## Outputs

- `docs/gauntlet/runs/macbook-arrival-2026-08-15/critics/{fidelity,invariants,enactability,assumptions}.md`
- `docs/gauntlet/runs/macbook-arrival-2026-08-15/refuters/{fidelity,invariants,enactability,assumptions}.md`
- `docs/gauntlet/runs/macbook-arrival-2026-08-15/adjudication.md` — orchestrator,
  with a verdict table in `PATH-TO-MAC-1`'s shape (axis / findings / refuted /
  confirmed / severity moves) and every correction applied to `plan.md` in
  place (dated revision entry, not a superseding sibling file).
- A `GAUNTLET.md` ledger entry with both scorecards.
