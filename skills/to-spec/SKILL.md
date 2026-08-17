---
name: to-spec
description: Turn the current conversation's plan/design into a published spec ticket — no interview, just synthesis of what's already been discussed, plus repository exploration. Use when a design needs to become a spec-shaped ticket before implementation.
edition: 0.1.0
---

Provenance for this skill's citation record lives in
`sergeant-rs-workspace`'s `knowledge/evidence/provenance/skills.md`.

Ported from `.sergeant/workflows/to-spec` (N1 candidate W31), which
retires: this package's own defining behavior — synthesize from "the
current conversation," never by interview — names a dependency on live
dialogue that a dispatched Work cannot receive (`sergeant-rs-workspace/knowledge/evidence/gauntlet/runs/
icm-r3/to-spec/adjudication-draft.md`'s "Driver and admission boundary").
The Placement Ladder's PL-4 test requires a workflow to produce "a result
that is meaningful independent of the original conversation continuing"
(`sergeant-rs-workspace/knowledge/evidence/reference/proposal-icm-r-procedure-authority.md` §5.6); this package's
whole point is the opposite — the conversation supplies its content. That
places it at PL-2, alongside `sergeant-rs-workspace/knowledge/evidence/reference/proposal-icm-r-procedure-
authority.md` §5.4's own example of Captain-skill behavior: "turns user
conversation into a bounded submission."

## When to use

The user wants their plan or design turned into a spec-shaped ticket
before implementation, or invokes a "spec this" / "write this up as a
spec" trigger phrase. Run this directly in the current conversation —
never via `sgt run`: there is no admitted-Work input that could carry
"what we've already discussed" into a fresh dispatched execution.

## How to write the spec

1. **Synthesize, don't interview.** Do not ask the user to re-explain the
   plan. Write the spec from what has already been discussed in this
   conversation, plus your own codebase exploration. If you haven't
   already explored the repository to understand current state, do that
   first. Use the project's domain glossary vocabulary throughout, and
   respect any ADRs in the area you're touching.
2. **Sketch the test seams before drafting the implementation section**,
   using `@@tdd`'s definition of a seam (the public boundary you test at
   — never against internals). Prefer an existing seam to a new one, and
   the highest seam possible. Aim for as few new seams as possible —
   ideally exactly one. Then confirm with the user, one exchange, that
   the proposed seams match their expectations, before finalizing the
   spec. **This is a distinct step from `@@tdd`'s own per-cycle seam
   confirmation** (ICM-R3 note, resolving a duplication/drift risk the
   independent reviewer flagged): here, seams are sketched once, at
   spec-writing time, to shape the Testing Decisions section below;
   `@@tdd`'s confirmation happens per red-green cycle, later, during
   actual implementation. The two are related but not the same gate —
   this skill does not duplicate `@@tdd`'s own content, it names the same
   underlying concept (a seam) at an earlier point in the process.
3. **Write the spec on the template below**, then publish it to the
   project's issue tracker and apply the ready-for-agent triage label.

**Before step 3, resolve which tracker/label convention actually governs
in this repository — do not guess.** Two things are true at once here and
this skill does not pick between them on your behalf:

- The `ready-for-agent` label name comes from this repository's own
  `triage` workflow, whose `50-apply-outcome` stage requires posting a
  *structured agent brief comment* to reach that outcome
  (`.sergeant/workflows/triage/50-apply-outcome/CONTEXT.md`) — not merely
  applying the label.
- This skill's own upstream source only says "apply the label... no need
  for additional triage," and separately assumes an issue-tracker/label
  vocabulary "provided to you" by a setup skill that does not exist in
  this repository (no `matt-pocock-skills` equivalent was ever ported
  here).

If the user hasn't told you which convention applies, ask before applying
`ready-for-agent`: either post the structured agent brief `triage`
requires, or get the user's explicit go-ahead to apply the label alone.
Do not silently pick one — a dispatched agent that later picks up this
ticket may rely on the agent brief being present.

<spec-template>

## Problem Statement

The problem that the user is facing, from the user's perspective.

## Solution

The solution to the problem, from the user's perspective.

## User Stories

A LONG, numbered list of user stories. Each user story should be in the
format of:

1. As an \<actor\>, I want a \<feature\>, so that \<benefit\>

<user-story-example>
1. As a mobile bank customer, I want to see balance on my accounts, so that I can make better informed decisions about my spending
</user-story-example>

This list of user stories should be extremely extensive and cover all
aspects of the feature.

## Implementation Decisions

A list of implementation decisions that were made. This can include:

- The modules that will be built/modified
- The interfaces of those modules that will be modified
- Technical clarifications from the developer
- Architectural decisions
- Schema changes
- API contracts
- Specific interactions

Do NOT include specific file paths or code snippets. They may end up
being outdated very quickly.

Exception: if a prototype produced a snippet that encodes a decision more
precisely than prose can (state machine, reducer, schema, type shape),
inline it within the relevant decision and note briefly that it came from
a prototype. Trim to the decision-rich parts — not a working demo, just
the important bits.

## Testing Decisions

A list of testing decisions that were made. Include:

- A description of what makes a good test (only test external behavior,
  not implementation details)
- Which modules will be tested
- Prior art for the tests (i.e. similar types of tests in the codebase)

## Out of Scope

A description of the things that are out of scope for this spec.

## Further Notes

Any further notes about the feature.

</spec-template>

## Bounded judgment

Apply `@@bounded-judgment`.

### This skill may decide
- Whether repository exploration is already sufficient or needs to happen
  first, before drafting.
- Which existing seam is highest/most reusable, and how many new seams
  (if any) are genuinely unavoidable.
- Wording, structure, and level of detail within each spec template
  section, provided every required section is present.

### This skill must ask the user
- Confirmation that the proposed test-seam plan matches their
  expectations, before finalizing the spec — one exchange, not an
  interview about the whole design.
- Which `ready-for-agent` convention governs (label alone vs. a
  structured agent brief per the `triage` workflow) whenever that hasn't
  already been established, before publishing with that label.

### This skill must not do
- Interview the user about the plan/design itself — synthesize from the
  conversation and codebase exploration instead.
- Run via `sgt run` or any durable Work dispatch — the entire procedure
  depends on this conversation's own content, which a dispatched
  execution cannot receive.
- Apply the `ready-for-agent` label without first resolving which
  tracker/label convention governs, when that isn't already settled.
- Invent tracker or label vocabulary that doesn't exist in this
  repository's own conventions merely because the upstream source assumed
  some external setup skill would have supplied it.

### Durable handoff
The published spec ticket is the artifact of record — that publication
*is* the durable handoff, not a Work-branch file. This skill does not
itself invoke `sgt run`; if the resulting ticket is meant to become
dispatched Work, that is a separate, later decision through the normal
task-intake path.

## Failure behavior

If this invocation has no live human who will send the next message —
the harness is running headless/unattended, not mid-conversation with a
person — this skill cannot perform its own defining behavior: there is
no "current conversation" to synthesize from, and the one-exchange seam
confirmation in step 2 has no one to confirm with. Say so plainly and
stop rather than fabricating a plan/design that was never actually
discussed, or silently skipping the seam-confirmation exchange and
finalizing an unconfirmed spec.
