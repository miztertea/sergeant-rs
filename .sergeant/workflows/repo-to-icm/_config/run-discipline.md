# Run discipline — blindness and fail-closed propagation

Layer 3 (`_config/`), stable across every run, shared by **every** stage of
this workflow (`00-contract` through `90-reconcile` — every stage's own
`## Inputs` table names this file). Two rules that apply to the whole run,
not to any one stage's own contract: the blindness boundary, and how a
stage must react if an upstream artifact reports itself unresolved.

Moved here (from the workflow-level `../CONTEXT.md`, Layer 1) because Layer
1 orients and is not stage instruction, and the engine does not deliver it
to any stage past the first (`docs/icm/convention.md` §1a rule 5) — a rule
every stage must actually follow cannot live somewhere only one stage's
Inputs table names. `../CONTEXT.md` still summarizes both rules for a human
skimming the workflow; this file is the operative version an actor's own
stage contract actually points at.

## 1. The blindness rule

If this run's purpose is **measurement** — comparing generated output
against an already-adjudicated reference decomposition (as in the N2
measurement run against `sergeant-rs-workspace/knowledge/evidence/reference/sergeant-upstream`, graded against
`sergeant-rs-workspace/knowledge/evidence/reference-corpus/`) — then for the entire run, every stage's actor is
**blind to `sergeant-rs-workspace/knowledge/evidence/reference-corpus/`**: never open it, never grep it, never let
it enter a prompt, never let a helper's output surface its contents. This
is not a preference; contaminating the run with the answer key invalidates
the measurement it exists to produce (`sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/N2.md`
Outcome §2). A stage that needs to check its own output's shape uses
`../scripts/validate-structure.py`, never `sergeant-rs-workspace/knowledge/evidence/reference-corpus/lint.py` or any
file under `sergeant-rs-workspace/knowledge/evidence/reference-corpus/` — those are the graders' tools, applied
*after* this run completes, by a separate comparison process this
workflow's actors do not perform. If a stage's actor is ever tempted to
peek "just to check the shape," that temptation is itself the signal to
stop and re-read this paragraph.

Outside a measurement run (e.g. a first decomposition of a repository with
no existing reference corpus), this rule is vacuous — there is nothing to
be blind to — but the discipline it enforces (cite only the *target*
repository under decomposition, never a pre-existing answer) still holds:
never source a behavior unit's `quote` from anywhere but the repository
this run is decomposing.

This is the run's central safety constraint, not a suggestion any one
stage can treat as background context — every stage listed above is bound
by it for the whole time it is executing, whether or not its own
`CONTEXT.md` restates it inline.

## 2. Fail-closed propagation: the `# AMBIGUOUS — NOT RESOLVED` marker

`00-contract`'s own contract requires it to fail closed rather than guess
when the subject repository, revision, or scope is ambiguous — and, because
the current engine gives no actor stage a way to pause its turn and wait
for a human answer, the fail-closed action available to it is to write
`output/contract.md` anyway, headed:

```text
# AMBIGUOUS — NOT RESOLVED
```

**Every stage from `10-inventory` onward: before doing any of your own
stage's work, check whether the upstream artifact(s) named in your own
Inputs table open with this exact heading.** If any does:

1. Do not proceed with your stage's ordinary work — do not invent a subject,
   revision, scope, or any other fact the unresolved artifact was supposed
   to establish.
2. Write your own stage's declared `output/` artifact(s) starting with the
   same `# AMBIGUOUS — NOT RESOLVED` heading, so the marker chains
   mechanically through every later hop without each stage having to sniff
   free-form prose for it. Under the heading, name which upstream artifact
   you found it in and quote its "What is ambiguous" line — do not
   originate a new explanation; you are relaying, not re-diagnosing.
3. Stop. Do not attempt an ordinary durable outcome you were not given the
   facts to reach honestly.

This propagation is itself the run's fail-closed behavior end to end: one
stage's honest "I could not resolve this" turns into every downstream
stage's honest "I did not proceed on an unresolved input," rather than a
guess silently compounding through nine more stages.
