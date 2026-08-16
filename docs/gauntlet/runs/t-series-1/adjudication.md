# T-SERIES-1 — adjudication

Orchestrator (Captain) adjudication of the panel and refuter output,
2026-08-16. Contract: `docs/gauntlet/contracts/T-SERIES-1.md`. Artifact:
`reference/proposal-tui-t-series.md` (2026-08-15 revision).

## Outcome: **validated with findings**

Every finding's own verdict, and every refuter's, holds that the affected
section survives a local correction. **No decision was invalidated and no
section was sent back.** One finding was fully refuted. Three severity
downgrades. Corrections are dispatched as a follow-on `sgt` Work rather than
applied by this adjudication directly — see "Disposition" below.

## Verdicts

| Axis | Findings | Refuted | Confirmed | Severity moves |
|---|---|---|---|---|
| fidelity | 4 | 0 | 4 | none |
| invariants | 3 | 0 | 3 | none — the unit's only surviving **error** on this axis |
| assumptions | 3 | 0 | 3 | 1 × error → warning |
| enactability | 9 | 1 | 8 | 1 × error → warning, 1 × warning → info |
| **total** | **19** | **1** | **18** | **3 downgrades** |

## The three surviving errors

**`inv-estate-doctor-bypasses-client-boundary`** (invariants). Decision
T2-14 (§5.3/§16.2/§16.3) has `tui.rs` call shared local Estate/Doctor
operations directly — a second crate-internal reach path beyond
`crate::api`, which `tests/m6_surfaces.rs` t5/t5b rejects today by
construction. The proposal names the rule it changes ("the old ... source-
scan rule") but never names the test itself, never states which of two
resolutions it intends (extend the API instead, leaving t5 untouched; or
explicitly revise t5 with its own justification), and the stated
reason for avoiding an API route — "would distort local/no-daemon
semantics" — is a cost the *CLI* bears, not the TUI, since T2-16 already
commits `tui.rs` to never running without a live daemon. Both critic and
refuter independently verified this against the live test source and ADR
0009; the refuter additionally checked a 2026-08-11 design note that
appears to anticipate T2-14's mechanism and found it *strengthens* rather
than excuses the finding — that note predates the very test hardening
(t5b) the proposal now silently collides with.

**Applied:** §5.3/§16.2/§16.3 do not get a unilateral fix invented at
adjudication time — this is a genuine open design question the proposal's
author must rule on, not something a grading pass should decide by
default. Per the FOUNDATION-1 precedent (`inv-one-owner-relocated`, which
carried an analogous error into a new explicit open-question subsection
rather than silently picking a side), T2-14 is marked **blocked pending an
explicit ruling** naming which of the two options wins, with `tests/
m6_surfaces.rs` t5/t5b cited by name for the first time. This ruling
becomes T0's first required output — which also directly closes
enactability Finding 9 (T0 previously had no named closure artifact).

**`assumptions F1`** (PR #111 stale "unmerged, concurrent" framing). PR
#111 merged into `main` at `3a46b87` on 2026-08-15T15:28:02Z — hours before
this proposal was drafted at its own stated `audit_revision`, and well
before this gauntlet unit ran. The proposal's frontmatter, header, §3.3,
and ~17 further locations (decision-register entries, acceptance items,
test sections, a falsifier) all instruct a reader or a dispatched Work to
treat PR #111's surfaces as pending. The assumptions refuter pushed back
hard on the FOUNDATION-1 precedent's instinct to treat this class of stale
citation as `warning` — and correctly distinguished this case: T2-06/T2-46
and §3.3 are **prescriptive instructions a dispatched Work would read as
binding**, not narrative background, so a T0 Work could actively avoid
consuming an already-shipped, already-verified API because the proposal
told it not to trust it yet. That is a materially different, worse risk
than inert prose, and it is why `error` is upheld here rather than
downgraded to match the two sibling citation defects (F2, F3) that *were*
downgraded.

**Applied:** every location the critic and refuter identified — and the
one additional site (`§24.2` line 2277) the refuter's independent sweep
found — is corrected from conditional to a plain statement of the merged
fact, dispatched below rather than hand-edited here given the ~20-location
spread.

**`enactability Finding 3`** (T0–T4 never assigns four of the five
canonical Work tabs to a phase). §13.2 requires `Thread Workflow Evidence
Graph Details`; §21 binds all four non-Thread tabs into acceptance (items
13, 24–26); nothing in §20.2–§20.5 names the Workflow rail, Evidence,
Graph, or Details tabs or their describing nouns. The refuter tried the
generous reading — "canonical Work shell" already means all five — and
rejected it on the proposal's own structure: "transcript-backed Thread" is
called out as a separate T1 bullet immediately next to "canonical Work
shell," which would be redundant if "shell" already meant everything.
Unlike the two downgraded enactability findings, this isn't a narrow
sub-clause with an obvious inference available — it's four full,
non-trivial, acceptance-bound deliverables with no owning phase, only
discoverable at final acceptance review.

**Applied:** name all four tabs explicitly under T1 in §20.2, per both
critic's and refuter's own suggested fix — the data backing all four
already exists in the shipped API, so this closes without new sequencing.

## The refuted finding

`enactability Finding 1` — that §20.1's "No product code" and its
dependency-resolution spike are mutually exclusive. Refuted cleanly: the
critic's own investigative method (`cargo tree`, `cargo add --dry-run`, a
scratch crate outside the product tree) already produces the required
evidence without touching a single product file, and the critic used
exactly those tools to write the finding one paragraph before concluding
no such path exists. Recorded because it is the panel working as designed
on this axis's own terms — a critic's evidence undermining its own
conclusion, caught by an adversarial pass that re-ran the critic's own
commands rather than trusting the prose.

## The three downgrades

- **`assumptions F2`** (error → warning): the proposal's cited commit for
  PR #111's head (`251a6f1`) is actually PR #122's merge commit, not PR
  #111's real head (`bceed96`) — real hash, wrong attribution. The refuter
  found this was scored `error` by association with sibling finding F1
  rather than on its own downstream impact, and is structurally identical
  to `F3` (a broken `supersedes` citation), which the same critic correctly
  scored `warning`. No normative content depends on the specific SHA.
- **`enactability Finding 2`** (error → warning): "review this proposal
  through the repository's proposal gauntlet" as a literal T0 task has no
  single dispatchable referent. The refuter found a competently dispatched
  Work has an obvious, low-judgment resolution (cite this unit's own closed
  outcome and proceed) rather than stalling — a wording gap, not a block.
- **`enactability Finding 6`** (warning → info): PR #111's disposition is
  pinned once at T0 with no named re-check point across T1–T4. The
  refuter's own live `gh pr view 111` call — re-run independently rather
  than trusting the critic's citation of the proposal's own stale prose —
  found the concrete risk scenario (T0 pins "excluded," PR merges later
  with no re-check) already foreclosed by real-world events that predate
  T0's earliest possible dispatch. Worth naming as a documentation nicety;
  no longer meets the "a Work would have to invent something" bar.

## The remaining twelve findings

All confirmed at their critic-assigned severity, all mechanically fixable
without touching any normative decision: three fidelity warnings (dead
`supersedes` link; a Doctor check-name list that invents "installation"/
"profiles" and drops the real `git`/`claude`/`permission_mode`; a
disposition-table row mislabeling new scope as a "revised" predecessor
decision), one fidelity info (an ADR 0009 attribution imprecision already
present in the codebase's own comments), one invariants warning (an
unjustified R7 tag on T2-40, missing the failed-lower-rung narrative the
proposal's own register convention requires and demonstrates two sections
earlier), one invariants info (§5.6's restatement of R-NS-6 drops its
per-transport conditionality and the WORKFLOW-IF-E3 consequence), one
assumptions warning (`supersedes.revision` points at the predecessor's own
audit pin, not the commit containing the predecessor), and five further
enactability findings (an ambiguous "validate" verb at T0; acceptance item
57 depending on the still-open #120 gate defect with no owning phase;
§19/§21 unassigned by phase; §8.7's spike checklist omitting the actual
gating test by name; T0 lacking its own closure artifact — this last one
resolved as a side effect of the client-boundary error's disposition
above).

## Disposition

Per this unit's contract, acceptance is the owner's, not the panel's — this
adjudication establishes the proposal is sound and current with what is
known now, not that it authorizes a build. **No build work is dispatched
by this adjudication.** Two follow-on items, both explicitly separate from
this ruling:

1. A mechanical corrections pass over `reference/proposal-tui-t-series.md`
   applying every confirmed finding's stated fix (frontmatter, ~20
   PR #111 locations, the Doctor check list, the disposition-table row,
   the ADR 0009 attribution, the R7 justification, the R-NS-6 restatement,
   the T0 client-boundary open question, the four Work-surface tabs under
   T1, and the six remaining smaller wording fixes) — dispatched as its
   own `sgt` Work against this branch so it gets the same review stage
   every other piece of this unit received, rather than hand-edited by
   this adjudication.
2. The client-boundary open question (`inv-estate-doctor-bypasses-client-
   boundary`) is a real, unresolved design decision for the proposal's
   owner, not something this adjudication invents an answer to. It is
   named as T0's first required ruling in the corrected proposal; §20's
   T1 is not dispatchable until it resolves.

Full record: `docs/gauntlet/runs/t-series-1/` (critics, refuters, this
adjudication). GAUNTLET.md carries the ledger entry.
