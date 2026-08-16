# The ICM decomposition ladder

Layer 3 (`_config/`), stable across every run of this workflow — the
classification method `40-classify` applies to every normalized behavior
unit, distilled from proposal §6 and the amended representation vocabulary
in `docs/icm/record-shapes.md` §4 (N1 adjudication A1). For every behavior
unit, ask these questions **in order** and stop at the first one that
answers yes; each question's representation is the classification.

## 6.1 — Is it a stable operating invariant?

Does the rule apply broadly and change rarely, independent of any one
procedure's current stage? (Examples: use Sergeant for durable substantive
work; do not silently substitute a harness; preserve source history and
authority boundaries.)

**Representation:** `agents-invariant` (→ `AGENTS.md` or another stable
repository instruction surface).

## 6.1a — Does this procedure decide what Work should exist, or execute one already admitted?

Added by `docs/adr/0013-icm-r0-owner-rulings.md` decision 1
(`reference/proposal-icm-r-procedure-authority.md` §3.3, Finding ICMR-F3):
§6.2's own question — "does it have a recognizable trigger, a bounded
outcome, and a completion condition?" — is necessary for a workflow but
not sufficient. A Captain skill can have all three too. This question must
be asked and answered **before** §6.2, not skipped past the way §6.3 was
historically skipped past §6.5 (the same failure mode, one rung earlier).

**The discriminator:** if the procedure's job is to converse with the
user, shape or revise intent, decide whether work stays direct or becomes
a durable Work, select a workflow/repositories/profile/envelope, or
interpret a result back to the user — it cannot itself require an
already-existing Work merely to make that decision. It receives no
admitted intent; it produces one.

