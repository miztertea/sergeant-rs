<!--
  Embedded ICM (Interpretable Context Model) policy context, added by
  split-hardening W2c/W3 (#261). This file exists because the dev-corpus
  ICM filesystem convention document — the source every shipped skill,
  workflow package, and shared context used to cite for its `.sergeant/`
  filesystem rules — is not part of the embedded distro and does not exist
  in a freshly `sgt init`'d consumer estate (`docs/` is not shipped; it is
  this repo's own dev corpus, per ADR 0014 decision 5).

  Rather than leave ~90 shipped files citing a path that resolves nowhere
  outside this one repository's working tree, this context carries the
  sections the shipped corpus actually cites — §1, §1a, §3, §4, §6.3,
  §7.4, and §7.5 — verbatim, with the source documents' own section
  numbering preserved unchanged, so an existing citation like "§1 rule 4"
  or "§6.3" still names the same rule it always did. §1, §1a, §3, §4, and
  §6.3 are numbered as in the ICM filesystem convention document; §7.4 and
  §7.5 are numbered as in that convention's own governing document (the
  ICM proposal, referenced below) — that source numbering is the one the
  shipped corpus's own "§7.4"/"§7.5" citations already assume. It is not
  either full document (those stay dev-corpus artifacts); it is the
  consumed subset, embedded so it ships with the binary and resolves in
  every estate `sgt init` produces.

  Sources: sergeant-rs's own ICM filesystem convention document
  (pre-relocation) for §1/§1a/§3/§4/§6.3, and the ICM proposal document
  (reference/proposal-next-iteration-icm-workflows.md, §7.4/§7.5 — the
  convention document's own governing document) for §7.4/§7.5 — both kept
  in this project's private development record. Internal cross-references
  to sections not carried here (e.g. §2, §5, §6.1, §6.2, §7.1-7.3, §7.6,
  §7.7) are prose pointers into that fuller source, not paths this file
  resolves — they are left as-is because renumbering would break the very
  citations this file exists to keep valid.
-->

# ICM Filesystem Convention — embedded policy excerpt

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
actor navigates itself. `_config/`, `references/`, and `output/`
directories are OPTIONAL per workflow and per stage — a stage with nothing
stable to reference and no declared artifact simply omits them.

Rules:

1. `.sergeant/index.md` is the root catalog. It MUST list every admitted
   workflow under `.sergeant/workflows/` and MAY link to each workflow's own
   `index.md`. A published workflow absent from the root index is a
   violation: the catalog is the discovery surface and an unlisted workflow
   is undiscoverable by design.
2. `.sergeant/common/` holds content shared by more than one workflow —
   contexts, scripts, and templates. A file placed under `common/` that is
   in fact used by exactly one workflow violates the sharing rule and MUST
   be moved to that workflow's own `scripts/`/context location, or a second
   consumer must be named.
3. Each workflow directory name is the workflow's identity and MUST be
   unique across `.sergeant/workflows/` and `.sergeant/drafts/workflows/`
   combined — a name collision between a draft and an admitted workflow is a
   violation (it makes "which one is `@@`-referenced or run" ambiguous).
4. Stage directories are prefixed with a two-digit (or wider, kept
   consistent within one workflow) ordinal (`00-`, `10-`, `20-`, ...) so the
   declared stage order and the directory listing order agree without
   reading `workflow.toml`. A directory listing whose lexical order
   disagrees with the order recorded in `workflow.toml` is a violation.
5. Every actor stage directory MUST contain a `CONTEXT.md`. A stage
   directory without one is not a stage anyone can run and is a violation.

## 1a. The four context layers (ICM)

Adopted from the published ICM protocol, which splits context on two axes
the flat model conflates: **orientation vs. contract** and
**stable-across-runs vs. produced-per-run**.

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
   declare which files an actor loads at stage entry. An actor reading
   files the stage did not declare is exploration (allowed, its judgment);
   a stage whose *contract* depends on a file its Inputs table omits is a
   violation — dependency tracking is the interpretability ICM is named
   for.
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
   surface and are Git-tracked on the Work branch — reviewable in the diff
   like any other change.
5. **Layer 1 is not a super-stage.** The workflow `CONTEXT.md` orients; it
   MUST NOT contain stage instructions, and no stage may require reading it
   in place of its own contract. The engine does not deliver Layer 1 —
   stages that need it name it in their Inputs table (typically only
   `00-`).
6. Downstream stages consume upstream Layer 4 artifacts by naming them in
   their own Inputs table (e.g. `10-hypothesize` inputs
   `00-reproduce/output/reproduction.md`). That named handoff — not shared
   conversation state — is how context flows between stages.

## 3. `AGENTS.md`: the small constitution

`AGENTS.md` teaches a harness how to enter the Sergeant system and resolve
this repository's conventions. It is not where procedure lives.

A minimal, complete shape:

```markdown
This repository uses Sergeant for durable procedural work.

- Discover available procedures in `.sergeant/index.md`.
- Select an admitted workflow explicitly when substantive work begins.
- Follow only the active stage context supplied by Sergeant.
- Resolve `@@name` references from `.sergeant/common/contexts/<name>.md`.
- Treat `.sergeant/common/scripts/` and workflow-local scripts as helpers,
  not independent procedure unless the workflow declares a durable stage.
- Do not treat `.sergeant/drafts/workflows/` as published procedure.
- Use Sergeant's respond, retry, cancel and inspection surfaces rather than
  fabricating workflow state in prose.
```

Rules:

1. `AGENTS.md` content MUST classify as a stable operating invariant. A
   rule that changes with each procedure, or that only applies inside one
   workflow's execution, does not belong in `AGENTS.md`; it belongs in
   that workflow's `CONTEXT.md` or a shared context (§4 below). Procedural
   detail leaking into `AGENTS.md` is a violation — it re-creates the
   "procedural encyclopedia" the small constitution exists to replace.
2. `AGENTS.md` MUST NOT duplicate content that already lives in
   `.sergeant/index.md`, a workflow's `index.md`, or a shared context. It
   references those surfaces by convention (as in the sample above); it
   does not restate their contents. Duplication is a violation because it
   creates two sources that can silently drift.
3. `AGENTS.md` changes rarely by design. A change to `AGENTS.md` driven by
   a single workflow's needs is a signal the change belongs elsewhere;
   reviewers SHOULD treat frequent `AGENTS.md` churn as a classification
   defect, not a documentation improvement.

**Rule 2 is superseded for the two ladders (owner ruling, 2026-08-17).**
Rule 2 forbids `AGENTS.md` restating a shared context. The owner ruled
that the CONSTRUCTION (Ponytail R1–R7) and AUTHORITY (Bounded-Judgment
J5–J0) ladders ship inline in `AGENTS.md`, because a Layer-0 always-on
file that points elsewhere for its own decision procedure is not
reachable in the mode that needs it most: `@@name` resolves only inside
an active stage's context, so a direct in-session Captain — the mode most
prone to over-escalation — had no ladder at all.

This is not an exemption from rule 2's *purpose*. Duplication is still a
violation. It is resolved by **moving canonicity, not by copying**:
`AGENTS.md` now owns the rung definitions for both ladders, and
`.sergeant/common/contexts/bounded-judgment.md` and `ponytail.md` are
reduced to what only they need (stage-specialization contract,
decision-evidence shape, conflict rule, authority inheritance, worked
example) and reference `AGENTS.md` for the rungs. One canonical source per
ladder, as rule 2 intends; `@@bounded-judgment` still resolves per §4.

## 4. The `@@name` shared-context convention

A stage context may include another file by reference rather than by
copying its text:

```markdown
Apply @@adversarial-review to the current change.
```

Resolution rule (fixed by `AGENTS.md`, per §3 above): `@@name` resolves to
`.sergeant/common/contexts/<name>.md`. There is no other resolution path —
no per-workflow override, no search path, no relative reference. A
`@@name` token that does not resolve to exactly that file is a violation:
either the referenced file does not exist (broken reference — a
structural-lint failure) or the author intended a workflow-local file,
which MUST be written out in full or referenced by its actual path, not
through `@@`.

Rules:

1. `@@name` is **context composition**, not workflow composition (§7.5 of
   the fuller source document). It pulls text into the current actor's
   stage context; it does not create a child stage, checkpoint, or durable
   identity. A `@@name` reference used to imply "and then run that other
   procedure as a sub-workflow" is a violation of scope — `@@name`
   composition was never, and is not now, the mechanism for that. An
   author who wants real nested execution within the same Work names a
   stage directory that carries its own `workflow.toml` (engine-level
   recursion); an author whose need is separately durable submits child
   Work instead, under the conditions AGENTS.md's ESTATE section states
   (host-atlas r3 ratification, ruling 2 — the same ruling ratified both
   primitives). Neither is a `@@name` reference, and reaching for one to
   imply either is still the same violation of scope this rule has
   always named.
2. Sergeant pins the textual `@@name` token in the referring `CONTEXT.md`,
   but at this milestone does **not** pin the transitive contents of
   `common/contexts/<name>.md` at replay time beyond what Git and the work
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

## 6.3. Review independence

A later stage in the *same* workflow may qualify as independent review
when it has a fresh execution, explicit inputs (not inherited conversation
state), a review-only contract, and no authority to edit the subject it
reviews. Independence lives in the execution boundary, not in whether the
reviewer happens to share a workflow wrapper with the work it reviews.

## 7.4. Authored metadata and observed telemetry remain separate

Authored files may contain:

```text
name
version
status
owner
description
tags
intended inputs and outputs
publication state
```

Run counts, completion rates, last execution, blocked time, cost, token
use, duration, retry frequency, and failure modes belong in the journal
and DuckDB projection.

The future discovery response may join them:

```text
diagnose-bug v3
  authored status     published
  tags                debugging, defect, investigation
  observed runs       184
  completion rate     87.5%
  median duration     14m22s
  last measured       2026-08-04
```

It should never write those mutable measurements back into the workflow's
front matter.

## 7.5. Shared context works now as an authoring convention

A stage context can contain:

```markdown
Apply @@adversarial-review to the current change.
```

The stable agent instructions define that token as:

```text
.sergeant/common/contexts/adversarial-review.md
```

The current actor receives the stage context and runs in the worktree, so
it can read that file without engine support.

This is **context composition**, not workflow composition. Sergeant pins
the textual reference in `CONTEXT.md`, but today it does not pin the
transitive contents of the referenced file. That is acceptable for the
measurement phase because Git preserves the source revision and the work
surface records its base SHA, but the exact replay semantics of transitive
workflow dependencies should remain an explicit future design question.
