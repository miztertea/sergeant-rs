# ICM Filesystem Convention

Governing documents: `reference/proposal-next-iteration-icm-workflows.md`
§§6, 7.1–7.2, 7.5–7.6, and the published ICM protocol itself (Van Clief &
McDermott, arXiv 2603.16021 — the four-layer context model in §6 below is
adopted from it by owner direction 2026-08-10; register row D9). Milestone:
`docs/gauntlet/contracts/N1.md`.

This document is normative for anyone authoring `.sergeant/` content by hand
and for any future generator (`repo-to-icm`, N2) producing it. It defines
**shapes**: what files and directories mean, what is legal where, and what
counts as a violation. It defines no tooling and no engine behavior — the
current loader (as of this milestone) reads only a declared `workflow.toml`
and its stage `CONTEXT.md` files; everything else described here is ordinary
Git content the loader does not need to understand yet (§7.1).

Companion document: `record-shapes.md` defines the front-matter and record
shapes referenced from here (`index.md` front matter, behavior units,
classification records, engine-gap claims).

---

## 1. The `.sergeant/` catalog layout

```text
.sergeant/
├── index.md
├── common/
│   ├── contexts/
│   │   └── <name>.md
│   ├── scripts/
│   │   └── <helper>
│   └── templates/
│       └── <template>.md
├── workflows/
│   └── <workflow-name>/
│       ├── workflow.toml
│       ├── index.md
│       ├── CONTEXT.md                  # Layer 1: workflow orientation
│       ├── _config/                    # Layer 3, workflow-shared
│       │   └── <policy-or-method>.md
│       ├── scripts/
│       │   └── <workflow-local helper>
│       ├── 00-<stage-name>/
│       │   ├── CONTEXT.md              # Layer 2: the stage contract
│       │   ├── references/             # Layer 3, stage-specific
│       │   │   └── <method>.md
│       │   ├── scripts/
│       │   │   └── <stage-local helper>
│       │   └── output/                 # Layer 4: per-run artifacts
│       │       └── README.md           # declares expected artifacts
│       ├── 10-<stage-name>/
│       │   └── ...
│       └── ...
└── drafts/
    └── workflows/
        └── <candidate-name>/
            ├── index.md
            ├── workflow.toml
            ├── CONTEXT.md
            ├── provenance.md
            ├── _config/
            ├── scripts/
            └── 00-.../
                └── CONTEXT.md ...
```

Only `workflow.toml`, stage ordering, and each stage's `CONTEXT.md` are
read by the engine today; every other path is authoring convention the
actor navigates itself (§7.1). `_config/`, `references/`, and `output/`
directories are OPTIONAL per workflow and per stage — a stage with nothing
stable to reference and no declared artifact simply omits them.

Rules:

1. `.sergeant/index.md` is the root catalog. It MUST list every admitted
   workflow under `.sergeant/workflows/` and MAY link to each workflow's own
   `index.md`. A published workflow absent from the root index is a
   violation: the catalog is the discovery surface and an unlisted workflow
   is undiscoverable by design (§7.3).
2. `.sergeant/common/` holds content shared by more than one workflow —
   contexts, scripts, and templates. A file placed under `common/` that is
   in fact used by exactly one workflow violates the sharing rule in §5 of
   this document (derived from proposal §7.6) and MUST be moved to that
   workflow's own `scripts/`/context location, or a second consumer must be
   named.
3. Each workflow directory name is the workflow's identity and MUST be
   unique across `.sergeant/workflows/` and `.sergeant/drafts/workflows/`
   combined — a name collision between a draft and an admitted workflow is a
   violation (it makes "which one is `@@`-referenced or run" ambiguous).
4. Stage directories are prefixed with a two-digit (or wider, kept
   consistent within one workflow) ordinal (`00-`, `10-`, `20-`, ...) so the
   declared stage order and the directory listing order agree without
   reading `workflow.toml`. A directory listing whose lexical order
   disagrees with the order recorded in `workflow.toml` is a violation
   (proposal §9.7 lists this as a structural-validator check).
5. Every actor stage directory MUST contain a `CONTEXT.md`. A stage
   directory without one is not a stage anyone can run and is a violation.

## 1a. The four context layers (ICM)

