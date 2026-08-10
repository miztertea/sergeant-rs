# Draft workflow package template

Layer 3 (stable across runs), local to `60-draft`. Distills
`docs/icm/convention.md` §1/§1a/§2 into the exact shape one materialized
candidate package must have. If this file and `docs/icm/convention.md` ever
disagree, `docs/icm/convention.md` governs (edit-source principle).

## Where it goes

```text
.sergeant/drafts/workflows/<candidate-name>/
```

in **this run's own worktree** — never `.sergeant/workflows/`. `<candidate-name>`
is the kebab-case name `50-synthesize` minted for a workflow candidate. Before
writing anything, re-check that name is not already taken by any directory
under `.sergeant/workflows/` or `.sergeant/drafts/workflows/` (uniqueness is
required across both trees combined, `docs/icm/convention.md` §1 rule 3) —
including this very workflow's own name, `repo-to-icm`.

## Required contents, per candidate package

```text
<candidate-name>/
├── index.md          front matter: kind: workflow, name: <candidate-name>,
│                      status: draft, version: 1, description (a real
│                      trigger/outcome/completion sentence, not a restatement
│                      of the name), tags (optional)
├── workflow.toml      [workflow] name = "<candidate-name>", version = "1"
│                      (a quoted, DIGITS-ONLY string — never "v1" or
│                      "1.0". The engine's own loader requires the TOML
│                      type to be a string (measured: a bare TOML integer
│                      fails to parse); record-shapes.md §1's integer
│                      requirement is about the *value*, satisfied by
│                      digits-only content, not the TOML type), stages =
│                      [ordered list matching directory names exactly]
├── CONTEXT.md          Layer 1 orientation only — what this candidate is for,
│                      how its stages relate. No stage instructions here.
├── provenance.md       maps every stage (and the workflow as a whole) to the
│                      behavior_id(s) that justify it — see below
├── _config/            (optional) shared policy/method material genuinely
│                      stable across every future run of this candidate,
│                      used by more than one of its stages
├── scripts/            (optional) workflow-local helpers named by a
│                      candidate's classification record if one implied one
└── NN-<stage-name>/
    ├── CONTEXT.md      Layer 2: the stage contract, opening with an
    │                  ## Inputs table (record-shapes.md §1a). L4 rows here
    │                  point at THIS CANDIDATE's own earlier stages'
    │                  output/, never at repo-to-icm's own output/ — the
    │                  candidate is a standalone package once promoted.
    ├── references/     (optional) stage-specific stable material
    └── output/
        └── README.md   declares THIS CANDIDATE's own expected per-run
                        artifact(s) and disposition (promote|evidence) for
                        *its own future runs* — never populated with an
                        actual artifact at draft time
                        (`docs/icm/convention.md` §1a rule 4)
```

## `provenance.md`

One entry per stage (and one for the workflow as a whole), each naming the
`behavior_id`(s) from `../40-classify/output/classifications.ndjson` that
justify it. A stage or workflow candidate with **no** source evidence is
either:

- a justified design inference — clearly marked as such, with a one-line
  reason (e.g., a stage that exists only to give an evidence-backed sibling
  stage somewhere to hand off to), or
- unsupported invention — which is a defect `70-lint`'s validator checks for
  presence of citations (not their honesty) and which `80-adversarial-review`
  exists specifically to catch.

Do not synthesize a citation to avoid the first category; an honest "design
inference, no direct source" line is the correct output when that is the
truth.

## What never happens in a draft package

- `status: published` anywhere while the package lives under
  `.sergeant/drafts/workflows/` (`docs/icm/convention.md` §2 rule 3).
- A populated file (anything other than `README.md`) inside any of the
  candidate's own `NN-.../output/` directories. Those directories describe
  shape for the candidate's *own future runs*, not artifacts of the current
  `repo-to-icm` run.
- A `@@name` token that does not resolve to an existing
  `.sergeant/common/contexts/<name>.md`. If the candidate genuinely needs
  shared content that does not yet exist there, write the content out in
  full in the candidate's own `_config/` or stage `references/` instead of
  inventing a `@@` reference — a broken `@@` reference is a structural-lint
  failure (`docs/icm/convention.md` §4), and this stage does not create new
  files under `.sergeant/common/` itself (that is a promotion-time decision
  for a human, not this run).
- Stage instructions inside the candidate's own `CONTEXT.md` (Layer 1) —
  that content belongs in each `NN-.../CONTEXT.md` (Layer 2).
