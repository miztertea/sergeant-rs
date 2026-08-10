# ICM Filesystem Convention

Governing document: `reference/proposal-next-iteration-icm-workflows.md`
§§6, 7.1–7.2, 7.5–7.6. Milestone: `docs/gauntlet/contracts/N1.md`.

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
│       ├── scripts/
│       │   └── <workflow-local helper>
│       ├── 00-<stage-name>/CONTEXT.md
│       ├── 10-<stage-name>/CONTEXT.md
│       └── ...
└── drafts/
    └── workflows/
        └── <candidate-name>/
            ├── index.md
            ├── workflow.toml
            ├── provenance.md
            ├── scripts/
            └── 00-.../CONTEXT.md
```

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
