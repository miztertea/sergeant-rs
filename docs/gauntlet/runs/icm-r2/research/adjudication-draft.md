# Package adjudication: research

Producer pass, ICM-R2 pilot (`docs/adr/0013-icm-r0-owner-rulings.md`
decisions 8-9; `reference/proposal-icm-r-procedure-authority.md` §8, §10.3).
Method applied: Contract, Inventory, Harvest, Normalize, Placement
classification, Authority classification, Synthesis (§8.2-8.8). Behavior
units are dispositioned first; the package verdict is synthesized from them
afterward (`docs/icm/record-shapes.md` §6 rule 1) — not decided first and
back-filled.

## Original intention

Per the package's own upstream source
(`reference/sergeant-upstream/.agents/skills/research/SKILL.md`, N1
reference-corpus candidate **W27**): a Captain session spins up a
background agent to investigate a question against primary sources —
official docs, source code, specs, first-party APIs, never a secondary
write-up of them — tracing every claim back to its owning source, then
writes the findings to a single Markdown file, citing each claim, and
places it where the repository already keeps such notes (or states an
explicit sensible choice if no convention exists). The point of
backgrounding it is so the requester keeps working while it reads.

## Current trigger and outcome

**Trigger** (`CONTEXT.md`, `00-investigate/CONTEXT.md`): a topic needs to
be researched, or docs/API facts need gathering, and reading legwork is
delegated.

**Outcome**: one Markdown findings file exists under
`00-investigate/output/`, every claim cited to a primary source, placed
per the repository's existing note-keeping convention or an explicitly
stated sensible choice; `promote` disposition (survives into the Work
branch merge).

## Driver and admission boundary

**Driver:** stage-actor. The workflow receives an already-formed research
question as its Work intent; it does not itself decide whether research
should happen or shape that intent — that decision belongs to whatever
Captain session or upstream stage dispatches it (`BU-P3-041`).

**Admission boundary:** in-work. `research` is dispatched as an ordinary
`sgt run` Work with an already-admitted intent (the question), consistent
with the upstream skill's own framing ("spin up a background agent ... so
you keep working while it reads" — the delegation decision is made
*before* this workflow is admitted, by the dispatching session).

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---:|---:|---|---|---|
| `BU-P3-040` — research investigates a question against high-trust primary sources and writes findings to a Markdown file in the repository. | `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (frontmatter: description) | PL-4 | n/a (workflow-identity fact, not an actor decision) | STAND | `.sergeant/workflows/research/CONTEXT.md`, `index.md` (already correctly stated) |
| `BU-P3-041` — research is delegated to a background/async execution context so the requester's foreground work is not blocked. | `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (body line 6) | PL-4 (admission-boundary fact, scheduling property of invocation — not itself a stage) | n/a | STAND | `.sergeant/workflows/research/CONTEXT.md` (Notes for reviewers, already correctly folded, not a stage) |
| `BU-P3-042` — research must be conducted against primary sources, with every claim traced back to its owning source. | `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (item 1, line 10) | PL-5 (`00-investigate`) | J2 — stage delegates "choose which primary sources are authoritative; trace every claim" | STAND | `00-investigate/CONTEXT.md` (Behavior contract; retained, now also cited from the rewritten `## Bounded judgment` section below) |
| `BU-P3-043` — the investigation's output is a single Markdown file where every claim carries a source citation. | `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (item 2, line 11) | PL-6 (deterministic write-and-place mechanism, folded helper per N1 adjudication A4 — no `kind = "execute"` stage exists) | J1 — local, mechanical, reversible formatting choice | STAND | `00-investigate/CONTEXT.md` (Helper invocation section, already correctly folded) |
| `BU-P3-044` — the findings file is placed per the repository's existing convention, or a sensible location with the choice explicitly stated if none exists. | `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (item 3, line 12) | PL-5 (judgment-bearing placement choice, not purely mechanical — retained inside the same folded helper invocation) | J2 — stage delegates "choose placement; state the choice explicitly when no convention exists" | STAND | `00-investigate/CONTEXT.md` (Helper invocation section, already correctly folded) |
| `BU-R2-045` — an actor stage that meets unexpected file, path, or worktree state must treat it as a stop-and-ask condition, never as license to relocate its write target outside its assigned worktree (including into an orchestrating session's own live checkout). *(New unit, minted this pass — not part of the N1 `BU-P3-*` corpus; sourced from measured ICM-R0 gauntlet evidence, not from the upstream `SKILL.md`.)* | `GAUNTLET.md` Backlog item **B9** ("the Work found its own surface path git-ignored from the outer checkout's perspective, concluded that meant it was in \"the wrong (ignored) location,\" and relocated its write target to `/home/miztertea/sergeant-rs\` — a live checkout the orchestrating session was actively using — instead of asking"); ratified by `docs/adr/0013-icm-r0-owner-rulings.md` decision 8 | PL-5 (`00-investigate`) | J0 — no higher rung resolves an unexpected surface state; it is definitionally risk-changing (a write outside the assigned worktree) | FOLD — folds into `00-investigate`'s required `## Bounded judgment` section as its named J0 clause | `00-investigate/CONTEXT.md` (new `## Bounded judgment` section, replacing the current non-canonical `## Judgment required` prose) |

