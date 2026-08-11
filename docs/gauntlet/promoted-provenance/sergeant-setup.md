# Provenance — Sergeant Setup

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W3** `sergeant-setup`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-001` | Bootstrapping or repairing a Sergeant installation must be interactive and idempotent: orchestrate only supported commands, and surface any missing capability as a separate tracked issue rather than inventing an undocumented workaround. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 8-10) |
| `BU-P5-002` | sergeant-setup should be loaded when the user wants to install/configure Sergeant for the first time, register a new project or add repositories to an existing one, diagnose/repair a broken or incomplete installation, or verify that an existing setup is working. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 14-18) |
| `BU-P5-003` | sergeant-setup must not be loaded for documentation-only questions or questions about a specific command; those route to sergeant-help instead. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 20-21) |
| `BU-P1-137` | When the user wants to interactively install, configure, or repair a Sergeant installation, load the sergeant-setup procedure, which owns the interactive setup wizard, prerequisite detection, consent-gated install, project YAML interview, sync verification, and an optional treehouse prompt. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L118, Procedural skills table row) |

Routed here at N1 verifier round 2 (finding V3): `BU-P1-137` is AGENTS.md's own Procedural-skills-table row for this workflow, corroborating `BU-P5-002`'s trigger from a second, independent source document.

## Stages

### `00-detect-prerequisites`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-009` | Phase 1 classifies every checked prerequisite as present, installable (a supported bootstrap command exists), or unsupported (file a tracked issue). | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 56-57) |
| `BU-P5-010` | Required prerequisites for Sergeant are: bash 3.2+, git, an authenticated gh, tmux, yq, lsof, a td implementation supporting --description/--json/--work-dir, and at least one interactive agent (opencode, goose, or claude). | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 59-68) |
| `BU-P5-011` | Optional prerequisites (mise, treehouse, graphify, no-mistakes, node/npm) are skipped, not failed, when absent. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 70-75) |
| `BU-P5-013` | sergeant-setup does not continue past Phase 1 until every required prerequisite is present or the user explicitly accepts the risk of proceeding without it. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 91-92) |
| `BU-P6-003` | Dependency checking distinguishes required tools, whose absence fails the check with a nonzero exit, from optional tools, whose absence only warns, so an operator can tell which gaps block use of Sergeant from which merely reduce functionality. | `reference/sergeant-upstream/mise.toml` (tasks.check, L148-154) |
| `BU-P7-025` | The mise dependency check must fail (nonzero exit, mentioning the missing tool and 'MISSING') when `td` is absent, and separately must reject an installed `td` binary whose `create --help` output does not advertise `--work-dir` support as unsupported. | `reference/sergeant-upstream/tests/mise-check-test.sh` (lines 101-113) |
| `BU-P7-026` | The dependency check must accept any of OpenCode, Claude, or Goose as a satisfying agent harness, and must fail with a specific 'install OpenCode, Goose, or Claude' message when none is present. | `reference/sergeant-upstream/tests/mise-check-test.sh` (lines 115-140) |
| `BU-P8-041` | First-install requires a fixed set of tools (Bash>=3.2, Git, authenticated gh, tmux, yq, Python 3, lsof, Marcus td, and at least one of OpenCode/Goose/Claude Code as a persistent interactive worker terminal) plus optional tools (mise, Treehouse, Graphify, no-mistakes, Node/npm), and installation does not proceed past this check until an automated or manual verification confirms every required tool is present. | `reference/sergeant-upstream/docs/getting-started.md` (L6-27 (Prerequisites)) |
| `BU-P8-042` | Installation must not continue past the prerequisite stage until `td create --help` proves Marcus td support for --description, --json, and --work-dir, and at least one supported agent executable resolves on PATH. | `reference/sergeant-upstream/docs/getting-started.md` (L51-53) |

### `05-file-capability-gaps`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-012` | For each unsupported prerequisite, sergeant-setup drafts a td issue (title, description, acceptance criteria) and shows it for explicit y/yes approval before creating it; on decline it reports the gap in the summary and creates no tracked work. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 77-89) |