Adopted from the published ICM protocol (arXiv 2603.16021), which splits
context on two axes the flat model conflates: **orientation vs. contract**
and **stable-across-runs vs. produced-per-run**.

```text
Layer 1  <workflow>/CONTEXT.md      orientation: what this workflow is for,
                                    how its stages relate, how an actor
                                    routes itself — never stage instructions
Layer 2  <stage>/CONTEXT.md         the stage contract: what must become
                                    true here, with an Inputs table naming
                                    exactly which files load at this stage
Layer 3  references/, _config/,     reference material STABLE ACROSS RUNS:
         common/contexts/           methods, policies, rubrics; edited only
                                    to change every future run
Layer 4  <stage>/output/            working artifacts PRODUCED PER RUN,
                                    written in the work surface, traveling
                                    with the Work branch
```

Rules:

1. **Layer 2 carries an Inputs table.** Every stage `CONTEXT.md` MUST
   declare which files an actor loads at stage entry (see
   `record-shapes.md` §1a for the shape). An actor reading files the stage
   did not declare is exploration (allowed, its judgment); a stage whose
   *contract* depends on a file its Inputs table omits is a violation —
   dependency tracking is the interpretability ICM is named for.
2. **The layer split is lifetime, not distance.** A file belongs in Layer 3
   iff it does not change between runs of the workflow; it belongs in
   Layer 4 iff a run produces it. Mixing them — per-run scratch written
   into `references/`, or a stable rubric parked in `output/` — is a
   violation: it forces every later reader (actor or human) to re-sort
   what the filesystem should already have sorted.
3. **The edit-source principle.** Fixing a defect by editing a Layer 4
   artifact fixes one run; fixing it by editing the Layer 2/3 source fixes
   every future run. Actors SHOULD surface source-level defects they meet
   (a wrong rubric, a stale method) as findings for the workflow's owner
   rather than silently compensating in output — outputs are edit
   surfaces, but the source is where permanence lives.
4. **Layer 4 declares its shape up front.** An `output/` directory in the
   authored tree contains only a `README.md` (or equivalently `.gitkeep`
   plus documentation in the stage `CONTEXT.md`) declaring the expected
   artifacts. Per-run artifacts are written there in the materialized work
   surface and are Git-tracked on the Work branch — reviewable in the
   diff like any other change. This is the lower-rung answer to the
   proposal's deferred artifact declaration (§24.4): declared locations,
   no engine collection, no artifact manifest machinery.
5. **Layer 1 is not a super-stage.** The workflow `CONTEXT.md` orients; it
   MUST NOT contain stage instructions, and no stage may require reading it
   in place of its own contract. The engine does not deliver Layer 1 —
   stages that need it name it in their Inputs table (typically only
   `00-`).
6. Downstream stages consume upstream Layer 4 artifacts by naming them in
   their own Inputs table (e.g. `10-hypothesize` inputs
   `00-reproduce/output/reproduction.md`). That named handoff — not shared
   conversation state — is how context flows between stages; the engine's
   fresh-execution-per-stage model (§7.1) depends on it.

