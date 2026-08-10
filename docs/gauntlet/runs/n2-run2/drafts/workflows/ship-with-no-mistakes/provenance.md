# provenance — ship-with-no-mistakes

Maps every stage, and the workflow as a whole, to the `behavior_id`(s) in
`../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson`
that justify it, per `../../../workflows/repo-to-icm/60-draft/references/
draft-package-template.md`.

## Workflow as a whole

- **`behavior_id`s:** `BU-0028`, `BU-0029`, `BU-0030`, `BU-0031`,
  `BU-0032`, `BU-0033`, `BU-0034` — seven `stage-context` records, all
  unattached (see "Unattached ground" below).
- **Representation:** none of these carries `representation: workflow`.
  No direct `workflow` or `stage` record names `ship-with-no-mistakes`
  anywhere in this corpus.
- **Basis:** indirect, but evidenced, not invented — the workflow's
  trigger, outcome, and completion-condition text (index.md, CONTEXT.md)
  is assembled entirely from these seven records' own trigger/outcome
  fields, per `../../../workflows/repo-to-icm/50-synthesize/output/
  candidates.md` bucket 1 candidate 3. This is the weakest form of
  workflow evidence in this corpus: unlike `standard-task-workflow`
  (evidenced by five `stage` records plus two unattached `stage-context`
  records), `ship-with-no-mistakes` has **no `stage`-level evidence at
  all** — only stage-context content that names it as a `workflow` value.

## Stages

**None.** No `behavior_id` in this corpus carries `representation: stage`
with `workflow: ship-with-no-mistakes`. This candidate has zero member
stage candidates — see `../CONTEXT.md`'s "Why this package has no stages"
and `workflow.toml`'s comment. Per `../../../workflows/repo-to-icm/
60-draft/CONTEXT.md` step 3, a stage with no direct source evidence is
marked as a justified design inference and never given an invented
citation — applied here at the whole-workflow level: no stage is invented
to fill this gap.

## Unattached ground (the entirety of this candidate's direct evidence)

All seven records name a `workflow`+`stage` pair with no corresponding
`stage` candidate anywhere in this corpus (per `../../../workflows/
repo-to-icm/50-synthesize/output/candidates.md`'s "Unattached records"
section, verified there programmatically):

| `behavior_id` | `stage` named | Why it doesn't attach |
|---|---|---|
| `BU-0028` | `start-run` | No `stage` candidate named `start-run` exists — no `representation: stage` record cites `ship-with-no-mistakes` at all. |
| `BU-0029` | `start-run` | Same as `BU-0028`. |
| `BU-0030` | `drive-gates` | No `stage` candidate named `drive-gates` exists. |
| `BU-0031` | `drive-gates` | Same as `BU-0030`. |
| `BU-0032` | `finish-run` | No `stage` candidate named `finish-run` exists. |
| `BU-0033` | `finish-run` | Same as `BU-0032`. |
| `BU-0034` | `route-findings` | No `stage` candidate named `route-findings` exists. |

Four distinct stage names are visible in this unattached content
(`start-run`, `drive-gates`, `finish-run`, `route-findings`), each cited by
one or two records, but none of the seven carries `representation: stage`
— they are all `stage-context` (judgment content that would attach to a
stage's own `CONTEXT.md` if that stage existed), so none of the four names
is promoted to a stage candidate on this run's evidence. A human reviewer
with access to the underlying source material could plausibly classify
each of these four names as its own `stage` on a future run — that is a
fresh classification decision this run's evidence does not make for them.