### `10-install-commands`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-015` | When cloning Sergeant, the destination path is asked for explicitly and the workflow waits for the user's answer before doing anything else. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 109-114) |
| `BU-P5-016` | The clone command is shown verbatim with the resolved destination and requires explicit y/N consent; any other response leaves the filesystem unchanged. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 116-122) |
| `BU-P5-017` | When mise is available, sergeant-setup first resolves the actual install directory from SGT_INSTALL_DIR, defaulting to $HOME/.local/bin. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 124-128) |
| `BU-P5-018` | sergeant-setup shows the resolved install target and requires explicit consent before running mise run install; if mise is unavailable or consent is declined, it instructs the user to symlink bin/ commands onto PATH manually and to verify the result before continuing. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 130-138) |
| `BU-P5-019` | After install, sergeant-setup verifies that at least sgt-list, sgt-context, sgt-dispatch, and sgt-watch resolve on PATH; missing commands and their expected source paths are reported, and the run stops if verification fails, resuming from Phase 2 on the next invocation. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 140-143) |
| `BU-P5-014` | For each installable prerequisite, sergeant-setup shows the exact installation command and requires explicit y/yes consent before running it; any other response leaves the system unchanged. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 94-103) |

### `20-global-config`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-020` | sergeant-setup checks for ~/.config/sergeant/config.yaml; if missing, it asks the user for a dev_root path, then shows a full preview of the file content and requires explicit y/N confirmation before writing anything. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 147-158) |
| `BU-P5-021` | If the global config is present and valid, sergeant-setup verifies dev_root is set and reports [ok] without further action. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (line 159) |
| `BU-P5-022` | If the global config exists but fails to parse as YAML (checked via yq), sergeant-setup reports the parse error and stops; it never overwrites the file without a timestamped backup, a diff preview, and explicit confirmation. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 160-162) |
| `BU-P8-044` | A global config.yaml sets one machine-wide dev_root used as the base for every relative repo path in every project YAML on that machine. | `reference/sergeant-upstream/docs/getting-started.md` (L85-94) |

### `30-project-interview`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-023` | If the project YAML already exists and the user wants to modify it, Phase 4 (the new-project interview) is skipped entirely in favor of Phase 5 (repair existing YAML). | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 166-167) |
| `BU-P5-024` | The new-project interview asks, in strict order and waiting for each answer before proceeding: project name (YAML filename stem, must match [a-z0-9_-]+), per-repository name/path/clone-URL/role/group, per-group description and shared agent_instructions, default agent instructions applied to every repository, project-level GitHub identity, and an optional Graphify output path. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 169-184) |
| `BU-P5-026` | Before writing the project YAML, sergeant-setup shows a complete preview of the file content and requires explicit confirmation; the file is written only after confirmation. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 186-195) |
| `BU-P5-027` | If a project YAML file already exists when Phase 4 would write a new one, a timestamped backup is created at <name>.yaml.bak.<timestamp> before writing. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 197-198) |
| `BU-P8-045` | Registering a first project requires the copied YAML's name to match its filename, every repo to have a unique name and correct path, clone URLs present wherever sgt-sync may need to clone, roles/groups that identify real ownership, and agent instructions that state commands and observable constraints rather than vague quality slogans. | `reference/sergeant-upstream/docs/getting-started.md` (L102-110) |

### `40-repair-existing`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-028` | Phase 5 (repair) first validates the existing project YAML with yq; if validation fails, it reports the parse error and stops without proceeding. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 204-205) |
| `BU-P5-029` | Phase 5 computes and displays a minimal diff between the current file content and the proposed changes before any write. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 206-207) |
| `BU-P5-030` | Phase 5 requires explicit confirmation before any write or backup; only after confirmation does it create a timestamped backup and then write the new content. The backup is never created before confirmation, changes are never applied on decline, and the backup is mandatory whenever a write happens -- it is never skipped even if asked. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 208-216) |
| `BU-P7-038` | Backup of an existing config file must be created only AFTER the user confirms applying changes, never before; the skill must not instruct pre-confirmation backup creation even framed as a safety measure. | `reference/sergeant-upstream/tests/sergeant-setup-test.sh` (lines 116-121) |