**Representation:** `captain-skill` (→ `skills/<name>/SKILL.md`, per the
Placement Ladder's PL-2 — see `reference/proposal-icm-r-procedure-
authority.md` §5.4 for the full rung and its worked examples) if the
answer is "decides what Work should exist." Continue to §6.2 only if the
answer is "executes an already-admitted intent."

A unit that is a reusable reasoning or operating technique used *inside*
either a Captain interaction or a workflow stage, without owning a
complete durable Work lifecycle itself, is neither — see PL-3 (`actor
skill`) in the Placement Ladder.

## 6.2 — Is it a reusable procedural outcome?

Does it have a recognizable trigger, a bounded outcome, and a completion
condition that could be invoked independently? (Examples: diagnose a
defect; prepare a prototype; validate and ship a change.)

**Representation:** `workflow`.

## 6.3 — Is it a meaningful durable checkpoint inside that procedure?

Would operators care that the work entered, blocked in, retried, completed,
or failed at this boundary? Does a fresh execution context matter? Should
its time, cost, evidence, and failure rate be measurable independently?

**Representation:** `stage`.

**The reimplementation test** (the discriminator to actually apply, not
just cite): *if this script were replaced tomorrow by another
implementation — Bash, Python, GitHub Actions, three manual commands, the
mechanism is irrelevant — would the procedural checkpoint still exist?* If
`release-verification` remains a meaningful boundary no matter what
implements it, it is a stage. If `test.sh` is merely one tool an
implementation actor reaches for before declaring implementation complete,
it is a helper (§6.5 below), not a stage — a script does not become a
stage merely because it is executable.

**The question must actually be answered, not skipped past.** N2
adjudicated exactly this failure at scale
(`docs/gauntlet/runs/n2-run2/comparison-c2-boundaries.md` §4.2): 73% of one
run's corpus was classified `helper`/`shared-helper` with a rationale that
answered §6.5's question ("is this deterministic machinery subordinate to
some checkpoint?") *without ever recording an answer to §6.3's own
question* for the same behavior — the reference corpus, applying the
identical documented test to the identical sentence in the identical file,
answered §6.3 "yes, operators would want this measured" every time. **A
unit may not be classified `helper`, `shared-helper`, or `shared-context`
without its `rationale` first stating the §6.3 answer explicitly — including
when that answer is "no."** "This is deterministic machinery" is not itself
a §6.3 answer; it is where you land *after* answering §6.3 "no, replacing
the implementation would not remove a checkpoint anyone cared about" and
continuing down the ladder. A rationale that jumps straight to §6.5's
language without that sentence has not cleared §6.3 — it has skipped it.
`40-classify/CONTEXT.md` states this as a required step, the same way this
run's own reference (`reference-corpus/`) required it in its own A4
adjudication (`docs/gauntlet/runs/n2-run2/comparison-c2-boundaries.md` §3):
a checkpoint question is answered *before* a lower-rung classification is
legal, not assumed by arriving there.

**The over-promotion tell, downstream.** `50-synthesize` clusters
`helper`/`shared-helper` records by contract (what a helper does, for whom
— `references/synthesis-method.md` bucket 5), not by originating file. If,
once clustered, a bucket-5 group's membership maps one-to-one onto a single
source file's own extracted units — this file's helper group is exactly
this file's units, no more, no fewer, and every other covered file gets its
own separate group the same way — that is not behavior-shape clustering, it
is file-shape mirroring, and it is the visible symptom of the same
rung-ordering error one stage upstream: §6.5 was answered before §6.3 was.
A behavior correctly classified `helper` because of what it *does* should
cluster with other helpers doing the same thing regardless of which file
happened to contain them; a corpus where helper clusters instead reproduce
the inventory's own file list one-for-one is the tell that §6.3 was never
actually asked (`docs/gauntlet/runs/n2-run2/comparison-c2-boundaries.md`
§4.2's own worked examples: `sgt-recover`, `sgt-respond`, `sgt-watch`,
`sgt-drain*` each became its own single-file helper cluster this way).

## 6.4 — Does the checkpoint require judgment?

Does an actor need to inspect evidence, choose among alternatives, ask the
user, modify work, or explain a decision at this checkpoint?

**Representation:** `stage-context` — actor guidance that belongs inside
that stage's own `CONTEXT.md`, not a new stage boundary by itself. (Note
the vocabulary split: `stage` at 6.3 names the checkpoint; `stage-context`
at 6.4 names *content that lives inside* an already-established stage's
context. A unit can require 6.4's judgment without itself creating a new
6.3 checkpoint — most `stage-context` units attach to a checkpoint some
other unit already established.)

## 6.5 — Is it deterministic machinery used while crossing a checkpoint?

Does it perform a repeatable operation whose invocation is subordinate to
the stage's outcome — collecting an inventory, validating a schema,
running a fixed test command, normalizing JSON, producing a diff summary?

**Representation:** `helper` (a script or executable referenced by the
stage's own context, never a stage in its own right — `docs/icm/
convention.md` §5 rule 1 applies the same reimplementation test from 6.3
to draw this line).

## 6.6 — Is the helper or context reused?

If it belongs to exactly one workflow, it stays workflow-local. If more
than one workflow uses it with the *same contract* — same inputs, same
output shape, same meaning — it is shared.

**Representation:** `shared-helper` / `shared-context` (→
`.sergeant/common/`) versus a workflow-local helper/context otherwise.

## 6.7 — Does Sergeant itself need to own a new durable fact?

Can the behavior not be represented faithfully at any lower rung because
the *runtime* — not the actor, not a helper — must own ordering, identity,
retry, recovery, authorization, isolation, or evidence semantics?

**Representation:** `engine-gap`. This is the last rung, reached only after
every lower rung above has been tried and has failed for a reason specific
to that rung's own mechanics — not "would be convenient" or "could be more
elegant," which are outright disqualifying, not merely under-detailed
(proposal §6.7).

An `engine-gap` classification's nested `engine_gap` object must carry the
full template, verbatim field names, all six required
(`record-shapes.md` §5; a record missing any one — most often
`lower_rungs_attempted` or `why_each_fails` — is auto-rejected at lint,
never merely flagged for review):

```text
behavior                              — what cannot be represented
source_evidence                       — the behavior unit id(s) requiring it
lower_rungs_attempted                 — actual ladder rungs tried (6.1-6.6
                                         names, not a restatement of the gap)
why_each_fails                        — a reason specific to THAT rung's
                                         mechanics, one entry per attempted
                                         rung (identical reasons across every
                                         rung is itself a violation — it
                                         shows the rungs were not actually
                                         reasoned about individually)
minimum_runtime_capability_required   — the smallest new durable fact the
                                         runtime would need to own
observable_acceptance_test            — a checkable scenario after the
                                         capability exists, not a restated
                                         feature name ("the engine supports
                                         nested workflows" does not qualify)
```

## One more disposition, outside the ladder itself

A unit may also resolve to `obsolete-mechanism`: the behavior is real, but
the mechanism the source implements (a specific shell script, a sentinel
file, a tmux pane) is something the current runtime replaces structurally
— the ladder's job is then to find where any *surviving policy* re-homes
(usually `agents-invariant` or `stage-context`), not to force the obsolete
mechanism itself into a rung it no longer needs (proposal §8.1-8.2).

## Recording the classification

Every classification carries `rationale` (why *this* rung and not an
adjacent one — never a restatement of the behavior itself, which fails to
discriminate) and `alternatives_considered` (the other rungs weighed and
rejected). `alternatives_considered` may be empty only where no adjacent
rung was facially plausible; it is required non-empty for every unit
carrying a workflow or stage boundary and every `engine-gap` unit
(`record-shapes.md` §4). A classification is a claim, not a fact, until
`80-adversarial-review` and `90-reconcile` have had their turn at it.
