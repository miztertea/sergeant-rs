# Provenance — Load Project

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W1** `load-project`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-010` | Listing known projects treats the special file config.yaml under the Sergeant config directory as global configuration, not a project, and excludes it from the project listing. | `reference/sergeant-upstream/bin/sgt-list` (L21-22) |
| `BU-P6-011` | Listing projects is a bounded, independently invocable procedure: enumerate every project YAML under the Sergeant config directory (other than the reserved config.yaml) and print each with its optional description, failing with a diagnosis if the config directory or any project is absent. | `reference/sergeant-upstream/bin/sgt-list` (L2-3) |
| `BU-P5-040` | cross-repo-work requires load-project to have already resolved repository paths, roles, groups, and instructions. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 16-17) |
| `BU-P5-090` | load-project resolves Sergeant project ownership, configuration, and paths before any work begins. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 8) |
| `BU-P5-091` | load-project is loaded when a project is named, registered, edited, synced, or graphed, or when repository ownership is not already established by sgt-context output. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 12-13) |
| `BU-P1-132` | When a project is named, registered, edited, synced, or graphed, load the load-project procedure, which owns registry lookup, schema, context loading, project edits, sync, and project Graphify. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L113, Procedural skills table row) |

Routed here at N1 verifier round 2 (finding V3): `BU-P1-132` is AGENTS.md's own Procedural-skills-table row for this workflow, corroborating `BU-P5-091`'s trigger from a second, independent source document.

## Stages

### `00-resolve-project-name`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-092` | If the project name is unknown, load-project runs sgt-list and requires an exact registered name before proceeding. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 17-18) |
| `BU-P5-108` | If a named project is unregistered, load-project stops and asks whether to register it, rather than proceeding on an assumed project. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 72) |

### `10-resolve-context`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-093` | load-project runs sgt-context <project> and records the owning repositories, resolved absolute paths and clone state, group membership and roles, instructions inherited in defaults-then-group-then-repository order, and any configured Graphify output and included groups. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 19-25) |
| `BU-P5-094` | A raw project YAML is read directly only when a required field is absent from the resolved sgt-context output. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 26-27) |
| `BU-P5-096` | Completion evidence for load-project is the sgt-context block showing every owning repository as cloned, plus the instructions and paths that will govern execution. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 31-32) |
| `BU-P6-021` | The generated context block reports each repo's clone status as one of exactly three states — cloned with its current branch, present but not a git repo, or not cloned at all — so a reading agent never has to infer clone state from absence. | `reference/sergeant-upstream/bin/sgt-context` (L67-75) |
| `BU-P6-022` | If a project has a knowledge graph configured, the emitted context block tells the agent whether a built graph report already exists to read or names the exact command to build one, rather than silently omitting graph information. | `reference/sergeant-upstream/bin/sgt-context` (L136-139) |
| `BU-P5-111` | If sgt-context and the raw project YAML disagree, the sgt-context failure is treated as blocking and the YAML is preserved for diagnosis. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 75) |

### `20-register-or-edit`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-097` | Registering or editing a project starts by reading docs/schema.md and, when editing, the existing YAML. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 36-38) |
| `BU-P5-098` | Project YAML is written only to ~/.config/sergeant/<project>.yaml, and credentials, tokens, or secret values are never placed in it. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 39-40) |
| `BU-P5-099` | Repository paths in project YAML are always either absolute or relative to the global dev_root. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 41) |
| `BU-P5-101` | After writing project YAML, load-project runs sgt-list and requires the project to appear exactly once, then runs sgt-context <project> and requires every edited field needed by agents to appear in the resolved output. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 44-46) |
| `BU-P5-103` | If validation fails after a registration/edit, load-project restores the prior YAML or leaves the new file uncommitted, and reports the exact command error. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 48-49) |

