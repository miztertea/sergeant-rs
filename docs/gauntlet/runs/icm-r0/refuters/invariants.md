# ICM-R0 — invariants refuter

Axis: **invariants** (`docs/gauntlet/contracts/ICM-R0.md`, axis 2).
Critic report under review: `docs/gauntlet/runs/icm-r0/critics/invariants.md`.
Artifact both are grading: `reference/proposal-icm-r-procedure-authority.md`.

This refuter did not write the proposal or the critic report. Each finding
below was re-derived independently against the repository (files, `grep`,
directory listings) rather than trusted from the critic's own quotations,
and argued against as hard as honestly possible before being accepted.

## F1 — §10.4 (ICM-R3 full library reconciliation) vs. the proposal's own hard boundary

**Critic's claim.** §10.4 names "the built-in software-change workflow as a
separate embedded package" as a reconciliation subject; reconciling it per
§7.2/§7.3 means adding an Authority-envelope section and per-stage
Bounded-judgment sections to its `CONTEXT.md` files, which live under
`src/workflows/software-change/` and are compiled in via `include_str!` —
directly contradicting the Executive Summary's hard boundary, §11.2's
`src/**` exclusion, and Acceptance Contract item 33 ("No file under
src/... changes during ICM-R0 through ICM-R4").

**Independent re-derivation.**

```
$ grep -n "software-change\|EMBEDDED_WORKFLOW" src/domain/workflow.rs
58:pub const DEFAULT_WORKFLOW: &str = "software-change";
598:const EMBEDDED_WORKFLOW_TOML: &str = include_str!("../workflows/software-change/workflow.toml");
603:        include_str!("../workflows/software-change/00-prepare/CONTEXT.md"),
607:        include_str!("../workflows/software-change/10-implement/CONTEXT.md"),
611:        include_str!("../workflows/software-change/20-review/CONTEXT.md"),
615:        include_str!("../workflows/software-change/30-close/CONTEXT.md"),
```

`find . -iname "*software-change*" -not -path "./.git/*"` resolves to
exactly one directory on disk: `src/workflows/software-change/`. No
workspace-local or `.sergeant/workflows/` override exists
(`find .sergeant -iname "*software-change*"` returns nothing). `AGENTS.md`
(lines 89–90) confirms this is the actual runtime fallback: "fall back to
the workspace's own `software-change` workflow, then the built-in
default." `src/tui/workflows.rs` independently labels this catalog entry
`"source": "embedded"` and renders it as `(embedded)` / "embedded
fallback" in the TUI, corroborating the critic's framing that this is the
one and only copy, and that it is compiled from `src/`.

The proposal text itself (line 1214, §10.4) reads exactly as quoted:
"the built-in software-change workflow as a separate embedded package,"
and §10.4's outcome list does say "every surviving package carries an
authority envelope" without carving out an exception for this item.
Acceptance item 33 (line 1552) and the Executive Summary boundary (line
48) read exactly as the critic quotes them.

**Attempted refutation.** Two possible outs were checked before accepting
this finding:

1. *Does the proposal's own draft/promotion process (§8.9: "All generated
   or substantially rewritten packages land in a reviewable draft location
   or branch... They do not replace admitted procedure until validation
   completes") save this from being a `src/` change during the ICM-R0–R4
   window?* No — §10.4's own outcome list explicitly includes "draft
   replacements independently reviewed and **promoted**" as part of
   ICM-R3 itself (not deferred to ICM-R5). Since `software-change` has
   exactly one physical location, promoting its draft necessarily means
   writing into `src/workflows/software-change/*/CONTEXT.md` inside the
   ICM-R0–ICM-R4 window the boundary governs. Staging elsewhere only
   delays the violation to the same milestone's own promotion step, it
   does not avoid it.
2. *Does §19 Owner Decision #3 ("Does 'every skill and workflow must be
   validated' include embedded software-change... as a first-class review
   subject?") already flag this as pending, softening the finding?* No —
   that question (line 1651) is scoped to the *validation* claim (one of
   §9.1's five: source/placement/authority/structural/execution-valid),
   not to whether §10.4's reconciliation-and-promotion action on the
   package's actual `CONTEXT.md` files is exempt from the `src/` boundary.
   Even if the owner answers "yes, validate it," that says nothing about
   whether *editing and promoting* its content during ICM-R0–ICM-R4 is
   permitted under Acceptance item 33. The two questions are independent;
   §19 Q3 does not defuse F1.

Both attempted refutations fail. The tension the critic identifies is real
and the proposal's own text does not resolve it.

**Verdict: CONFIRMED.** Severity as reported (error) is appropriate: this
is a direct, unqualified inclusion of `src/`-resident content inside a
workstream whose own Acceptance Contract forbids `src/` changes, not a
hypothetical or indirect dependency.

## F2 — §17 Ponytail Decision Register: ICMR-01 and ICMR-22 rung mismatches

**Critic's claim.** ICMR-01 ("Pin `3a46b87`; do not design against moving
main") cites R1, but R1's actual question ("does this need to exist? no →
skip it") doesn't match a pin-the-revision decision — the proposal's own
§2.2 justification for this practice ("The T-series supplies the closest
house-format precedent. It pins an audit revision... This proposal uses
the same form") is R2's question (reuse, don't rewrite), not R1's.
ICMR-22 ("Admit runtime work only through a separately accepted PL-7
engine-gap contract") cites R7, but is itself a restatement of an existing
discipline (§11.4: "This is the existing engine-gap discipline, applied
rather than merely cited"), which is R2's question, not R7's — and even
charitably read as R7, it never names which of R1–R6 were checked and why
they failed for this specific decision, which the ladder's own logging
convention requires of an R7 entry.

**Independent re-derivation.** Read the Ponytail rung table fresh from
`reference/notes/ideaos-agent-contract.md` (lines 34–54), not from the
critic's paraphrase:

```
R1 | Does this need to exist? | No → skip it (YAGNI)
R2 | Already in this codebase? | Reuse it, don't rewrite
R7 | Only then | The minimum that works
```

and the logging convention: "An `R7` entry must name which lower rungs
were checked and why they failed."

Read §17 fresh from the proposal (lines 1583, 1604):

```
|ICMR-01 |R1  |Pin `3a46b87`; do not design against moving main|
|ICMR-22 |R7  |Admit runtime work only through a separately accepted PL-7 engine-gap contract|
```

and §2.2 (lines 179, "This proposal uses the same form") and §11.4 (line
1331, "This is the existing engine-gap discipline, applied rather than
merely cited").

**Attempted refutation.** Tried three ways to save the citations before
accepting the critic's read:

1. *Is "pin the audit revision" itself a creation decision — did the
   proposal invent the practice of pinning, making R1 ("does a new thing
   need to exist") the right frame?* No — the proposal's own §2.2 frames
   audit-pinning as reuse of the T-series' existing form, not as a novel
   practice being justified for existence. The proposal's own words
   contradict the R1 framing more than they support it.
2. *Is ICMR-22 defensibly R7 because it results in a concrete artifact
   (the PL-7 template) that is itself "the minimum that works"?* The
   template long predates this proposal (§11.4 says it is "the existing
   engine-gap discipline") — the decision recorded at ICMR-22 is to
   *apply* that pre-existing discipline here, which is reuse (R2), not
   authorship of new minimal machinery (R7). Even granting a charitable
   R7 reading, the rung-logging convention's own requirement — name which
   lower rungs were checked and why they failed, for *this* decision — is
   not met anywhere in the ICMR-22 row or its surrounding prose; §5.9's
   template will require that of *future* engine-gap records, which the
   critic correctly distinguishes as a different claim than satisfying it
   here.
3. *Could this be a harmless simplification not worth flagging, since the
   substance of both decisions is sound?* The critic already concedes
   this explicitly ("Neither mismatch changes what the proposal actually
   decided... both decisions are sound") and grades it as a
   warning-severity labeling defect rather than a substantive reversal —
   this is the correct proportionality, not an overstatement.

No refutation survives. Both citations are mismatched against the rung
table read directly from source, exactly as the critic found.

One check the critic did not need but this refuter ran anyway: confirmed
no other row in §17 was mislabeled by spot-checking a sample of five
additional rows (ICMR-04, ICMR-07, ICMR-10, ICMR-12, ICMR-19) against
their cited rungs — all five hold up (ICMR-04 "extend the existing
decomposition method" = R2 reuse; ICMR-10 "classify behavior units before
packages" = R1, a genuine new-requirement decision; ICMR-19 "pilot before
full-corpus migration" = R1, similarly a new sequencing requirement). The
critic's claim that only two of twenty-five rows are mismatched is not
contradicted by this spot check.

**Verdict: CONFIRMED.** Severity as reported (warning) is appropriate —
both underlying decisions are sound and the fix is a relabeling, not a
reversal, exactly as the critic concludes.

## Checked, no violation found (critic's non-findings) — spot-verified

The critic's report also includes an extensive "Checked, no violation
found" section (NORTH-STAR R-NS-1..6, AGENTS.md routing, all twelve ADRs
including 0012, PL/J ladder consistency with the current ICM ladder).
Since the contract requires every *finding* to be refuted, and these are
explicitly non-findings, they are not adjudicated as findings here — but
because a refuter who only checks reported findings could rubber-stamp a
critic who quietly missed something, several load-bearing factual claims
underlying those non-findings were independently spot-checked:

- **ADR 0012 exists, is dated 2026-08-16, and is titled "Estate and Doctor
  are daemon API surface, not a second TUI reach path"** —
  `docs/adr/0012-estate-and-doctor-are-daemon-api-surface.md`, confirmed
  by direct read. The "never a second computation or a second validation"
  clause the critic quotes is present verbatim (line 58).
- **`requires_ask` is an existing `workflow.toml` field, not new grammar**
  — confirmed via `grep -n requires_ask src/domain/workflow.rs`; the field
  is defined and used at multiple points including lines 247, 578, 1033,
  1108, matching the critic's citation.
- **R-NS-1 through R-NS-6 exist substantially as the critic describes** —
  `docs/gauntlet/notes/north-star-draft-2026-08-11.md` contains R-NS-1
  through R-NS-5 in full prose (lines 87–101); `NORTH-STAR.md` restates
  R-NS-6 verbatim and references the rest, matching the critic's own
  caveat that "`NORTH-STAR.md` itself only restates R-NS-6 verbatim."

All three spot checks hold. No basis found to elevate any non-finding to
a finding, and no basis found to doubt the critic's stated method was
actually followed.

## Summary

Both of the critic's findings survive independent adversarial refutation
with their reported severities unchanged:

- **F1 — CONFIRMED (error).** The `src/`-boundary contradiction is real,
  not resolved by the proposal's own draft/promotion process, and not
  softened by the unrelated §19 Owner Decision #3.
- **F2 — CONFIRMED (warning).** Both rung mismatches hold against the
  Ponytail table read from source; a spot check of five other rows found
  no further mismatches, consistent with the critic's claim that these
  are isolated labeling defects rather than a systemic problem with the
  register.

Neither finding is struck or downgraded. Nothing in the critic's
"Checked, no violation found" section was found to hide an unreported
violation under spot-checking.