Open questions (recorded, not resolved — N2's runs are the measurement):

- **Merge-back semantics for Layer 4.** ICM assumes a static workspace
  whose outputs accumulate; Sergeant's Work branches are meant to merge.
  Working rule (owner-shaped, 2026-08-10; measured by N2): every declared
  output carries a **disposition** in its stage's `output/README.md` —
  `promote` (survives into the merge) or `evidence` (Work-branch record
  only) — and a workflow that declares any output ends with a
  deterministic **finalize** step that applies the policy mechanically:
  keep `promote` files, remove `evidence`-class and undeclared files in a
  final commit ("silence promotes nothing", executed rather than
  reviewed; removed files remain in Work-branch history). Today the
  finalize step is a shared helper invoked by the closing actor stage; it
  is a canonical execute-stage workload once `kind = "execute"` exists.
  Judgment about *whether* an artifact should exist belongs to the stage
  that writes it, not to finalize — authors wanting conditional logic in
  finalize is grammar pressure to record, not accommodate. Both halves
  are structurally lintable (disposition present; finalize step present
  when outputs are declared).
- **Inputs-table enforceability.** Lint can verify that listed paths
  resolve and that L4 inputs come from earlier stages (machine-checkable).
  *Completeness* — no contract-bearing dependency omitted — is
  review-enforced only. Rule 1's "violation" is a review verdict, not a
  lint result, and the convention claims no more than that.

## 2. The draft publication boundary

```text
.sergeant/drafts/workflows/   generated, reviewable, NOT runnable by name
.sergeant/workflows/          admitted, versioned, runnable procedure
```

This split is the entire publication mechanism at this milestone. There is
no `status = draft` engine enforcement and none is required (§7.1) — the
boundary is the directory itself.

Rules:

1. Nothing under `.sergeant/drafts/` is procedure an agent may follow as if
   it were admitted. An agent instructed by `AGENTS.md` (§3 below) MUST
   treat everything under `drafts/` as read-only evidence for human or
   reviewed-generator promotion, never as a workflow to execute.
2. Promotion from draft to admitted is a distinct, human-reviewed act: it
   MUST move (not copy-and-leave) the candidate from
   `.sergeant/drafts/workflows/<candidate>/` to
   `.sergeant/workflows/<name>/`, dropping or archiving `provenance.md`
   generation artifacts as the review record dictates. A workflow directory
   that exists identically in both trees at once is a violation — it means
   the boundary was not actually crossed, only copied, and the draft
   original remains a latent duplicate identity.
3. A `workflow.toml` or `index.md` MUST NOT declare `status: published` (see
   `record-shapes.md` §1) while physically located under
   `.sergeant/drafts/`. Status is asserted by front matter; location is
   asserted by the filesystem; the two MUST agree. A draft claiming
   `published` status is a violation regardless of which future tool would
   have caught it.
4. Generated content (from `repo-to-icm` or any future generator) MUST land
   under `.sergeant/drafts/workflows/`, never directly under
   `.sergeant/workflows/`. A generator that writes directly into the
   runnable namespace violates this convention even if its output is
   correct — correctness does not substitute for review (proposal §9.6,
   §9.1: "It does not publish workflows").

## 2a. The execution-surface test (owner ruling, 2026-08-11)

Added after the first dogfood gauntlet measured the conflation: the §6
publication ladder sorts invariants from procedures from machinery, but
never asked **who drives**. Before anything is admitted as a workflow, it
passes the three-way test:

1. **Workflow** — receives an *intent*, runs multi-stage with declared
   inputs/outputs, requires judgment at its checkpoints, and is driven
   end-to-end by sergeant. The test: "would a human type
   `sgt run '<intent>' --workflow X`?" If the package cannot absorb an
   intent — if it does the same thing every time — it is not a workflow.
2. **CLI surface** — a deterministic operation on sergeant's own state
   (fleet status, cleanup, graph, resolution). The "judgment" test: if
   the judgment is just reading sgt state and acting mechanically, it is
   a `sgt` verb (or TUI affordance — see
   `reference/proposal-tui-t-series.md`), not a workflow. Candidates are
   reconciled against existing product surface before anything is built.
3. **Operator skill** — instructions that teach the interactive harness
   (Claude Code in the deployment flow: clone → `sgt init` → launch
   Claude) how to operate sergeant well. These live at the skills/
   `AGENTS.md` layer the harness loads — sergeant never drives them.

A package that fails the workflow test is not deleted — it is re-homed
to the surface it actually belongs to. Interactive workflows (grilling-
class) remain workflows ONLY where the engine can hold their checkpoints
open for a human (the E3 design item); until then their packages must
say so.

**4. Absorbed-by-engine (owner ruling, 2026-08-11, second amendment).**
Before any of the three buckets above, every candidate passes the R1
rung check *against the current product*: does sergeant-rs already do
this? A package whose behavior the engine has subsumed (dispatch's
work/worktree/brief/intent mechanics = `sgt run` + surfaces + journal;
respond-to-worker = `sgt respond`; wake-and-resume = recovery + the
settle driver) is neither workflow nor verb-to-build — it becomes
engine documentation or retires with provenance preserved. The N1
classification missed this class structurally: the Ponytail ladder was
applied within packages (A4) but the engine itself was never on the R1
shelf as existing functionality, because the corpus's subject predated
the engine. Every future classification pass lists the engine's
capability surface beside the candidates.

## 3. `AGENTS.md`: the small constitution

`AGENTS.md` teaches a harness how to enter the Sergeant system and resolve
this repository's conventions. It is not where procedure lives (proposal
§7.2).