Verification note on the package-specific hint given for this pass: the
hint (add an explicit surface-boundary J0 clause) is **confirmed against
current content**, not assumed. `00-investigate/CONTEXT.md` currently
carries a `## Judgment required` section (ICM-R0-era vocabulary) that
states this is "an actor stage" requiring judgment in general terms, but
contains no J-rung structure and no clause addressing unexpected
file/path/worktree state at all — B9's failure mode is not covered by the
current text. `CONTEXT.md` (Layer 1) also has no `## Authority envelope`
section, which `docs/icm/convention.md` §6.1 requires unconditionally.
Both gaps are real, not hypothetical.

## Surviving package design

Placement is unchanged: one Sergeant workflow (PL-4) with one actor stage
(PL-5, `00-investigate`) that folds one deterministic helper invocation
(PL-6, write-and-place) — this matches the package as it stands and no
behavior unit's evidence argues for a different rung. What changes is
required *content*, per ADR 0013 decision 4 (every actor stage always
carries a local `## Bounded judgment` section, "inherits workflow envelope
unchanged" included) and decision 8 (this package specifically must get a
real, specific section, not the inherited-boilerplate default, as a direct
test against B9).

Two content amendments are specified in full below, as the actual
deliverable this pass owes decision 8. They are not applied to the live
package by this producer (see Review and promotion policy) — the
independent reviewer step evaluates this exact text before any promotion.

### Amendment 1 — `.sergeant/workflows/research/CONTEXT.md`: add `## Authority envelope`

```markdown
## Authority envelope

This workflow receives an already-admitted Work intent: a research
question or documentation/API-fact request, typically delegated by a
Captain session that wants to keep working while sources are read
(`BU-P3-041`).

### Workflow may decide
- Which primary sources are authoritative for the question, tracing every
  claim back to the source that owns it (`BU-P3-042`).
- Where the findings file is placed when the repository has no existing
  note-keeping convention, provided the choice is stated explicitly in the
  file itself (`BU-P3-044`).

### Workflow may not decide
- That a location outside its assigned Work surface is ever a valid write
  target, no matter how the surface appears from outside (git-ignored,
  unfamiliar, or otherwise "wrong"-looking) — see the stage's `J0` clause
  below.
- To answer from secondary summaries when primary sources are reachable.

### Human or Captain gates
- Any unexpected file, path, or worktree state is never resolved by the
  workflow alone; it is a stop-and-ask (`needs_input`) condition (see
  `00-investigate/CONTEXT.md`'s `## Bounded judgment`).

### Decision record
Material decisions (source selection, findings placement when no
convention exists, any `J0` stop) are recorded in the stage's own turn and
surfaced through `needs_input` where applicable; this single-stage
workflow declares no separate decision-log file.
```

### Amendment 2 — `00-investigate/CONTEXT.md`: replace `## Judgment required` with `## Bounded judgment`

```markdown
## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Choosing which primary sources (official docs, source code, specs,
  first-party APIs) are authoritative for the question, and tracing every
  claim back to the source that owns it (`BU-P3-042`).
- Choosing where the findings file is placed: follow the repository's
  existing note-keeping convention if one exists; otherwise choose a
  sensible location and state that choice explicitly in the findings file
  itself (`BU-P3-044`).

### J1 — local choices allowed
- Mechanical formatting of the findings file (heading structure, citation
  style), as long as every claim carries a source citation (`BU-P3-043`).
- The findings file's own filename, chosen inside the stage's assigned
  work surface.

### J0 — must become `needs_input`
- **Any unexpected file, path, or worktree state** — a surface that looks
  git-ignored, missing, unfamiliar, or otherwise inconsistent with what
  this stage was told to expect. This is always a stop-and-ask condition,
  never a reason to search for, infer, or relocate a write target outside
  the stage's own assigned worktree — including into an orchestrating
  session's own live checkout. (Direct fix for the observed failure
  recorded as `GAUNTLET.md` backlog item B9 and ratified by
  `docs/adr/0013-icm-r0-owner-rulings.md` decision 8: a dispatched
  `research` Work once inferred its own surface was "the wrong (ignored)
  location" and wrote into the orchestrating session's active checkout
  instead of asking.)
- Primary sources conflict on a material fact and no higher rung resolves
  which one governs.
- No primary source can be found for a claim the requester needs answered.

### Completion boundary
This stage may complete only when a single Markdown findings file exists,
every claim in it carries a source citation, it has been placed per the
rule above, and no `J0` condition above was encountered without first
being raised as `needs_input`.

### Decision evidence
Record material J2 decisions (source-selection rationale, findings
placement choice when no convention exists) in the findings file itself,
under a short "Sources and placement" note. A `J0` stop is recorded in the
turn's own `needs_input` question per `@@bounded-judgment`'s canonical
shape.
```

The existing `## Helper invocation: write findings` section is otherwise
unchanged; it continues to carry `BU-P3-043`/`BU-P3-044`'s citations
exactly as N1 adjudication A4 folded them.

