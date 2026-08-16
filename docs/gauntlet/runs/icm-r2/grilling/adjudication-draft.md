# Package adjudication: grilling

Producer pass, ICM-R2 pilot (`docs/adr/0013-icm-r0-owner-rulings.md` decisions 8-9;
method: `reference/proposal-icm-r-procedure-authority.md` §8). Package-specific
hint supplied at dispatch ("likely STAND, verify Bounded-judgment section is
complete") is verified below against current content, not assumed — see
Behavior-unit dispositions and Final disposition.

## Original intention

Interview the user relentlessly, one question at a time, about every aspect
of a plan/decision/idea until a shared understanding is reached, offering a
recommended answer alongside each question, and gate any action on the
interview's conclusions behind the user's explicit confirmation. Originally
authored (`reference/sergeant-upstream/.agents/skills/grilling/SKILL.md`) and
first promoted into this repository as a two-stage `sgt`-run workflow
(N1 candidate W28: `00-interview-loop`, `10-confirm-understanding`;
provenance: `docs/gauntlet/promoted-provenance/grilling.md`).

Rehomed workflow → skill on 2026-08-12 (`docs/icm/re-homing-record-2026-08-12.md`
row 32) under North Star ruling R-NS-6 ("execution ≠ dialogue"): a dogfood
measurement found both stages completing autonomously with **zero**
`needs_input` pauses in 2/2 runs on this host — "negative value vs plain
terminal Claude" — because a durable `sgt run` stage has no mid-turn hold for
a human's answer to land in. The current package is the sole surviving
artifact of that rehome: `skills/grilling/SKILL.md`. No `.sergeant/workflows/
grilling/` directory exists any longer (confirmed by direct filesystem check
this pass); the workflow identity is fully retired, not merely deprecated in
place.

## Current trigger and outcome

**Trigger:** the user wants their plan, decision, or idea stress-tested, or
invokes a "grill" trigger phrase, during a live interactive session.

**Outcome:** either (a) a shared, user-confirmed understanding is reached
before any implementation, `sgt run` submission, or file edit driven by the
interview's conclusions, or (b) — when the harness cannot actually hold a
mid-turn pause open for a human answer — the actor produces its own
best-guess answers and states plainly that nothing was confirmed, rather than
presenting an unconfirmed guess as a reached shared understanding.

## Driver and admission boundary

**Driver:** Captain (the interactive harness). This is the entire point of
the 2026-08-12 rehome and is restated explicitly in the package itself
("Run this interview directly in the current conversation — never via
`sgt run`").

**Admission boundary:** pre-work / always. The skill runs inside the live
conversation and never itself requires an already-admitted Work; if its
conclusions warrant durable execution, a *separate* subsequent act (an
`sgt run` submission) would create that Work — the interview itself is
gated before that, per PL-2's discriminator ("if the procedure's job is to
decide what Work should exist, it cannot itself require an already-existing
Work merely to make that decision").

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-R2-GRILL-01 — Trigger is a stress-test request or "grill" phrase; runs directly in the current conversation, never via `sgt run` | `skills/grilling/SKILL.md` "## When to use" (lines 18-22) | PL-2 | J5 — North Star R-NS-6 ("execution ≠ dialogue") forbids dispatching this as durable Work; a lower rung cannot override it | STAND | `skills/grilling/SKILL.md` |
| BU-R2-GRILL-02 — Interview walks each branch of the decision tree, resolving dependent decisions in order, offering a recommended answer alongside each question | `skills/grilling/SKILL.md` "## How to grill" para 1 (lines 26-29) | PL-2 | J2 — this skill delegates branch order and recommendation content to the actor within the named bound "offer your own recommended answer alongside each question" | STAND | `skills/grilling/SKILL.md` |
| BU-R2-GRILL-03 — One question at a time: ask, then wait for the user's answer before asking the next | `skills/grilling/SKILL.md` bullet "One question at a time." (lines 31-32) | PL-2 | J5 — the skill's own contract forbids batching; not a delegated choice | STAND | `skills/grilling/SKILL.md` |
| BU-R2-GRILL-04 — Facts discoverable by exploring the environment are looked up by the actor; only genuine decisions are put to the user, one at a time | `skills/grilling/SKILL.md` bullet "Facts vs. decisions." (lines 33-36) | PL-2 | J2 — delegates the fact-vs-decision classification to the actor within the named bound (anything discoverable via filesystem/`sgt doctor`/tests/docs/`--help`/any available tool is a fact, not a question) | STAND | `skills/grilling/SKILL.md` |
| BU-R2-GRILL-05 — Hard gate: no implementation, `sgt run` submission, or file edit driven by the interview's conclusions until the user explicitly confirms shared understanding | `skills/grilling/SKILL.md` bullet "Do not act on the plan..." (lines 37-41) | PL-2 | J5 — governing constraint; matches "Captain shapes and admits the Work" (proposal §1) — the actor may not silently treat its own read of the conversation as confirmation | STAND | `skills/grilling/SKILL.md` |
| BU-R2-GRILL-06 — When the harness cannot pause mid-turn for a human answer, the actor degrades to best-guess autonomous answers and must say so plainly rather than presenting an unconfirmed guess as reached shared understanding | `skills/grilling/SKILL.md` "## Failure behavior" (lines 43-51), citing `docs/environments/cerberus.md` (Claude CLI row: non-interactive turns cannot hold a mid-turn pause open on this host) | PL-2 | J5 — governing constraint against silent degradation; this is the concrete evidence-backed rule the 2026-08-12 rehome exists to enforce | STAND | `skills/grilling/SKILL.md` |
| BU-R2-GRILL-07 — Package must carry an explicit local `## Bounded judgment` section (what it may decide, what it must ask the user, what it must not do, its durable handoff), always present, per the newly-ratified stage/skill requirement | `docs/adr/0013-icm-r0-owner-rulings.md` decision 4; `docs/icm/convention.md` §6.1 | PL-2 | J5 — a repository-doctrine requirement ratified 2026-08-16, binding on every Captain skill regardless of package-local preference | **gap — not present** | `skills/grilling/SKILL.md` (amendment required; see Surviving package design) |

Units 01-06 are the package's substantive behavior, unchanged in meaning
since the 2026-08-12 rehome and each independently traceable to current
source text; no unit needs to move, split, or fold elsewhere. Unit 07 is a
structural-compliance finding, not a behavior extracted from the package's
prose — it is the gap the package-specific dispatch hint pointed at, and it
is real: the file has no heading matching `## Bounded judgment` anywhere.
The hint's phrasing ("verify its Bounded-judgment section is complete")
presupposed a section that, on inspection, does not exist as a labeled
section at all — the equivalent content is present but scattered across
"How to grill" and "Failure behavior" in ordinary prose, not under the
required heading. This is a completeness/format gap in an otherwise sound
package, not evidence the package belongs on a different surface.

No residual workflow-era language was found: the file's own framing
("Ported from `.sergeant/workflows/grilling`... which retires") correctly
describes the rehome as history rather than leaking stage/`CONTEXT.md`/
`sgt run`-as-primary-mechanism phrasing into current instructions. The
"never via `sgt run`" clause is the opposite of residue — it is the
rehome's governing constraint restated for the reader.

## Surviving package design

`skills/grilling/SKILL.md` stands as currently structured (frontmatter +
When to use / How to grill / Failure behavior), with one required addition:
an explicit `## Bounded judgment` section per `docs/icm/convention.md` §6.1
and ADR 0013 decision 4. Proposed text, synthesized from the six behavior
units above and offered here as illustrative content for the independent
reviewer and Captain's reconcile-and-publish step — this record does not
itself edit the live package (decision 6/§4.9: a producer does not
self-promote):

```markdown
## Bounded judgment

Apply `@@bounded-judgment`.

### This skill may decide
- Which branch of the decision tree to walk next, and in what order,
  provided every dependent decision is resolved before decisions that
  depend on it (BU-R2-GRILL-02).
- Whether a given fact is discoverable by exploring the environment
  (filesystem, `sgt doctor`, tests, docs, `--help`, any available tool)
  rather than a genuine decision that must go to the user
  (BU-R2-GRILL-04).
- What recommended answer to offer alongside each question
  (BU-R2-GRILL-02).

### This skill must ask the user
- Every genuine decision identified by the interview, one at a time,
  waiting for the answer before asking the next (BU-R2-GRILL-03,
  BU-R2-GRILL-04).
- Explicit confirmation that shared understanding has been reached,
  before acting on any of the interview's conclusions (BU-R2-GRILL-05).

### This skill must not do
- Run via `sgt run` or any durable Work dispatch — R-NS-6 places this
  entirely inside the current conversation (BU-R2-GRILL-01).
- Ask more than one question at a time (BU-R2-GRILL-03).
- Act on the plan/decision/idea — implementation, `sgt run` submission,
  or a file edit driven by the interview's conclusions — before the
  user's explicit confirmation (BU-R2-GRILL-05).
- Present an unconfirmed, harness-degraded best guess as a reached
  shared understanding; say so plainly instead (BU-R2-GRILL-06).

### Durable handoff
None. This skill produces no promotable artifact of its own; a confirmed
understanding is consumed directly in the same session (e.g. to shape a
subsequent `sgt run` submission), not written to a Work surface.
```

## Inputs and outputs

No formal Layer-2 Inputs table applies — `SKILL.md` is a Captain skill, not
a workflow stage, and the convention's Inputs-table rule (`record-shapes.md`
§1a) governs stage `CONTEXT.md` files, not skills. The package's only cited
external evidence is `docs/environments/cerberus.md` (measured fact backing
the Failure-behavior section), referenced in prose, not through `@@name`.

Outputs: none. The skill is read-only with respect to the repository and
produces no Layer-4 artifact; its entire effect is the live conversation and
whatever the user does with the confirmed understanding afterward. Decision
6/§6.2's promotable-effects scope therefore does not require independent
review of any *run* of this skill — only of a change to the package's own
content (the proposed `## Bounded judgment` addition above), before that
change lands.

## Review and promotion policy

The proposed `## Bounded judgment` addition is a promotable content change
to an admitted skill and follows the standard chain (proposal §9.5): this
draft record → independent adversarial review (ICM-R2's separate reviewer
step) → remediation or acceptance → explicit promotion by editing
`skills/grilling/SKILL.md` in place (no rehome, no new directory — the file
already is the admitted surface). This producer pass performs no such edit.

## Alternatives considered

- **REHOME** (back to a workflow, or elsewhere): rejected. The 2026-08-12
  rehome's own dogfood evidence (zero `needs_input` pauses in 2/2 runs) is
  reconfirmed by this pass's reading of the current file — nothing in the
  current content depends on durable execution, retry, checkpoints, or
  cross-stage handoff. PL-4/PL-5 do not hold; PL-2 still does.
- **SPLIT**: rejected. The six substantive behavior units form one coherent
  interview technique with a single durable outcome (confirmed shared
  understanding); none is independently triggerable or separately owned.
- **HARVEST**: rejected. No other package currently needs any of these units
  in isolation; `grill-with-docs` already delegates to this package wholesale
  rather than duplicating its content (`docs/icm/re-homing-record-2026-08-12.md`
  row 33), which is the correct reuse shape, not a harvest candidate.
- **FOLD**: rejected for the package as a whole — grilling is a
  independently triggerable Captain skill with its own trigger phrase, not
  subordinate content that belongs inside another package's context.
  (The *missing section* is itself folded in as a package amendment, which
  is the ordinary maintenance path for an admitted skill, not a
  disposition-level FOLD.)
- **RETIRE**: rejected. The 2026-08-12 rehome already retired the prior
  workflow identity; the current skill is live, correctly placed, and still
  the only mechanism satisfying R-NS-6 for this behavior.
- **STAND with no changes** (matching the dispatch hint literally): rejected
  after inspection — the file has no `## Bounded judgment` heading, so
  "verify completeness" was not answerable as asked; the actual finding is
  an absence, not a completeness defect, and is recorded as such (unit
  BU-R2-GRILL-07) rather than silently upgraded to match the hint's framing.

## Final disposition

STAND

The package's surface and identity (`skills/grilling/SKILL.md`, a Captain
skill) are correct and remain unchanged. One content amendment is required
before the package fully satisfies current doctrine: adding an explicit
`## Bounded judgment` section (draft text above). This is ordinary
maintenance of an admitted package under STAND, not a rewrite requiring a
`draft/` mirror per the dispatch instructions' REHOME/SPLIT/HARVEST trigger.

## Validation evidence

- Direct filesystem check this pass: `.sergeant/workflows/grilling/` does
  not exist; `skills/grilling/SKILL.md` is the package's only file.
  `grill-with-docs` (`skills/grill-with-docs/SKILL.md`) delegates to this
  package rather than duplicating it, confirmed by
  `docs/icm/re-homing-record-2026-08-12.md` rows 32-33.
- `docs/gauntlet/promoted-provenance/grilling.md`: original workflow-era
  behavior units (`BU-P3-005` through `BU-P3-009`) and the G5 engine-gap
  resolution record for the retired workflow's `needs_input` mechanics —
  read for provenance; superseded as *current* behavior by the skill's own
  text, which is what this record classifies.
- `docs/environments/cerberus.md`: Claude CLI row, this host — non-
  interactive turns cannot hold a mid-turn pause open, the measured fact
  the package's own "Failure behavior" section is built on.
- No execution-evidence gap found beyond the 2026-08-12 dogfood measurement
  already on record (2/2 runs, zero `needs_input`); this pass adds no new
  execution trace, only a content/structure re-read against the newly
  ratified doctrine.
