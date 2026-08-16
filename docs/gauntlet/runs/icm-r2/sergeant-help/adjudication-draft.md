# Package adjudication: sergeant-help

Record shape: `docs/icm/record-shapes.md` §6. Method: `reference/proposal-icm-r-procedure-authority.md` §8. Pilot corpus authority: `docs/adr/0013-icm-r0-owner-rulings.md` decisions 8-9.

Current package: `skills/sergeant-help/SKILL.md` (single file, no delegated stage files, no `@@`-referenced shared context). Historical predecessor: `.sergeant/workflows/sergeant-help` (retired; see `docs/gauntlet/promoted-provenance/sergeant-help.md` and the SKILL.md's own "Ported from" note) — that workflow directory no longer exists on disk, confirmed by direct listing of `.sergeant/workflows/`.

## Original intention

Answer a Sergeant usage/setup/troubleshooting question strictly from repository-owned documentation, read-only, with a fixed source-precedence order, never inventing behavior. Originally a published workflow (N1 reference-corpus candidate W4); re-homed to a Captain skill at the 2026-08-11 execution-surface test (`docs/icm/retriage-2026-08-11.md`) on the finding that a doc lookup needs no worktree and no durable Work state.

## Current trigger and outcome

**Trigger:** the user asks what Sergeant is, how to install/configure/use it, where workflows or skills come from, how a specific `sgt` command or flag works, or how to diagnose a Sergeant error (`skills/sergeant-help/SKILL.md` lines 3-4, 17-19; corroborated independently by `AGENTS.md`'s own routing table, line 46, and its explicit "Doc/help questions always route to `sergeant-help`" rule, line 57).

**Outcome:** a formatted answer (`Answer/Command/Requires/Verify/Docs`) grounded in cited repository-relative document paths, or an explicit statement that the behavior is undocumented/unmeasured/contradictory rather than an invented answer. No file is written, no Work is created, and no repository state changes as a result of running this skill — it is strictly read-only (SKILL.md lines 21-24, 101).

## Driver and admission boundary

**Driver:** Captain (the interactive harness). This is loaded and run directly in the current harness session, per `AGENTS.md`'s routing table row and `skills/sergeant-help/SKILL.md`'s own front matter — there is no dispatched `sgt run`, no worktree, and no Work admission anywhere in this package's procedure.

**Admission boundary:** `always`. Unlike a pre-work/in-work/post-work Captain behavior gated on Work lifecycle position, this skill answers a doc/help question whenever one is asked — before a Work is admitted, while one is being discussed, or with no Work in play at all. Nothing in the package's content depends on Work admission state.

This answer is unambiguous: two independent sources (the skill's own front matter/body, and `AGENTS.md`'s routing table plus its explicit "always route to `sergeant-help`" rule) agree, and no competing routing entry claims this trigger.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-SH-01 | Load this skill when the user asks a read-only Sergeant usage/install/config/troubleshooting question (`SKILL.md` lines 3-4, 17-19; `AGENTS.md` line 46, 57) | PL-2 | J5 — `AGENTS.md`'s own routing rule ("Doc/help questions always route to `sergeant-help`") governs; not a locally reversible choice | STAND | `skills/sergeant-help/SKILL.md` |
| BU-SH-02 | Do not use this skill in place of actually doing the thing; once the user has asked for setup, repo registration, or work submission, hand off to `estate-navigation` or `sgt run` (`SKILL.md` lines 21-24) | PL-2 | J5 — governed by R-NS-6 ("execution ≠ dialogue") and `AGENTS.md`'s routing table, not a discretionary call by this skill | STAND | `skills/sergeant-help/SKILL.md` |
| BU-SH-03 | Classify the question against the fixed documentation map, read the primary document before searching broadly (`SKILL.md` lines 26-51) | PL-2 | J2 — the skill names and delegates use of its own documentation map | STAND | `skills/sergeant-help/SKILL.md` |
| BU-SH-04 | For terms not resolved by the primary document, search repository documentation with a named `rg` invocation (`SKILL.md` lines 52-56) | PL-2 | J1 — local, reversible search mechanics; does not change scope or authority | STAND | `skills/sergeant-help/SKILL.md` |
| BU-SH-05 | For flag/argument questions, run `sgt <command> --help` rather than assume syntax (`SKILL.md` lines 58-64) | PL-2 | J5 — matches `LESSONS.md` L1 ("the Claude adapter's behavior is measured, never assumed from docs"), a governing doctrine, not a local option | STAND | `skills/sergeant-help/SKILL.md` |
| BU-SH-06 | Answer with the exact command, required preconditions, expected evidence, and repository-relative doc links, in the fixed `Answer/Command/Requires/Verify/Docs` field format (`SKILL.md` lines 65-66, 79-87) | PL-2 | J2 — the skill delegates the answer-shape decision within a named format | STAND | `skills/sergeant-help/SKILL.md` |
| BU-SH-07 | When sources disagree, apply the fixed precedence order (measured `--help`/observed behavior, then `AGENTS.md`, then the trigger-loaded skill/workflow's own doc, then `MVP-1.md`, then `README.md`/`DEVELOPMENT.md`) (`SKILL.md` lines 67-75) | PL-2 | J3 — reuses an already-settled ordering over other settled documents; not reopened per query | STAND | `skills/sergeant-help/SKILL.md` |
| BU-SH-08 | State when a behavior is undocumented, unmeasured, or contradictory; never invent a command, flag, state transition, or safety guarantee (`SKILL.md` lines 76-77) | PL-2 | J5 — anti-fabrication is governing doctrine (`LESSONS.md` L1), not a local judgment call | STAND | `skills/sergeant-help/SKILL.md` |
| BU-SH-09 | Keep destructive operations out of examples unless the documentation itself requires confirmation for them and the user explicitly requested them (`SKILL.md` lines 89-92) | PL-2 | J5 — governed by `AGENTS.md`'s Guardrails policy | STAND | `skills/sergeant-help/SKILL.md` |
| BU-SH-10a | If the primary document for a question is missing, report its expected path and stop before guessing (`SKILL.md` line 98) | PL-2 | J2 — the skill delegates a named report-and-stop response to this specific failure mode | STAND | `skills/sergeant-help/SKILL.md` |
| BU-SH-10b | If observed command behavior differs from documentation, report the mismatch, trust the measured behavior, and name the stale doc as a fix candidate rather than silently papering over it (`SKILL.md` line 99) | PL-2 | J2 — named delegated response, consistent with BU-SH-05/BU-SH-08's governing anti-fabrication constraint | STAND | `skills/sergeant-help/SKILL.md` |
| BU-SH-10c | If a question actually requires estate/repo state, load `estate-navigation` rather than answering from memory (`SKILL.md` line 100) | PL-2 | J5 — same routing boundary as BU-SH-02, governed by `AGENTS.md`'s routing table | STAND | `skills/sergeant-help/SKILL.md` |
| BU-SH-10d | If a question actually requires submitting or mutating work, hand off to the standard workflow loop / `sgt run`; this skill stays strictly read-only (`SKILL.md` line 101) | PL-2 | J5 — same routing boundary as BU-SH-02/BU-SH-10c | STAND | `skills/sergeant-help/SKILL.md` |

No unit in this package classifies below PL-2 (no deterministic-mechanism or engine-gap candidate found), and none is a candidate for REHOME/SPLIT/HARVEST — every unit is internal specialization of one Captain skill with one driver and one admission boundary. `alternatives_considered` for each unit: PL-1 (stable invariant) was considered and rejected for BU-SH-01/02/05/08/09/10c/10d because, although they cite governing (J5) constraints, the *behaviors themselves* are specific to this skill's doc-lookup procedure, not broadly-applicable rules independent of one trigger (§5.3's own test); PL-3 (actor skill/shared method) was considered and rejected across the whole package because no other package currently reuses this documentation-map/precedence procedure — it owns one complete Captain interaction end to end (§5.5's discriminator from PL-2), not a technique invoked by another package.

## Surviving package design

Unchanged: one Captain skill, `skills/sergeant-help/SKILL.md`, loaded directly by the interactive harness on trigger match. No stage split, no new workflow, no shared-context extraction — no second consumer of this content exists yet (§5.10's shared/local modifier does not apply).

**Gap found against `docs/icm/convention.md` §6.1 / §7.4 (required sections):** the current `skills/sergeant-help/SKILL.md` carries no `## Bounded judgment` section. Decision 4 of `docs/adr/0013-icm-r0-owner-rulings.md` requires this section on every actor stage "always... even when it is only 'inherits...unchanged'"; §6.1 of `convention.md` extends the same requirement to every Captain skill, adapted to its driver. This package predates that requirement (it was last touched at the 2026-08-11 re-homing round, before ICM-R0/R1). The package-specific hint that prompted this pilot pass ("verify it needs no durable handoff section... and its driver/admission-boundary answer is unambiguous") is correct on both of the two things it named, but did not name this gap — it is a real, distinct finding surfaced by applying §8.7 (Authority classification) directly to the current file rather than trusting the hint.

Recommended section content for the reconcile-and-publish step (§8.12), synthesized from the Behavior-unit dispositions table above — not applied to the live file by this producer step, per the promotion-chain rule (§9.5, `convention.md` §6.2/6.3: a producer does not self-promote; this record is the draft input to independent review):

```markdown
## Bounded judgment

### This skill may decide
- Which primary document answers a classified question (BU-SH-03).
- How to phrase the `Answer/Command/Requires/Verify/Docs` response (BU-SH-06).
- How to respond to a missing primary document or an observed doc/behavior
  mismatch: report and stop, or report and flag, rather than guess
  (BU-SH-10a, BU-SH-10b).

### This skill must ask the user
- Nothing — this skill is read-only and does not itself reach J0; every
  unresolved condition it can encounter (missing doc, stale doc, question
  needing estate state or mutation) routes to a stated report or a named
  hand-off (BU-SH-10a-d), not a live question of its own.

### This skill must not do
- Answer from memory when a primary document is missing (BU-SH-10a).
- Assume `sgt` command/flag syntax instead of running `--help` (BU-SH-05).
- Invent a command, flag, state transition, or safety guarantee (BU-SH-08).
- Include a destructive-operation example unless the documentation requires
  confirmation for it and the user explicitly requested it (BU-SH-09).
- Continue answering once the question actually requires estate/repo state
  or work submission — hand off instead (BU-SH-02, BU-SH-10c, BU-SH-10d).

### Durable handoff
None. This skill produces no promotable artifact; the only "handoff" is
routing to a different skill (`estate-navigation`) or `sgt run` when the
user's need turns out not to be read-only, at which point this skill's own
procedure ends (BU-SH-02, BU-SH-10c, BU-SH-10d).
```

## Inputs and outputs

**Inputs:** the user's live question (conversational, not a file); the fixed documentation map table inside `SKILL.md` itself; the repository documents that table names (`README.md`, `NORTH-STAR.md`, `AGENTS.md`, `docs/icm/convention.md`, `.sergeant/index.md`, `docs/environments/<host>.md`, `docs/DEVELOPMENT.md`, `docs/gauntlet/contracts/MVP-1.md`, `docs/gauntlet/notes/estate-manifest-design-2026-08-11.md`, `GAUNTLET.md`, `LESSONS.md`, `docs/gauntlet/contracts/<milestone>.md`); the running `sgt` binary via `sgt <command> --help`.

**Outputs:** a live conversational answer in the fixed format. No file artifact, no Work, no journal entry. Confirms the package-specific hint: this package needs no durable-handoff section beyond the null case documented above, because it produces no promotable artifact (§9.7's scoping: "any artifact... that will be merged, published, installed, admitted, signed, released, or treated as settled" does not include this skill's live answers).

## Review and promotion policy

This package produces no promotable artifact per se, so §9.7's independent-review requirement does not attach to its ordinary operation. It does attach to *this adjudication record and the recommended `## Bounded judgment` section text above*: per `docs/adr/0013` decision 6/`convention.md` §6.2, the record is promotable review input, and the recommended section text is a proposed edit to an admitted package — neither is self-promoted by this producer step. Both require an independent reviewer position (a fresh execution, explicit inputs limited to this record and the current `SKILL.md`, a review-only contract, no edit authority) before the section is actually added to the live file at reconcile-and-publish (§8.12).

## Alternatives considered

- **REHOME to a workflow.** Rejected: the 2026-08-11 execution-surface test already established, and this pass reconfirms independently, that a doc lookup needs no worktree, no durable Work, and no fresh-execution-per-checkpoint structure (§5.6's PL-4 test fails outright — there is no bounded outcome independent of the live conversation continuing).
- **HARVEST into `estate-navigation`.** Rejected: the two packages have adjacent but distinct triggers (read-only doc lookup vs. resolving/syncing declared repos and groups) and `AGENTS.md`'s routing table already keeps them as separate rows with separate ownership; merging would blur a boundary that is currently unambiguous, not simplify it.
- **FOLD into `AGENTS.md` as a stable invariant (PL-1).** Rejected for the whole package: PL-1 requires a rule that "must apply broadly across many tasks and change rarely, independent of one trigger" (§5.3); this package's content (a documentation map, a precedence order, an answer format) is specific to one trigger's procedure, not a cross-cutting invariant. (Individual J5-cited constraints within it, e.g. BU-SH-08's anti-fabrication rule, are themselves already PL-1 material living in `LESSONS.md`/`AGENTS.md` — this package correctly *cites* them rather than restating them, which is what keeps it at PL-2.)
- **SPLIT the documentation-map/precedence content into a shared `.sergeant/common/contexts/` reference.** Rejected for now: §5.10's shared/local modifier requires two or more consumers of the same contract; no second package currently consumes this documentation map or precedence order. Revisit only if a future package needs the same precedence rule.

## Final disposition
STAND

## Validation evidence

- Direct listing of `.sergeant/workflows/` confirms no `sergeant-help` workflow directory survives — the 2026-08-11 re-homing to `skills/sergeant-help/SKILL.md` is the package's only live form.
- `AGENTS.md` lines 46 and 57 independently corroborate the trigger and driver stated in `SKILL.md`'s own front matter and body — two-source agreement, no competing routing-table row.
- `grep` for `sergeant-help` across `AGENTS.md`, `.sergeant/index.md`, and every `skills/*/SKILL.md` finds exactly the expected references (the routing-table row, the catalog listing, and the package's own file) — no orphaned or duplicate reference.
- The missing `## Bounded judgment` section is a structural gap directly observed in the current file (grep for the heading returns no match), not inferred.
- This is a producer-step record only; it has not yet passed independent adversarial review (§8.11) or reconcile-and-publish (§8.12).