### `60-task-tracking-init`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-032` | For each registered repository, sergeant-setup checks td initialization via `td status --json --work-dir <repo-path>`; if not initialized, it shows and requires explicit consent for `td init --work-dir <repo-path>`, and reports the gap in the final summary if consent is declined. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 235-248) |
| `BU-P5-033` | sergeant-setup never initializes td in a repository that is not registered in the current project YAML. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 250-251) |
| `BU-P7-037` | td initialization, Graphify initialization, and Treehouse initialization each require an explicit confirmation prompt before Sergeant performs them; none may be silently auto-initialized. | `reference/sergeant-upstream/tests/sergeant-setup-test.sh` (lines 69-77) |
| `BU-P5-031` (folded helper: sync and verify, formerly `50-sync-and-verify`) | After the project YAML is written, sergeant-setup runs sgt-list, sgt-context <project>, sgt-status <project>, and sgt-sync <project> in that fixed order, stopping at the first failure with its full output and never advancing to the next command until the previous one succeeds. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 220-231) |

### `70-optional-capabilities`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-034` | If treehouse is on PATH, sergeant-setup offers to initialize Treehouse worktree pools with an explicit y/N prompt; it runs only on confirmation, skips silently on decline or absence, and never marks overall setup incomplete because Treehouse was skipped. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 255-263) |
| `BU-P5-035` | If graphify is on PATH and the project YAML declares graphify.output, sergeant-setup offers to run sgt-graphify with an explicit y/N prompt, skips silently on decline, and on a successful run requires both graph.json and GRAPH_REPORT.md to exist at the configured output path. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 267-275) |
| `BU-P6-018` | Initializing a treehouse worktree pool for a project's repos only acts on repos that are already cloned and skips (never fails) repos lacking a local clone, and it treats an already-present treehouse.toml as already-initialized rather than re-initializing. | `reference/sergeant-upstream/bin/sgt-treehouse-init` (L59-71) |
| `BU-P8-047` | Worktree pools via Treehouse are only initialized for repositories where they are explicitly desired, and any repository-owned treehouse.toml produced by that step is committed through normal review rather than treated as install-time throwaway state. | `reference/sergeant-upstream/docs/getting-started.md` (L144-151) |
| `BU-P8-048` | A project graph is only considered successfully published when both graph.json and GRAPH_REPORT.md exist at the configured output. | `reference/sergeant-upstream/docs/getting-started.md` (L153-161) |
| `BU-P5-007` (folded helper: completion summary, formerly `90-completion-summary`) | sergeant-setup maintains a visible numbered checklist: before each step it verifies whether the step is already complete and skips it without prompting if so; after each step it writes an [ok] or [skipped] status line; when a phase fails, the run stops with actionable output identifying the last completed phase. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 45-49) |
| `BU-P5-036` (folded helper: completion summary) | The Phase 10 completion summary lists every checklist item as exactly one of [ok], [skipped], or [issue: <td-id>]. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 279-292) |
| `BU-P5-037` (folded helper: completion summary) | Re-running sergeant-setup after a successful setup must produce the same final state; no phase destroys existing working configuration merely to reach the same end state it already represents. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 296-298) |
| `BU-P5-008` (folded helper: completion summary) | Re-running sergeant-setup after a partial run restarts the checklist from Phase 1 but skips every phase that already passes verification; resumability works by re-checking each phase before acting, not by persisting run state between invocations. | `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 49-52) |
| `BU-P7-039` (folded helper: completion summary) | The setup skill's failure table must cover every required failure mode (missing/uninstallable prerequisite, declined consent, YAML parse error, unsupported capability, sync failure, partial setup on exit) with a stop condition, and must not indicate auto-continuing past a failed sync. | `reference/sergeant-upstream/tests/sergeant-setup-test.sh` (lines 94-106) |
| `BU-P7-040` (folded helper: completion summary) | The setup skill must cover a fixed ordered checklist of phases — detect prerequisites, install command links, write global config, run an interview, repair existing YAML, sync and verify, optionally initialize treehouse — and each mutating phase must be idempotent. | `reference/sergeant-upstream/tests/sergeant-setup-test.sh` (lines 79-88) |
| `BU-P8-049` (folded helper: completion summary) | Worker briefs already discover their required workflow skills from this repository's own vendored .agents/skills/ tree, so installing engineering skills for dispatched workers is not a separate manual step beyond having the repository itself; any additional locally-installed skills must still come from reviewed sources. | `reference/sergeant-upstream/docs/getting-started.md` (L163-171) |
| `BU-P8-051` (folded helper: completion summary) | The getting-started checklist's own definition of a completed installation is a fixed nine-item checklist: required commands resolve, the coordinator runs in a tmux pane, sgt-list shows the project exactly once, sgt-context resolves every owning repo and instruction layer, required repos are cloned, Marcus td is installed and initialized, GitHub CLI can access required repos, optional Treehouse/Graphify features pass their own verification, and required (plus any reviewed extra) skills are present. | `reference/sergeant-upstream/docs/getting-started.md` (L192-202 (Completion checklist)) |

## Adjudication A4 (N1-BH-02 sweep)

Original stages: `00-detect-prerequisites`, `05-file-capability-gaps`, `10-install-commands`, `20-global-config`, `30-project-interview`, `40-repair-existing`, `50-sync-and-verify`, `60-task-tracking-init`, `70-optional-capabilities`, `90-completion-summary`. Three stages were originally classified "Deterministic-machinery candidate" (§6.5): `00-detect-prerequisites`, `50-sync-and-verify`, `90-completion-summary`.

- `50-sync-and-verify` and `90-completion-summary` carried no argument beyond the §6.5 boilerplate — neither had an "Additional note" checkpoint argument — so both demote by A4's default rule. `50-sync-and-verify` folds forward into `60-task-tracking-init`; `90-completion-summary` folds backward into `70-optional-capabilities` (the workflow's new terminal stage, since `90` was last and no later judgment stage exists to fold into instead).
- `00-detect-prerequisites` carried an Additional note, but on inspection that note is about a citation cross-reference (`BU-P7-026`'s shared ownership with `skill-discovery`), not an argument for why the stage belongs at §6.3 versus a helper. Independently applying §6.3's reimplementation test to the stage's own behavior contract: `BU-P5-013` requires the actor to stop and ask the user to accept risk when a required prerequisite is missing — an actor decision, not incidental machinery, and structurally identical to the confirm-then-act pattern every other actor stage in this package already carries. Swapping the detection tool's implementation would leave that risk-acceptance gate unchanged, which is exactly the reimplementation test's affirmative case. **Kept, reclassified from §6.5 to §6.4.** The original Additional note about `BU-P7-026` is preserved in the stage's `CONTEXT.md`, unaffected by this reclassification.

**Decision:** stage count drops from 10 to 8: `00-detect-prerequisites` (reclassified, not demoted), `05-file-capability-gaps`, `10-install-commands`, `20-global-config`, `30-project-interview`, `40-repair-existing`, `60-task-tracking-init` (absorbs `50`), `70-optional-capabilities` (absorbs `90`). The behavior units are not deleted — see the surviving stages' "Helpers (folded per N1 adjudication A4)" sections.

## Notes

**Demoted/merged candidates:** `sergeant-install` (P8, from `docs/getting-started.md`) documents the identical procedure as a checklist rather than as phases and is merged into this workflow (conflict X10). The two partitions' prerequisite lists differ (P5 omits Python 3 and Node; P8 adds them) — that difference is unresolved and is preserved as an open item in provenance.md rather than silently picking one list.

**Standing constraints** (`BU-P5-004`, `BU-P5-005`, `BU-P7-036`, `BU-P5-006`): Write only to Sergeant-owned paths. Never write to other tools' config surfaces. Never auto-initialize external tools without explicit consent. These apply across every stage of this workflow, not to one checkpoint — they belong in this workflow's `_config/` (Layer 3), not in any single stage's Inputs table.

**Promotion note (`docs/icm/promotion-spec-2026-08-11.md` §1, added at archiving, not part of the N1 adjudication above):** this package's true closing stage per `workflow.toml`'s own stage order is `70-optional-capabilities` (absorbs `90-completion-summary` per A4), which declares an `evidence` output and names no finalize step (no `scripts/finalize.py` or equivalent) — one of the 30 of 34 corpus packages in that shape, per the promotion spec's D9 observation; disposition is left to human review at merge time, not resolved here.