A minimal, complete shape:

```markdown
This repository uses Sergeant for durable procedural work.

- Discover available procedures in `.sergeant/index.md`.
- Select an admitted workflow explicitly when substantive work begins.
- Follow only the active stage context supplied by Sergeant.
- Resolve `@@name` references from `.sergeant/common/contexts/name.md`.
- Treat `.sergeant/common/scripts/` and workflow-local scripts as helpers,
  not independent procedure unless the workflow declares a durable stage.
- Do not treat `.sergeant/drafts/workflows/` as published procedure.
- Use Sergeant's respond, retry, cancel and inspection surfaces rather than
  fabricating workflow state in prose.
```

Rules:

1. `AGENTS.md` content MUST classify at ladder rung §6.1 (stable operating
   invariant — see `record-shapes.md` §4 for the classification record that
   makes this determination explicit). A rule that changes with each
   procedure, or that only applies inside one workflow's execution, does
   not belong in `AGENTS.md`; it belongs in that workflow's `CONTEXT.md` or
   a shared context (§4 below). Procedural detail leaking into `AGENTS.md`
   is a violation — it re-creates the "procedural encyclopedia" the small
   constitution exists to replace (§7.2).
2. `AGENTS.md` MUST NOT duplicate content that already lives in
   `.sergeant/index.md`, a workflow's `index.md`, or a shared context. It
   references those surfaces by convention (as in the sample above); it
   does not restate their contents. Duplication is a violation because it
   creates two sources that can silently drift.
3. `AGENTS.md` changes rarely by design (§7.2). A change to `AGENTS.md`
   driven by a single workflow's needs is a signal the change belongs
   elsewhere; reviewers SHOULD treat frequent `AGENTS.md` churn as a
   classification defect, not a documentation improvement.

## 4. The `@@name` shared-context convention

A stage context may include another file by reference rather than by
copying its text:

```markdown
Apply @@adversarial-review to the current change.
```

Resolution rule (fixed by `AGENTS.md`, per §3 above): `@@name` resolves to
`.sergeant/common/contexts/name.md`. There is no other resolution path —
no per-workflow override, no search path, no relative reference. A
`@@name` token that does not resolve to exactly that file is a violation:
either the referenced file does not exist (broken reference — a
structural-lint failure per proposal §9.7) or the author intended a
workflow-local file, which MUST be written out in full or referenced by
its actual path, not through `@@`.

Rules:

1. `@@name` is **context composition**, not workflow composition (§7.5).
   It pulls text into the current actor's stage context; it does not create
   a child stage, checkpoint, or durable identity. A `@@name` reference used
   to imply "and then run that other procedure as a sub-workflow" is a
   violation of scope — see §7.7's ruling that true nested workflows do not
   exist yet; that intent must be recorded as an engine-gap claim
   (`record-shapes.md` §5), not smuggled through a context reference.
2. Sergeant pins the textual `@@name` token in the referring `CONTEXT.md`,
   but at this milestone does **not** pin the transitive contents of
   `common/contexts/name.md` at replay time beyond what Git and the work
   surface's base SHA already provide (§7.5). Authors MUST NOT assume a
   `@@name` reference is content-addressed or versioned independently of
   the repository revision; treating it as such is a documentation error,
   not a filesystem violation, but reviewers SHOULD flag replay-sensitive
   uses (e.g. contexts a workflow depends on being byte-stable across many
   runs) as a candidate engine-gap finding rather than silently relying on
   Git history.
3. Every name under `.sergeant/common/contexts/` MUST be unique (it is a
   flat namespace by construction — one directory, one file per name). A
   second file that would shadow an existing `@@name` is a violation.

## 5. Helper rules

A stage may say:

```markdown
Run `.sergeant/common/scripts/validate-drafts.py`.
Review its structured result and correct any defects before completing.
```

Helpers are deterministic machinery invoked *while crossing* a checkpoint,
subordinate to the stage's judgment-bearing outcome (§6.5, §7.6). They are
not procedure in their own right.

Rules:

