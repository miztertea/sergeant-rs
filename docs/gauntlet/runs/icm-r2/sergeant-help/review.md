# Independent adversarial review: sergeant-help package adjudication

Reviewer position: independent of the producer draft
(`docs/gauntlet/runs/icm-r2/sergeant-help/adjudication-draft.md`). Fresh
execution, inputs limited to that draft, the live package
(`skills/sergeant-help/SKILL.md`), and the cited governing sources
(`docs/adr/0013-icm-r0-owner-rulings.md`;
`reference/proposal-icm-r-procedure-authority.md` §5, §6, §8.10-8.11;
`docs/icm/record-shapes.md` §6; `docs/icm/convention.md` §6; `AGENTS.md`;
`LESSONS.md`; `docs/gauntlet/promoted-provenance/sergeant-help.md`;
`docs/icm/retriage-2026-08-11.md`). Review-only contract, no edit authority
over the draft or the live package (`docs/adr/0013` decision 7,
`convention.md` §6.3).

Re-derivation was performed against the actual package content
(`skills/sergeant-help/SKILL.md`, read in full) and the actual current
routing table (`AGENTS.md` lines 39-58), not from the producer's own
citations, then checked against those citations for fidelity.

## Behavior-unit dispositions

### BU-SH-01 -- verdict: NEEDS-REVISION

