# Provenance — To Spec

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W31** `to-spec`.

## Stages

### `00-gather-context`

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-050` | Turning the current conversation into a published spec is a synthesis-only procedure: do not interview the user, and instead write the spec from what has already been discussed and from codebase exploration. | `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` (L3, L7) |
| `BU-P4-051` | Before drafting a spec, explore the repository to understand current state (if not already done), and use the project's domain glossary vocabulary and respect any ADRs in the touched area throughout the spec. | `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` (Process step 1, L13) |

### `10-sketch-seams`

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-052` | Before writing a spec's implementation section, sketch out the seams at which the feature will be tested, preferring existing seams and the highest possible seam, aiming for as few new seams as possible (ideally exactly one). | `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` (Process step 2, L15) |
| `BU-P4-053` | After sketching test seams for a spec, confirm with the user that the proposed seams match their expectations before finalizing the spec. | `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` (Process step 2, L17) |

### `10-sketch-seams` (helper invocation, folded from demoted `20-write-and-publish`)

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-054` | Write the spec using the fixed spec template, publish it to the project issue tracker, and apply the ready-for-agent triage label without requiring additional triage. | `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` (Process step 3, L19) |

## Adjudication A4

- **`20-write-and-publish` — DEMOTED.** Its CONTEXT.md carried only the §6.5 deterministic-machinery boilerplate ("candidate execute-stage workload") with no additional checkpoint argument (no "Additional note" section). Per A4's default rule, folded into `10-sketch-seams` as a helper invocation; `BU-P4-054` moves with it. The stage directory is removed; `10-sketch-seams` absorbs the workflow's terminal `promote` output disposition.