## Inputs and outputs

Unchanged from the current package. `00-investigate/CONTEXT.md`'s Inputs
table names only `../CONTEXT.md` (L1, first stage only) — correct, since
this workflow has no Layer 3 references and no upstream Layer 4 artifact
to consume (single-stage workflow). Output: one `promote`-disposition
Markdown findings file under `00-investigate/output/`, unchanged.

## Review and promotion policy

This package's live content is `status: published` — a promotable
surface (`docs/icm/convention.md` §2.1). Per ADR 0013 decision 6 /
`reference/proposal-icm-r-procedure-authority.md` §9.7, a content change
to it may not be landed by this producer; it requires the ICM-R2 pilot's
own independent reviewer step (fresh execution, explicit inputs, review-only
contract — decision 7) before Captain's reconcile-and-publish pass (§8.11
-8.12). This record is that reviewable draft: Amendments 1-2 above are the
exact proposed replacement text, not yet applied to
`.sergeant/workflows/research/`.

## Alternatives considered

- **REHOME to a Captain skill (PL-2).** Rejected: research's primary
  product is a durable, cited artifact usable independent of the
  originating conversation continuing, and it can run with Captain absent
  (`BU-P3-041`'s whole point is backgrounding it away from the live
  session). It fails PL-2's own discriminator — its job is not to decide
  what Work should exist.
- **Engine-level containment fix** (a Work simply cannot address paths
  outside its assigned worktree, regardless of instruction quality).
  `GAUNTLET.md` B9 names this as an open alternative to a content fix.
  Not pursued here: ICM-R2 is content-only under ADR 0013 decision 10's
  runtime freeze, and B9 itself records that a second, differently
  instructed dispatch with explicit surface-boundary language did *not*
  reproduce the failure — evidence the content-layer fix is plausible
  before an engine change is warranted (proposal §4.8, lowest viable
  rung). If a properly-bounded intent reproduces the failure again after
  this amendment lands, B9's own trigger clause points at the engine-level
  fix next, not before.
- **Leave the existing `## Judgment required` prose as-is and treat B9 as
  already covered.** Rejected on inspection (see Verification note above)
  — the current text is generic actor-stage boilerplate with no J-rung
  structure and no clause naming unexpected file/path/worktree state at
  all. ADR 0013 decision 4 requires the canonical `## Bounded judgment`
  shape unconditionally; decision 8 requires this package specifically to
  make it real, not inherited.
- **Mint the new unit as a shared context instead of a stage-local
  clause** (e.g., add it to `@@bounded-judgment` itself as a universal
  rule for every actor stage). Considered but not adopted by this
  producer: `.sergeant/common/contexts/bounded-judgment.md` is explicitly
  the canonical ladder definition, not a place for one package's local
  specialization (`docs/icm/convention.md` §2 rule 2 — shared only when
  two or more consumers use the *same contract*). Whether the same clause
  should generalize to every actor stage's inherited default is a
  cross-package question the ICM-R1/R2 synthesis step, not this single
  package's adjudication, should decide once other packages' pilot passes
  are in.

## Final disposition
STAND

## Validation evidence

- **Source-valid:** every retained behavior unit's citation was re-read
  directly from `reference/sergeant-upstream/.agents/skills/research/
  SKILL.md` this pass (not trusted from `docs/gauntlet/promoted-provenance/
  research.md` alone) and matches; `BU-R2-045`'s citation was re-read
  directly from `GAUNTLET.md`'s B9 row and `docs/adr/
  0013-icm-r0-owner-rulings.md` decision 8, both quoted above.
- **Placement-valid:** PL-4/PL-5/PL-6 rungs re-derived from
  `reference/proposal-icm-r-procedure-authority.md` §5 in this pass, not
  copied from the package's own prior self-description; discriminators
  against PL-2/PL-3 checked explicitly (see Alternatives considered).
- **Authority-valid:** every surviving unit now has a named J-rung; the
  package's only real gap (no explicit J0 clause, no canonical section
  shape, no Authority envelope at all) is the one this pass exists to
  close, per decision 8's own framing ("a direct fix for an observed
  failure, not a hypothetical one").
- **Structurally valid:** `workflow.toml` stage list (`00-investigate`)
  matches the directory listing; `index.md` front matter (`kind`, `name`,
  `status`, `version`, `description`) resolves against
  `docs/icm/record-shapes.md` §1 with no violation found.
- **Execution-valid:** not exercised this pass (content-only producer
  pass, per ADR 0013 decision 10's runtime freeze and this workstream's
  own content-first discipline). Amendments 1-2 should be exercised on a
  real `sgt run` dispatch — ideally one that reproduces an unexpected
  surface state — once promoted, per proposal §9.3's needs_input
  validation requirement.
- **This producer's own compliance:** this record was produced from the
  worktree assigned to this task
  (`.sergeant/data/surfaces/01M05XXD4TD9FKEDZ1JRZMC55G/sergeant-rs`,
  already at the target branch's commit). No file outside this worktree
  was read or written in producing it — the same discipline `BU-R2-045`
  requires of `research`'s own stage is applied here by this producer,
  not merely prescribed for the package under review.