Re-derivation: `SKILL.md`'s front matter description + "When to use" (lines
3-4, 15-19) and `AGENTS.md`'s routing-table row + explicit rule (lines 46,
57) do independently *state* the same trigger, and PL-2/STAND is correct —
this is squarely Captain-driven, pre/in/post-work-agnostic dialogue
(§5.4's discriminator holds). But the draft's J boundary text ("two
independent sources... agree") overstates the independence: both surfaces
were touched in the same 2026-08-11 re-homing round
(`docs/icm/retriage-2026-08-11.md` line 57 predates and motivates the
current `SKILL.md`), and neither file declares itself canonical for the
trigger wording if the two ever diverge. This is exactly the "duplicated or
drift-prone content" class named in §8.11: the same trigger condition is
maintained in two places with no cross-reference stating which one wins.
Disposition (STAND, PL-2) is unaffected; the record should either have
`SKILL.md`'s "When to use" section state it mirrors `AGENTS.md`'s routing
row (single source of truth), or drop the "two independent sources"
framing since it isn't structurally guaranteed to stay true.

### BU-SH-02 -- verdict: NEEDS-REVISION

Re-derivation: the behavior (hand off to `estate-navigation`/`sgt run`
rather than substitute for actually doing the thing — `SKILL.md` lines
21-24) is real and correctly PL-2/STAND. The J boundary is mis-rung,
though: "governed by ... `AGENTS.md`'s routing table" is offered as J5
("governing constraint... binding law, safety policy, repository doctrine,
authority boundary" — §6.2), but a routing table that assigns ownership is
closer to §6.4's J3 ("an accepted upstream artifact... settles the
question... reuse it and cite the artifact") than to a J5 prohibition. The
routing table doesn't *forbid* an action the way "do not expose a secret"
does; it settles which package owns a class of question. Same issue
recurs at BU-SH-10c/BU-SH-10d below — this looks like a systematic
J5-vs-J3 conflation across the routing-boundary units, not an isolated
slip. Recommend re-rung to J3.

### BU-SH-03 -- verdict: CONFIRMED

Re-derivation against `SKILL.md` lines 26-51 (the documentation-map table
and step 1-2 of the query procedure): classifying a question against a
named table and selecting the primary document to read is exactly §6.5's
J2 example "select which evidence sources to inspect." PL-2/STAND holds;
no adjacent rung fits better.

### BU-SH-04 -- verdict: CONFIRMED

Re-derivation against `SKILL.md` lines 52-56 (the `rg` invocation): local,
reversible search mechanics that don't touch scope, authority, or another
actor's contract — squarely §6.6 J1. Citation and rung both hold.

### BU-SH-05 -- verdict: CONFIRMED

Re-derivation against `SKILL.md` lines 58-64 and `LESSONS.md` L1 ("Measure
the Claude CLI, never trust its docs or its exit codes" — verified at
`LESSONS.md` line 441, generalizing directly to "run `--help`/observe
rather than assume syntax"). L1 is genuinely governing, repo-wide doctrine,
independent of this one skill — correctly J5, and correctly still PL-2
(the *behavior* of running `--help` inside a doc-lookup procedure is local
to this skill even though the underlying principle it enacts is J5).

### BU-SH-06 -- verdict: CONFIRMED

Re-derivation against `SKILL.md` lines 65-66, 79-87 (the fixed
`Answer/Command/Requires/Verify/Docs` format): the shape is fixed by the
package; filling each field's content per query is delegated judgment
within a named bound — §6.5 J2, matching the "choose among ... designs that
all preserve the bound contract" example. PL-2/STAND holds.

### BU-SH-07 -- verdict: CONFIRMED

Re-derivation against `SKILL.md` lines 67-75: the five-tier precedence
order is stated once and reused per query rather than re-litigated —
exactly §6.4 J3's "reuse it... do not reopen settled intent merely because
another choice is possible." Correctly distinguished from BU-SH-05/08's J5
citations (this is the *skill's own* settled ordering, not an externally
binding rule) and from J2 (nothing is being chosen here, only applied).

### BU-SH-08 -- verdict: CONFIRMED

Re-derivation against `SKILL.md` lines 76-77 and `LESSONS.md` L1: the
anti-fabrication instruction ("do not invent a command, flag, state
transition, or safety guarantee") is a direct restatement of governing,
repo-wide doctrine, not a local judgment call. J5 citation holds.

### BU-SH-09 -- verdict: NEEDS-REVISION

Re-derivation against `SKILL.md` lines 89-92 and the cited
`AGENTS.md` "## Guardrails" section (lines 189-207, read in full): every
Guardrails bullet governs actually *performing* a destructive operation or
extending standing authorization to one — none of them addresses what
prose examples may appear inside a documentation answer. The behavior here
("keep destructive operations out of examples unless the doc itself
requires confirmation and the user asked") is this skill's own editorial
policy for answer content, not an externally binding prohibition on an
action this skill takes. That makes the J5 citation a stretch — this reads
more like §6.5 J2 (a named, bounded delegation about what an answer may
include) or, if treated as risk-reducing but still local, §6.6 J1. It does
not change the unit's PL-2/STAND disposition, but the rung and its
rationale ("governed by AGENTS.md's Guardrails policy") should be
corrected or the citation should point at a passage that actually supports
a J5 reading.

### BU-SH-10a -- verdict: CONFIRMED

Re-derivation against `SKILL.md` line 98 and the Failure-behavior table:
"report the expected path and stop before guessing" is a named, bounded
response to one specific failure mode — §6.5 J2. Consistent with
BU-SH-08's governing anti-fabrication constraint as the draft notes.

### BU-SH-10b -- verdict: CONFIRMED

Re-derivation against `SKILL.md` line 99: "report the mismatch, trust
measured behavior, name the stale doc as a fix candidate" is again a
named delegated response — J2, correctly distinguished from BU-SH-05's J5
(the *principle* that measured behavior wins is J5; *what to do* when a
mismatch is found is this skill's own J2 procedure).

### BU-SH-10c -- verdict: NEEDS-REVISION

Same rung concern as BU-SH-02: the behavior (load `estate-navigation`
rather than answer from memory — `SKILL.md` line 100) is real and
correctly PL-2/STAND, but citing "`AGENTS.md`'s routing table" as a J5
governing constraint again looks like J3 (a settled record this skill
reuses) rather than a binding prohibition. Recommend re-rung to J3,
consistent with BU-SH-02.

### BU-SH-10d -- verdict: NEEDS-REVISION

Same rung concern again: "hand off to the standard workflow loop / `sgt
run`... stays strictly read-only" (`SKILL.md` line 101) is PL-2/STAND-
correct, but "same routing boundary as BU-SH-02/10c" inherits the same
J5-should-be-J3 issue rather than resolving it. Recommend re-rung to J3.

## Additional findings outside the per-unit table

### Citation error: `docs/icm/convention.md` §7.4 does not exist

The "Surviving package design" section's gap-finding cites "`docs/icm/
convention.md` §6.1 / §7.4 (required sections)." `convention.md` has no
§7 at all (confirmed directly: `grep -n '^## 7\|^### 7'` returns nothing;
its highest section is `## 6`). §7.4 ("Skill-level bounded judgment") is a
real section, but it lives in `reference/proposal-icm-r-procedure-
authority.md`, not `convention.md`. This is a source-fidelity slip per
§8.11's first checklist item — the underlying finding (the missing
`## Bounded judgment` section) is itself CONFIRMED real (verified directly:
`skills/sergeant-help/SKILL.md` has no such heading), but the citation
needs to read "`convention.md` §6.1; proposal §7.4" rather than attributing
both section numbers to `convention.md`.

### Citation mismatch: "Current trigger and outcome" outcome citation

The draft's "Outcome" paragraph (draft lines 15-16) cites `SKILL.md` lines
21-24 and 101 for the claim that the skill produces "a formatted answer...
grounded in cited repository-relative document paths, or an explicit
statement that the behavior is undocumented... No file is written, no Work
is created." Lines 21-24 are the "do not use this in place of actually
doing the thing" hand-off instruction (BU-SH-02's own source), which
doesn't support the answer-format/no-file-written claim at all — the
actual support is the "Answer format" section (`SKILL.md` lines 79-87) and
the Failure-behavior table (94-101). Minor, but a real source-fidelity
miss: the citation should point at 79-87 (and optionally 94-101), not
21-24.

## Checklist coverage (§8.11)

- **Source fidelity:** two citation errors found and corrected above
  (§7.4 misattribution, Outcome-paragraph line range). All other citations
  checked directly against the cited files and hold.
- **Rung order (PL and J):** PL rungs all independently re-derived and
  confirmed at PL-2 (no unit satisfies PL-0/PL-1/PL-3/PL-4 on direct
  application of §5.2-§5.6 to the current file). J rungs: four units
  (BU-SH-02, 09, 10c, 10d) have a J5 citation that should be J3 or J2 on
  closer reading of the cited source; disposition unaffected, rung
  citations should be corrected before promotion.
- **Captain/workflow boundary:** confirmed correct. `.sergeant/workflows/
  sergeant-help` does not exist on disk (independently reconfirmed:
  `ls .sergeant/workflows/ | grep -i help` returns nothing); the PL-4
  discriminator (§5.6: "a result meaningful independent of the original
  conversation continuing") fails for a live doc-lookup Q&A, so PL-4/
  REHOME-to-workflow is correctly rejected.
- **Stage/helper boundary:** not applicable — single-file Captain skill,
  no stage split claimed or warranted.
- **Authority grants and missing J0 cases:** re-derived independently by
  walking `SKILL.md`'s own failure modes (missing primary doc, stale doc,
  needs estate state, needs mutation, contradictory sources) against every
  J0 trigger in §6.7. None of them changes scope, security, destructive
  effects, or promotion — each has a named report-or-hand-off response
  already in the package. The draft's "must ask the user: Nothing" claim
  holds.
- **Package identity/naming:** confirmed. No collision; name matches the
  live directory and every reference to it (`AGENTS.md`, `.sergeant/
  index.md`, `docs/gauntlet/promoted-provenance/sergeant-help.md`).
- **Duplicated or drift-prone content:** one real instance found (BU-SH-01
  above) not surfaced by the draft — the trigger condition lives in two
  files with no stated single source of truth.
- **False pairing assumptions:** related to the above — the draft's "two
  independent sources... agree, no competing routing entry" framing for
  the driver/admission-boundary answer (draft lines 17-23) treats
  `AGENTS.md` and `SKILL.md` as independent corroboration when they were
  authored/maintained together at the same re-homing event. The underlying
  conclusion (driver = Captain, admission boundary = always) is still
  correct on the merits — `AGENTS.md`'s routing table has no competing row
  for this trigger, and `SKILL.md`'s own content depends on no Work
  lifecycle position — but "independent" overstates the evidence.
- **Unjustified engine gaps:** none present; none claimed. Correctly
  absent — a read-only doc-lookup procedure has no PL-7 candidate content.

## Overall verdict

**Final disposition: STAND — CONFIRMED.**

Every behavior unit independently re-derives to PL-2/STAND against the
actual current `skills/sergeant-help/SKILL.md`; no unit belongs at a
different placement rung, and no alternative disposition (REHOME/SPLIT/
HARVEST/ABSORBED/FOLD/RETIRE) survives re-checking the draft's own
"Alternatives considered" section against §5.2-§5.10 directly. The missing
`## Bounded judgment` section is a real, confirmed gap that must be added
at reconcile-and-publish (§8.12) before this record is treated as settled
— consistent with the draft's own recommendation and the promotion-chain
rule that this producer draft does not self-apply it.

Before reconcile-and-publish, the record should also: (1) fix the §7.4
misattribution (cite the proposal, not `convention.md`, for the Bounded-
judgment template); (2) fix the Outcome paragraph's citation to point at
`SKILL.md` lines 79-87/94-101 instead of 21-24; (3) re-rung BU-SH-02,
BU-SH-09, BU-SH-10c, and BU-SH-10d's J-boundary citations (J5 → J3 for the
routing-table-governed units, J5 → J2/J1 for BU-SH-09's example-content
policy); and (4) either state explicitly that `SKILL.md`'s trigger text
mirrors `AGENTS.md`'s routing row as its single source of truth, or drop
the "two independent sources" framing in the driver/admission-boundary
section, per the BU-SH-01 and false-pairing findings above. None of these
four corrections changes the package's surviving design or its STAND
disposition; they are citation- and rung-precision fixes, not structural
findings.