1. A helper MUST NOT be treated as a stage merely because it is executable
   (§6.3). The ladder's test governs: if replacing the helper's
   implementation tomorrow (Bash, Python, a compiled binary — the mechanism
   is irrelevant) would leave the surrounding procedural checkpoint
   unchanged, it is a helper, not a stage. Materializing a helper as its own
   stage directory when it fails this test is a violation — it fragments a
   single checkpoint into machinery-shaped ceremony (§6.3's `test.sh`
   example).
2. A helper's outcome becomes a durable checkpoint only when the *ladder*
   classifies it that way (§6.3–6.4), not when an author finds it
   convenient to track separately. A helper promoted to stage status without
   a classification record (`record-shapes.md` §5) justifying rung §6.3 or
   §6.4 is a violation.
3. Placement follows reuse, not convenience (§6.6, §7.6):
   - A helper used by exactly one workflow lives under that workflow's own
     `scripts/`.
   - A helper used by more than one workflow with the *same contract*
     (same inputs, same output shape, same meaning) lives under
     `.sergeant/common/scripts/`.
   - A helper copy-pasted into two workflows' local `scripts/` directories
     with the same contract is a violation of the reuse rule in §1.2 above
     — it must be consolidated under `common/scripts/` or the two uses must
     be shown to have genuinely different contracts (in which case they are
     not "the same helper" and the duplication is not a violation).
4. The current harness executes a helper as the invoking user, in the work
   surface, with no engine-mediated sandboxing or result interpretation
   (§7.6). A helper's `CONTEXT.md` reference MUST NOT imply that Sergeant
   itself parses, sandboxes, or acts on the helper's output — the actor
   stage does that. A context that reads as if the engine validates a
   helper's result is a violation; the engine only starts the actor's turn
   and journals what happens.
5. A helper is never itself a place to hide judgment. If a script's exit
   code or output requires the actor to *decide* something non-mechanical
   (choose among alternatives, ask the user, explain a decision), that
   decision point is evidence for an actor-stage or a distinct checkpoint
   (§6.4), not for adding more branching logic inside the helper. A helper
   that embeds judgment the ladder would classify as rung §6.4 is a
   violation — the judgment must surface into the stage context, not stay
   buried in script logic no reviewer will read as procedure.

## 6. Placement and Authority (ICM-R, `docs/adr/0013-icm-r0-owner-rulings.md`)

Two ladders, canonical sources fixed elsewhere, referenced here rather than
restated:

- **Placement Ladder (PL-0..PL-7)** — `reference/proposal-icm-r-procedure-
  authority.md` §5. Answers "what is the lowest-authority, smallest-surface
  representation that faithfully owns this behavior?" Extends this file's
  own §6.1a/6.2 driver discriminator with the full PL rung set (Captain
  skill, actor skill, workflow, stage, deterministic mechanism, engine
  gap).
- **Bounded-Judgment Ladder (J5..J0)** — `.sergeant/common/contexts/
  bounded-judgment.md`, referenced as `@@bounded-judgment`. Answers "what
  authority allows this actor to decide this material question without
  returning to a human or higher authority?"

### 6.1. Required sections

Every workflow's Layer-1 `CONTEXT.md` carries an `## Authority envelope`
section (what the workflow may decide, may not decide, which decisions are
human/Captain gates, where material decisions are recorded). Every actor
stage's `CONTEXT.md` carries a `## Bounded judgment` section (its J2
delegations by name, its J1 local choices, what must become `needs_input`
at J0, its completion boundary, where decisions are recorded) — always
present, even when it is only "inherits workflow envelope unchanged"
(decision 4: omission is never ambiguous). Every Captain skill's `SKILL.md`
carries the same conceptual section adapted to its driver (what it may
decide, what it must ask the user, what it must not do, its durable
handoff if any).

### 6.2. Review scope

Independent review, and the "no producer self-promotes" rule (§2 above),
apply to **promotable effects only** — artifacts that will be merged,
published, installed, admitted, signed, released, or treated as settled
(decision 6). Ephemeral output does not require a new Work merely to
exist, and does not require review-of-the-review.

### 6.3. Review independence

A later stage in the *same* workflow may qualify as independent review
when it has a fresh execution, explicit inputs (not inherited conversation
state), a review-only contract, and no authority to edit the subject it
reviews (decision 7). Independence lives in the execution boundary, not in
whether the reviewer happens to share a workflow wrapper with the work it
reviews.