### `20-register-or-edit` (folded helpers)

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-095` (folded helper: sync repositories, formerly `30-sync-repositories`) | A missing required repository is synced only once the requested work actually requires it, via sgt-sync <project>; the workflow stops if cloning or pulling fails. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 28-29) |
| `BU-P5-102` (folded helper: sync repositories) | sgt-sync <project> runs only when repositories actually need cloning or refreshing, not unconditionally after every edit. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 47) |
| `BU-P6-013` (folded helper: sync repositories) | Syncing a project's repos treats three distinct repo states differently: an already-cloned repo on a named branch is pulled fast-forward-only (never merged), an existing non-git directory is left untouched with a warning, and a missing repo with a configured URL is cloned; every other combination is reported and skipped rather than guessed at. | `reference/sergeant-upstream/bin/sgt-sync` (L30-45) |
| `BU-P6-014` (folded helper: sync repositories) | A repo pull only proceeds fast-forward and is skipped with a warning (never force-merged or rebased) when the branch has diverged or has no upstream, and a detached HEAD is skipped outright rather than guessed at. | `reference/sergeant-upstream/bin/sgt-sync` (L33-39) |
| `BU-P5-109` (folded helper: sync repositories) | If a required repository entry has no clone URL, load-project stops with the repository name and the missing field named explicitly. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 73) |
| `BU-P5-110` (folded helper: sync repositories) | If a required executable is missing, load-project reports the executable and a platform-neutral installation requirement, and never invents a fallback parser. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 74) |
| `BU-P6-012` (folded helper: report state, formerly `40-report-state`) | Showing a project's status walks every configured repo and reports, per repo, whether it is cloned, its current branch, its working-tree cleanliness, and how far ahead/behind its upstream it is — never mutating anything. | `reference/sergeant-upstream/bin/sgt-status` (L1-2) |
| `BU-P6-035` (folded helper: report state) | Listing tracked work across a project defaults to showing only open tasks and can be narrowed by status, priority, or an explicit repo subset; every repo is queried independently and repos without an initialized task database are silently skipped rather than erroring the whole listing. | `reference/sergeant-upstream/bin/sgt-td-list` (L2-13) |

## Adjudication A4 (N1-BH-02 sweep)

Original stages ended in `30-sync-repositories` and `40-report-state`.

- `30-sync-repositories` carried only the §6.5 deterministic-machinery boilerplate — no "Additional note" checkpoint argument — so it demotes by A4's default rule.
- `40-report-state` carried an Additional note: "Borderline per synthesis.md (closer to a query than a checkpoint); kept as a stage because operators do care whether it succeeded before planning." Judged against §6.3's reimplementation test: replacing the status/listing implementation with a different tool tomorrow would leave the surrounding checkpoint (load-project's completion) entirely unchanged — the note's own framing already concedes "closer to a query than a checkpoint," and "operators do care" is not a discriminating rationale (it does not distinguish this stage from any other read-only report). The argument fails the test; **demoted**.

**Decision:** both fold as helper invocations into `20-register-or-edit`, which becomes the workflow's terminal stage (there is no later judgment-bearing stage in this package to fold into instead). The behavior units are not deleted — see `20-register-or-edit/CONTEXT.md`'s "Helpers (folded per N1 adjudication A4)" section. Stage count drops from 5 to 3.

## Notes

**Demoted/merged candidates:** `list-projects` (BU-P6-010/011), `project-status` (BU-P6-012), `project-sync` (BU-P6-013/014), and `project-task-list` (BU-P6-035) were each extracted as standalone workflows by one partition (P6) but are command surfaces, not procedures with a bounded outcome and completion condition (§6.2) — folded into this workflow's stages instead. See synthesis.md conflict X11.

## Curation note (promotion gate-record completion, 2026-08-11)

`load-project`'s promotion commit (e187c72) asserted that an
engine-acceptance gate ran ("run separately against a private scratch
subject repo and data dir with the sgt binary from sergeant-runb") but
recorded none of promotion-spec §3's five required assertions and no
daemon-stop confirmation. This note completes the record with a fresh run:
`docs/icm/promotion-spec-2026-08-11.md` §3's procedure, run 2026-08-11
against `/home/miztertea/sergeant-runb/target/debug/sgt` in a
package-private scratch subject repo and data dir, `SGT_FAKE_SCRIPT`
unset — `work.state == "completed"`; one `workflow.bound` whose
`stage_bindings` matched `workflow.toml`'s three stages
(`00-resolve-project-name`, `10-resolve-context`, `20-register-or-edit`)
in order; matching `stage.entered`/`stage.completed` pairs in that order;
one terminal `work.completed` with `stages == 3`; three distinct
`execution_id`s (`01KZREPRKD2F82A224XK4CXFMH`, `01KZREPRKDH04Q5WPRVHHDTTTJ`,
`01KZREPRKEF146YF6FQE6D412E`). Daemon stopped and pgrep-confirmed gone
before teardown. Per spec §1's D9 observation, the closing stage
(`20-register-or-edit`) declares an `evidence`-dispositioned output with
no finalize step named — not a promotion blocker, recorded here rather
than left implicit.

