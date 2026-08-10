# Source Inventory — `reference/sergeant-upstream`

Part of the N1 reference corpus (`docs/gauntlet/contracts/N1.md`, Inventory step
of §8.3's method). Enumerates **every file** under
`reference/sergeant-upstream`, pinned at `f430cfd4f90174a98adbd7abebbece6303817929`
(per `reference/UPSTREAM.md`), vendored 2026-08-08.

`find reference/sergeant-upstream -type f | wc -l` → **179**. This inventory
lists all 179. It does not separately enumerate `.claude/skills/*` — that
directory holds 17 symlinks (not regular files, so outside the 179) that mirror
`.agents/skills/*` 1:1 for Claude Code's skill-discovery path; each points at a
directory already inventoried under `.agents/skills/`, so its disposition and
partition are identical to its target and are not restated.

Nothing under `reference/` is edited by this document or by anything it
produces — read-only evidence per `docs/gauntlet/contracts/N1.md`'s non-goals.

## Legend

| Disposition | Meaning |
|---|---|
| **decompose** | Behavior-bearing. Goes to extraction (behavior-units.ndjson) in a later N1/N2 stage. |
| **helper-evidence** | Deterministic mechanics (ladder §6.5/§6.6) that inform `helper-map.md`/`shared-context-map.md` but are not themselves a durable checkpoint or procedural outcome. |
| **obsolete-candidate** | A mechanism the Rust runtime structurally replaces already, per an existing invariant or an already-settled deviation-register ruling — cited per row. *Candidate* because the final ruling belongs to classification/`obsolete-mechanisms.md`, not this inventory. |
| **reference-only** | Excluded from extraction: research/PRDs/audits/licenses/binary/bytecode. Reason given per row. |

Where a **decompose** row's mechanism is unusually dense with old Sergeant's
tmux/sentinel/worker architecture (§8.2's "designated obsolete-mechanism stress
test"), the note says so explicitly — it stays **decompose** so the intent
inside is not lost to extraction, but flags it as a priority input to
`obsolete-mechanisms.md` at the classify stage rather than pre-ruling here.

---

## Root

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `AGENTS.md` | Stable coordinator/executor role instructions: resolve project context via `sgt-context`, default to coordinating multi-repo work, direct-mode branch/PR discipline. | decompose | **P1** |
| `README.md` | Project positioning/genesis (inspired by `firstmate`); one-paragraph pitch for what Sergeant is and isn't. | decompose | **P1** |
| `LICENSE` | MIT license text, Copyright Lars Cromley. | reference-only | License text, not behavior. |
| `.gitignore` | Two ignore patterns (`.DS_Store`, `.todos/`). | reference-only | Trivial repo hygiene, no procedural content. |
| `Dockerfile.test` | Minimal Debian-bookworm image for running the drain test suite reproducibly (paired with an Alpine `bash:3.2` image for the Bash-3.2 regression pass). | helper-evidence | Deterministic test-environment machinery, not itself a checkpoint. |
| `mise.toml` | Task runner config: `install` task symlinks `sgt-*`/`_sgt-*` scripts and the pre-push hook into PATH/`.git/hooks`; other tasks drive the Docker-based drain-test matrix. | decompose | **P6** |
| `opencode.json` | Tells OpenCode where to discover skills (`.agents/skills`, `skills`). | decompose | **P6** |

## `.agents/skills/` — provenance records (not skill-scoped)

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `.agents/skills/PROVENANCE.md` | Records that the 17 upstream skill directories were imported unchanged from `mattpocock/skills` via `~/.agents/.skill-lock.json`; last-synced date. | reference-only | Attribution/provenance record, not behavior. |
| `.agents/skills/THIRD_PARTY_NOTICES.md` | Lists the redistributed skills and their upstream MIT license. | reference-only | License notice, not behavior. |

## `.agents/skills/` — dev-skills-a (P2)

Standards/spec review, implementation, TDD, and bug diagnosis.

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `.agents/skills/code-review/SKILL.md` | Reviews a diff on two parallel axes (Standards, Spec) via sub-agents, reported side by side. | decompose | **P2** |
| `.agents/skills/code-review/agents/openai.yaml` | Cross-harness display metadata (`display_name`, `short_description`) so OpenAI-style agents can present the skill. | helper-evidence | Metadata-shape convention, informs `helper-map.md`; not procedure itself. |
| `.agents/skills/diagnosing-bugs/SKILL.md` | Diagnosis loop for hard bugs/perf regressions: feedback loop, reproduce/minimize, hypothesize, instrument, fix with regression coverage, clean up. Explicitly called out in the proposal (§8.2) as "a strong low-ambiguity reference workflow." | decompose | **P2** |
| `.agents/skills/diagnosing-bugs/agents/openai.yaml` | Cross-harness display metadata for this skill. | helper-evidence | Same as above. |
| `.agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh` | Copy-and-edit Bash template implementing a human-in-the-loop `step`/`capture` reproduction loop; agent runs it, user follows terminal prompts. | helper-evidence | Deterministic machinery subordinate to the "reproduce" checkpoint (ladder §6.5). |
| `.agents/skills/implement/SKILL.md` | Implements a piece of work from a spec or ticket set. | decompose | **P2** |
| `.agents/skills/implement/agents/openai.yaml` | Cross-harness display metadata. | helper-evidence | Same pattern. |
| `.agents/skills/no-mistakes/SKILL.md` | Coordinator-only reference for the no-mistakes shipping-gate contract: pipeline semantics, gate meaning, findings policy; explicit "do not invoke from a worker pane" rule. Proposal §8.2 flags this as likely to "produce legitimate pressure for shared procedures, independent actor selection, and deterministic validation stages." | decompose | **P2** |
| `.agents/skills/tdd/SKILL.md` | Test-driven development: red-green-refactor workflow for features/bug fixes. | decompose | **P2** |
| `.agents/skills/tdd/agents/openai.yaml` | Cross-harness display metadata. | helper-evidence | Same pattern. |
| `.agents/skills/tdd/mocking.md` | Rule set for what to mock (system boundaries only) vs. never mock (own code, internal collaborators). | decompose | **P2** |
| `.agents/skills/tdd/tests.md` | Good-vs-bad test examples; prefers integration-style tests over internal mocks. | decompose | **P2** |

## `.agents/skills/` — dev-skills-b (P3)

Prototyping, research, triage, and adversarial/interview loops.

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `.agents/skills/grill-with-docs/SKILL.md` | Interview loop that stress-tests a plan/design while also producing ADRs/glossary as it goes. | decompose | **P3** |
| `.agents/skills/grill-with-docs/agents/openai.yaml` | Cross-harness display metadata. | helper-evidence | Same pattern. |
| `.agents/skills/grilling/SKILL.md` | Relentless interview of the user about a plan/decision/idea; trigger-phrase driven. | decompose | **P3** |
| `.agents/skills/grilling/agents/openai.yaml` | Cross-harness display metadata. | helper-evidence | Same pattern. |
| `.agents/skills/prototype/SKILL.md` | Builds a throwaway prototype to answer a design question; branches between logic and UI prototypes. Proposal §8.2: "stresses conditional procedure without requiring the runtime to become a DAG immediately." | decompose | **P3** |
| `.agents/skills/prototype/agents/openai.yaml` | Cross-harness display metadata. | helper-evidence | Same pattern. |
| `.agents/skills/prototype/LOGIC.md` | Sub-procedure: tiny interactive terminal app to hand-drive a state model, for business-logic/state-transition questions. | decompose | **P3** |
| `.agents/skills/prototype/UI.md` | Sub-procedure: generate several radically different UI variants on one route, switchable in-browser, for "what should this look like" questions. | decompose | **P3** |
| `.agents/skills/research/SKILL.md` | Investigates a question against high-trust primary sources and captures findings as a Markdown file; delegable to a background agent. | decompose | **P3** |
| `.agents/skills/research/agents/openai.yaml` | Cross-harness display metadata. | helper-evidence | Same pattern. |
| `.agents/skills/resolving-merge-conflicts/SKILL.md` | Resolves an in-progress git merge/rebase conflict. | decompose | **P3** |
| `.agents/skills/resolving-merge-conflicts/agents/openai.yaml` | Cross-harness display metadata. | helper-evidence | Same pattern. |
| `.agents/skills/triage/SKILL.md` | State machine moving issues/external PRs through categorize → verify → grill (if needed) → agent-ready brief. | decompose | **P3** |
| `.agents/skills/triage/agents/openai.yaml` | Cross-harness display metadata. | helper-evidence | Same pattern. |
| `.agents/skills/triage/AGENT-BRIEF.md` | Format/principles for writing a durable agent brief (durability over precision; no stale file-path references) posted on an issue/PR at `ready-for-agent`. | decompose | **P3** |
| `.agents/skills/triage/OUT-OF-SCOPE.md` | `.out-of-scope/` knowledge-base convention: persistent record of rejected feature requests, for institutional memory and dedup. | decompose | **P3** |

## `.agents/skills/` — design-skills (P4)

Domain/module design and spec/ticket synthesis.

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `.agents/skills/codebase-design/SKILL.md` | Shared vocabulary for designing deep modules (module, interface, seam, adapter); used when finding deepening opportunities or improving testability. | decompose | **P4** |
| `.agents/skills/codebase-design/agents/openai.yaml` | Cross-harness display metadata. | helper-evidence | Same pattern. |
| `.agents/skills/codebase-design/DEEPENING.md` | How to safely deepen a cluster of shallow modules; classifies dependencies (in-process / local-substitutable / etc.) to decide seam placement. | decompose | **P4** |
| `.agents/skills/codebase-design/DESIGN-IT-TWICE.md` | Parallel sub-agent pattern for exploring alternative interfaces on a deepening candidate ("Design It Twice", after Ousterhout). | decompose | **P4** |
| `.agents/skills/domain-modeling/SKILL.md` | Builds/sharpens a project's domain model and ubiquitous language; records architectural decisions. | decompose | **P4** |
| `.agents/skills/domain-modeling/agents/openai.yaml` | Cross-harness display metadata. | helper-evidence | Same pattern. |
| `.agents/skills/domain-modeling/ADR-FORMAT.md` | ADR file/naming convention (`docs/adr/0001-slug.md`), minimal single-paragraph template. | decompose | **P4** |
| `.agents/skills/domain-modeling/CONTEXT-FORMAT.md` | `CONTEXT.md` structure convention: name, description, per-term "Language"/"Avoid" glossary entries. | decompose | **P4** |
| `.agents/skills/to-spec/SKILL.md` | Turns the current conversation into a spec and publishes it to the issue tracker; synthesis only, no interview. | decompose | **P4** |
| `.agents/skills/to-spec/agents/openai.yaml` | Cross-harness display metadata. | helper-evidence | Same pattern. |
| `.agents/skills/to-tickets/SKILL.md` | Breaks a plan/spec/investigation/PR/conversation into dependency-aware tracer-bullet work items for Sergeant and `td`. | decompose | **P4** |
| `.agents/skills/wayfinder/SKILL.md` | Plans work too large for one session as a shared map of decision tickets on the issue tracker, resolved one at a time. | decompose | **P4** |
| `.agents/skills/wayfinder/agents/openai.yaml` | Cross-harness display metadata. | helper-evidence | Same pattern. |

## `.agents/skills/sergeant-setup/` and `skills/` — ops-skills (P5)

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `.agents/skills/sergeant-setup/SKILL.md` | Interactive, idempotent bootstrap/repair of a Sergeant installation; required vs. optional prerequisites, platform-sensitive. Proposal §8.2: "useful evidence for `needs_input`, user authority, platform detection." | decompose | **P5** |
| `skills/cross-repo-work/SKILL.md` | Decomposes ownership, dependency order, merge order, and acceptance for outcomes owned by more than one repo, before dispatch. | decompose | **P5** |
| `skills/dispatch/SKILL.md` | Plans and executes cross-repo work by dispatching one autonomous subagent per repo, each in an isolated git worktree, via tmux/sentinel/callback/worker-lifecycle machinery. Proposal §8.2: **the designated obsolete-mechanism stress test** — highest-priority `obsolete-mechanisms.md` input. | decompose | **P5** (flagged: obsolete-mechanism stress-test primary artifact) |
| `skills/load-project/SKILL.md` | Resolves a named project's repository ownership, config, paths, and inherited instructions when a project is named/registered/edited/synced/graphed. | decompose | **P5** |
| `skills/sergeant-help/SKILL.md` | Answers Sergeant install/setup/usage/skills/troubleshooting questions from repo-owned docs; explicitly not a substitute for `load-project`/`cross-repo-work`/`dispatch`/`wiki` once execution is requested. | decompose | **P5** |
| `skills/wiki/SKILL.md` | Maintains Sergeant's automatic activity captures (`~/wiki/.captures/`) and curated daily session digest (`~/wiki/`, governed by `~/wiki/SCHEMA.md`). | decompose | **P5** |

## `bin/` — shared libraries (sourced, not executed directly)

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `bin/_sgt-bash-version.sh` | Shared minimum-Bash-version check sourced by every `sgt-*` entry point. | helper-evidence | Deterministic version gate (ladder §6.5/§6.6). |
| `bin/_sgt-lib.sh` | Shared helpers (`_die`, `_info`, `_require_*`, path resolution) and `SGT_*` env vars sourced by all `sgt-*` scripts. | helper-evidence | Canonical shared helper library. |
| `bin/_sgt-intent.sh` | Shared helper: content-hash revisioning of an intent file (`shasum`/`sha256sum`), plus a capability probe that reads `no-mistakes axi run --help` to discover accepted flags (help output is the only capability surface no-mistakes exposes). | decompose | **P6** — capability-probe-via-help-output is directly relevant to the proposal's §17 Harness Registry/doctor design. |
| `bin/_sgt-drain.sh` | Drain-state helpers: global/project drain files under `$SERGEANT_CONFIG/drain/` (key=value format), PID-liveness checks, lock-file admission control gating new dispatch/relaunch; also owns the `_sgt_claude_bg_*` bg-session stop helpers referenced by the (superseded) Claude Background Harness mechanism. | decompose | **P6** — majority is a portable drain/admission-control invariant; the embedded `_sgt_claude_bg_*` functions are D2-superseded mechanism, noted for `obsolete-mechanisms.md`. |
| `bin/_sgt-harness.sh` | Single registry (`<harness>:<probe>:<launch-args>`) driving the accepted-harness capability gate, tmux-pane readiness probe, and launch invocation together — replacing three previously-drifting definitions (cites GH #156, #175). Readiness answers "will a keystroke reach a running harness in this pane," via two consecutive render observations rather than a fixed delay. | decompose | **P6** — flagged: dense with tmux-pane mechanism (obsolete-mechanism stress-test input), but the "one registry, not three drifting definitions" principle and the probe-not-string-match design are portable engine-pressure evidence for §17. |
| `bin/_sgt-response-lock.sh` | Shared serialization/archive format for response publication and consumption: a consumed response is a directory of four fields (`body`, `gate_generation`, `applied_status`, `proof`); `sgt-ack-response` publishes, `sgt-cleanup` validates before retiring fleet state. | decompose | **P6** — flagged: file-lock/archive mechanism is a stress-test input, but the four-field durable-response shape is useful evidence for what a "delivered response" fact must contain. |
| `bin/_sgt-review-axes.sh` | Canonical independent-review axis/severity vocabulary, sourced by both `sgt-dispatch` (what it instructs workers to produce) and `sgt-review-findings` (what it accepts). | helper-evidence | Single source of truth for a shared vocabulary (ladder §6.6); the vocabulary itself is echoed inside the `no-mistakes`/`code-review` skills already captured there. |

## `bin/` — commands

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `bin/sgt-ack-response` | Acknowledges and clears one consumed worker response. | decompose | **P6** |
| `bin/sgt-callback` (Python) | Durable, profile-bound callback events for fleet tasks: trusted local executables selected by profile name, delivery left to the consumer. | decompose | **P6** |
| `bin/sgt-cleanup` | Removes worktrees and fleet state for a completed task; handles both treehouse-leased and plain git worktrees; requires fleet state and worktree on the same filesystem for atomic publish. | decompose | **P6** |
| `bin/sgt-context` | Emits a structured agent context block for a project: reads project YAML, resolves instruction layering (defaults → group → repo). | decompose | **P6** — direct evidence for the ICM `shared-context-map.md`. |
| `bin/sgt-dag-dispatch-hook` | Stage hook invoked by the external `dagr` tool when a DAG stage becomes ready: calls `sgt-dispatch`, writes `dagr_run_id`/`dagr_stage_id` tracking files so `sgt-watch` can auto-advance the DAG on completion. | decompose | **P6** — unaddressed engine-gap evidence (DAG/branching does not exist in sergeant-rs yet, per proposal §7.7 non-goal), not a structural replacement. |
| `bin/sgt-dag-run` | Creates/starts a `dagr` run from a project YAML's `dag:` block (named stages, deps, per-stage `td`/repos/brief), dispatching all initially-ready stages. | decompose | **P6** — same engine-gap relevance as above. |
| `bin/sgt-dispatch` | Core dispatch procedure: creates an isolated git worktree per repo, writes a mission brief, spawns an agent in a local tmux window. | decompose | **P6** — the other half of the dispatch obsolete-mechanism stress test (with `skills/dispatch/SKILL.md`). |
| `bin/sgt-drain` | Sets or removes a persistent drain (project or global); while drained, new pane starts from response-driven relaunch and stall recovery are refused; responses are stored generation-safely for later delivery. | decompose | **P6** |
| `bin/sgt-drain-force` | Force-stops workers that failed cooperative drain; requires an active drain and explicit `--yes`/`--dry-run`; displays exact identity before stopping. | decompose | **P6** |
| `bin/sgt-graphify` | Runs `graphify` across every repo in a project and merges results into one project graph, published atomically. | decompose | **P6** |
| `bin/sgt-interactive-worker` | Owns one persistent interactive agent pane for the life of a worker. | decompose | **P6** — flagged: this file's unifying purpose (own a tmux pane) is exactly what the daemon's exclusive process-handle ownership (CLAUDE.md "One owner") plus D2's headless `-p`/`--resume` turn model structurally replace; its embedded checkpoints (readiness, drain, escalation, recovery) are still real decompose targets, so kept in — not pre-excluded — pending classify-stage adjudication. |
| `bin/sgt-list` | Lists all known Sergeant projects from `~/.config/sergeant/`. | decompose | **P6** |
| `bin/sgt-no-mistakes-finding` | Applies a disposition to one no-mistakes finding. | decompose | **P6** |
| `bin/sgt-notify` | Injects a worker update (escalation or terminal outcome) into the primary Sergeant session; default transport is a metadata-only wake marker for the durable fleet watcher, with raw tmux injection as a compatibility option. | decompose | **P6** |
| `bin/sgt-recover` | Attempts one bounded stall recovery for a stalled in-progress worker: kills the stalled pane, relaunches a fresh worker, atomically updates fleet metadata, delivers a recovery notification. | decompose | **P6** — the "bounded recovery, atomic metadata update" shape maps directly onto sergeant-rs's own "ambiguity fails closed" invariant (CLAUDE.md §4.3/recovery.rs). |
| `bin/sgt-respond` | Delivers a response and resumes a dead waiting worker when needed. | decompose | **P6** — precursor evidence for `needs_input`/waiting semantics (proposal §15.4). |
| `bin/sgt-review-findings` | Routes structured independent-review findings to `td`. | decompose | **P6** |
| `bin/sgt-status` | Shows git status across every repo in a project. | decompose | **P6** |
| `bin/sgt-sync` | Clones missing repos and pulls existing ones for a project. | decompose | **P6** |
| `bin/sgt-td-create` | Creates one `td` task per target repo for a cross-repo brief, all-or-nothing; returns created task IDs for `sgt-dispatch`. | decompose | **P6** |
| `bin/sgt-td-list` | Shows `td` tasks across every repo in a project, unified. | decompose | **P6** |
| `bin/sgt-td-memory` | Records non-secret worker recovery pointers in `td`. | decompose | **P6** |
| `bin/sgt-treehouse-init` | Initializes treehouse worktree pools in every repo of a project lacking a `treehouse.toml`; optional group filter. | decompose | **P6** |
| `bin/sgt-undrain` | Removes a drain record, project-scoped or global. | decompose | **P6** |
| `bin/sgt-validate` | Launches coordinator-owned no-mistakes beside an interactive worker. | decompose | **P6** — the `no-mistakes` stress case named directly in the N1 contract's Method section. |
| `bin/sgt-validation-worker` | Runs coordinator-owned no-mistakes in an interactive pane. | decompose | **P6** — same `no-mistakes` stress case. |
| `bin/sgt-wake` | Evaluates a durable wake condition (read from `.sergeant-wake-condition` in the worktree) and resumes a waiting worker. | decompose | **P6** — precursor evidence for waiting/`needs_input` semantics. |
| `bin/sgt-watch` | Monitors a dispatched fleet and reports outcomes. | decompose | **P6** — durable "observe fleet state, report terminal outcomes" outcome maps to the proposal's API/SSE + TUI/dashboard concerns even though today's mechanism is tmux/pane introspection. |
| `bin/wiki-daily-digest` | Synthesizes a daily wiki session digest from opencode/goose/claude session histories, enriched with merged PRs and `td` completions, via the Anthropic API against `~/wiki/SCHEMA.md`. | decompose | **P6** |
| `bin/__pycache__/sgt-callbackcpython-312.pyc` | Compiled Python bytecode cache for `bin/sgt-callback`. | reference-only | Binary build artifact, not source; `bin/sgt-callback` itself is already inventoried. |

## `docs/` — root-level

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `docs/README.md` | Documentation index: "Sergeant is a single-user, local-first project orchestrator..."; per-developer independent installation model. | decompose | **P8** |
| `docs/what-is-sergeant.md` | High-level model: repositories that belong together, instruction layering, work tracking, worker isolation/observation. | decompose | **P1** |
| `docs/skills.md` | Skill-loading policy: skills load only when their trigger applies; review sources before installing/updating. | decompose | **P1** |
| `docs/repo-scoped-skills.md` | Explains `.agents/skills/` as the canonical Agent Skills tree Codex discovers directly, vendoring worker-brief-required skills. | decompose | **P1** |
| `docs/callbacks.md` | Durable Callback Protocol v1: gives an external request a durable return path independent of a coordinator pane, OpenCode session, or model turn; profile names select trusted executables. | decompose | **P8** |
| `docs/schema.md` | Project YAML schema: lives at `~/.config/sergeant/<name>.yaml`, filename is the project identifier. | decompose | **P8** |
| `docs/getting-started.md` | Install checklist for one local user plus first-project registration. | decompose | **P8** |
| `docs/using-sergeant.md` | Command-usage walkthrough starting from `sgt-list`/project context. | decompose | **P8** |
| `docs/troubleshooting.md` | Diagnostic procedures: prefer supported Sergeant commands over manual process/tmux/git operations; preserve exact errors/state before recovery. | decompose | **P8** |
| `docs/adr-oc-inject-deletion.md` | ADR recording the August-2026 deletion of the `oc-inject` prototype (decided, executed in PR #190). | reference-only | Historical decision record about an already-deleted mechanism, not current behavior. |
| `docs/audit-2026-07.md` | July-2026 system audit of `bin/`, shared libs, schema, skills, docs across Simplicity/Elegance/Correctness/Performance axes. | reference-only | Point-in-time audit report, research-class evidence. |
| `docs/dead-code-2026-07.md` | July-2026 dead-code audit; status note that most items (D-1…D-7) were actioned in PR #190. | reference-only | Point-in-time audit report. |
| `docs/prd-enforced-phased-dispatch.md` | Draft PRD (status: awaiting explicit human approval) for enforced phased dispatch. | reference-only | Draft PRD — excluded per contract regardless of its non-`prds/` path. |

## `docs/prds/` and `docs/research/`

All five `docs/prds/*.md` and all three `docs/research/*.md` files are
**reference-only** — the N1 contract explicitly excludes this class ("NOT
`prds/` or `research/` which are reference-only").

| Path | What it is | Reason |
|---|---|---|
| `docs/prds/axi-agent-ergonomics.md` | Draft PRD, AXI agent-ergonomics native projections + `sgt-axi` wrapper. | Draft PRD. |
| `docs/prds/claude-background-harness.md` | Draft PRD for the `claude --bg` + `attach` background-harness mechanism. | Draft PRD; also the design doc for the mechanism D2 (deviation register) already supersedes with headless `-p`/`--resume`. |
| `docs/prds/code-improvements.md` | Draft PRD, code-improvements audit-follow-up. | Draft PRD. |
| `docs/prds/enforced-phased-dispatch.md` | Draft PRD, awaiting human approval. | Draft PRD. |
| `docs/prds/tasks-axi-migration.md` | Draft PRD, Tasks AXI migration. | Draft PRD. |
| `docs/research/axi-agent-ergonomics-spike.md` | Research spike, 2026-07-24: hybrid native-projections + `sgt-axi` wrapper decision. | Research spike. |
| `docs/research/claude-background-harness-spike.md` | Research spike, 2026-08-07: measures `claude --bg` + `attach` as the launch mechanism, `claude stop` to end. | Research spike; same D2-superseded mechanism as its paired PRD. |
| `docs/research/tasks-axi-configurable-workflows.md` | Research spike, 2026-07-25: `tasks-axi` TypeScript source inspection for configurable workflows. | Research spike. |

## `schema/` and `templates/`

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `schema/project.yaml.example` | Annotated example of a Sergeant project YAML: name, description, identity resolution order, repo list, etc. | decompose | **P7** |
| `templates/worker-brief.md` | Worker-brief template with `{{TASK_ID}}`/`{{PROJECT}}`/`{{BRIEF}}`-style placeholders, consumed by `sgt-dispatch`. | decompose | **P7** |

## `scripts/hooks/`

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `scripts/hooks/pre-push` | Git pre-push hook: runs the Dockerized drain test suite before every push (skippable via `--no-verify`); installed by `mise run install`. | decompose | **P6** — a real "tests must pass before push" shipping-gate invariant, directly analogous to this repo's own `scripts/gate.sh`. |

## `tests/` — general/infrastructure

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `tests/global-state-isolation-test.sh` | Guard: no test suite may touch real Sergeant state (isolation invariant for the whole suite). | decompose | **P7** |
| `tests/instruction-policy-test.sh` | Tests direct-mode instruction policy assertions ("Never edit a default branch in direct mode", "Open a PR for every direct-mode implementation"). | decompose | **P7** |
| `tests/mise-check-test.sh` | Tests the mise dependency/version-detection check task, including correct platform flags for `tmux`/`lsof`. | decompose | **P7** |
| `tests/mise-install-test.sh` | Tests the mise `install` task: symlinks every `sgt-*`/helper script and hook into PATH, and removes stale symlinks (e.g. deleted `oc-inject`). | decompose | **P7** |
| `tests/no-remote-test.sh` | Guards that the distribution declares no remote-execution contract (local-only invariant). | decompose | **P7** |
| `tests/repo-skills-test.sh` | Tests repo-scoped skill vendoring/discovery under `.agents/skills/`, including that `no-mistakes`' user-invocable guard is stripped for the vendored copy. | decompose | **P7** |
| `tests/run-drain-tests.sh` | Drain-suite runner: orchestrates the two-container (system-Bash + Bash-3.2) Docker test matrix. | helper-evidence | Test-orchestration machinery, not itself an assertion of product behavior. |
| `tests/runtime-bash-test.sh` | Tests drain-state directory resolution/isolation (`SERGEANT_DRAIN_DIR` vs. `SERGEANT_CONFIG/drain` vs. real operator state) across commands. | decompose | **P7** |
| `tests/sergeant-setup-test.sh` | Tests for the `sergeant-setup` Agent Skill (interactive/idempotent bootstrap). | decompose | **P7** |

## `tests/` — response/lease/notification lifecycle

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `tests/sgt-ack-response-test.sh` | Tests `sgt-ack-response`'s response-clearing and retry-convergence behavior. | decompose | **P7** |
| `tests/sgt-callback-test.sh` | Tests `bin/sgt-callback`'s durable, profile-bound callback event protocol. | decompose | **P7** |
| `tests/sgt-lease-convergence-test.sh` | Regression: `sgt-respond`/`sgt-recover` must converge an exact-matching completed turn *before* refusing the lease as prior-supervisor-owned. | decompose | **P7** |
| `tests/sgt-lease-exit-branch-test.sh` | Regression: every worker-exit branch settles its accepted action lease; terminal recycling invokes the same finalizer. | decompose | **P7** |
| `tests/sgt-lease-finalizer-test.sh` | Regression for the single response-lock-protected action-lease finalizer. | decompose | **P7** |
| `tests/sgt-lib-notification-target-test.sh` | Regression: race boundary in `_sgt_notification_target_create`. | decompose | **P7** |
| `tests/sgt-lib-owned-file-test.sh` | Regression for `_sgt_read_owned_file`/`_sgt_read_same_owned_files`. | decompose | **P7** |
| `tests/sgt-notify-test.sh` | Tests `sgt-notify`'s escalation ("Agent Escalation") and completion ("Agent Completion") notification delivery. | decompose | **P7** |
| `tests/sgt-respond-drain-test.sh` | Tests `sgt-respond`'s drain admission: response stored but relaunch held when a drain is active. | decompose | **P7** |
| `tests/sgt-respond-recovery-test.sh` | Regression: `sgt-respond` never leaves a response indefinitely pending. | decompose | **P7** |
| `tests/sgt-respond-test.sh` | Tests `sgt-respond`'s response publication and worker-resume behavior, including publication-failure handling. | decompose | **P7** |
| `tests/sgt-response-lock-release-test.sh` | Regression: `_sgt_response_lock_release` preserves the lock directory var and returns non-zero on `rm` failure so callers can retry. | decompose | **P7** |
| `tests/sgt-review-findings-test.sh` | Tests `sgt-review-findings`' routing of independent-review findings to `td`, including exact-identity matching. | decompose | **P7** |
| `tests/sgt-no-mistakes-finding-test.sh` | Tests `sgt-no-mistakes-finding`'s disposition-application to one finding. | decompose | **P7** |
| `tests/sgt-td-memory-worktree-test.sh` | Regression: `sgt-td-memory` records handoff evidence only from a verified worktree, with every git field resolved from that worktree. | decompose | **P7** |

## `tests/` — dispatch / drain / recovery / cleanup

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `tests/sgt-dispatch-adopt-branch-test.sh` | Tests resuming preserved work on an existing branch during dispatch. | decompose | **P7** |
| `tests/sgt-dispatch-bash32-test.sh` | Regression: every shipped script must parse under Bash 3.2; `sgt-dispatch` must avoid inline complex alternation in `=~`. | decompose | **P7** |
| `tests/sgt-dispatch-brief-test.sh` | Tests `sgt-dispatch`'s worker-brief templating/content. | decompose | **P7** |
| `tests/sgt-dispatch-identity-test.sh` | Tests the identity-key feature for dispatch. | decompose | **P7** |
| `tests/sgt-dispatch-model-tuple-test.sh` | Regression: `sgt-dispatch` pins an explicit provider/model/variant tuple and records it as durable non-secret launch evidence (GH #177). | decompose | **P7** — direct precedent for `ExecutionRecord`-style immutable launch evidence (proposal §16.4). |
| `tests/sgt-dispatch-oc-target-test.sh` | Tests `sgt-dispatch`'s OpenCode (`oc`) target-session resolution for routing coordinator notifications. | decompose | **P7** |
| `tests/sgt-dispatch-td-test.sh` | Tests `sgt-dispatch`'s `td` integration for existing vs. newly created tracked work. | decompose | **P7** |
| `tests/sgt-dispatch-unpushed-guard-test.sh` | Regression: the re-dispatch guard refuses a branch carrying committed work absent from every remote (td-7f3a9a). | decompose | **P7** |
| `tests/sgt-dispatch-worker-test.sh` | Regression: capability validation must abort before any durable side effect (no fleet dir, intent file, `td` task, or worktree) on failure. | decompose | **P7** |
| `tests/sgt-cleanup-cross-filesystem-test.sh` | Regression for `sgt-cleanup`'s atomic publish/rollback across filesystem boundaries (races a directory into the destination between copy and publication). | decompose | **P7** |
| `tests/sgt-cleanup-test.sh` | Tests `sgt-cleanup` across orphaned/present/absent-worktree × open/closed-`td` combinations. | decompose | **P7** |
| `tests/sgt-drain-force-test.sh` | Tests `sgt-drain-force`: confirmed force-stop of drain-eligible workers (td-fa4fed). | decompose | **P7** |
| `tests/sgt-drain-terminate-test.sh` | Regression: a cooperative drain actually stops the worker (td-f57072 / GH #152). | decompose | **P7** |
| `tests/sgt-drain-test.sh` | Tests `sgt-drain`/`sgt-undrain`: persistent drain state and reevaluation. | decompose | **P7** |
| `tests/sgt-graphify-test.sh` | Tests `sgt-graphify`: CLI, failure handling, symlink-alias resolution, atomic publication. | decompose | **P7** |
| `tests/sgt-harness-test.sh` | Regression for the shared harness contract (td-ed15d5 / GH #175): one registry driving gate/probe/launch. | decompose | **P7** |
| `tests/sgt-interrupted-fallback-test.sh` | Regression for interrupted fallback execution reported as never-started despite committed work (#94), across three code paths. | decompose | **P7** |
| `tests/sgt-recover-drain-test.sh` | Tests that stall recovery is refused when drained. | decompose | **P7** |
| `tests/sgt-recover-lease-owner-test.sh` | Regression for two `sgt-recover` defects from GH #160 (td-3de0ac). | decompose | **P7** |
| `tests/sgt-recover-replacement-test.sh` | Regression: `sgt-recover` validates the replacement supervisor before retiring the stalled one, restoring fleet state and refusing on any validation failure. | decompose | **P7** |
| `tests/sgt-recover-test.sh` | Tests `sgt-recover`'s bounded stall recovery. | decompose | **P7** |
| `tests/sgt-wake-test.sh` | Tests `sgt-wake`'s durable wake-condition scheduler. | decompose | **P7** |
| `tests/sgt-watch-background-test.sh` | Tests `sgt-watch --background`: active/terminal/duplicate/failed-start/stale-unit coverage. | decompose | **P7** |
| `tests/sgt-watch-recycle-test.sh` | Regression for terminal-worker recycling (td-b377a0 / GH #152, absorbing td-de2936). | decompose | **P7** |
| `tests/sgt-watch-snapshot-test.sh` | Regression for the bounded read-only fleet snapshot (td-c4f3de / GH #154). | decompose | **P7** |
| `tests/sgt-watch-test.sh` | Tests `sgt-watch`'s fleet monitoring and outcome reporting. | decompose | **P7** |

## `tests/` — validate / no-mistakes launch

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `tests/sgt-validate-test.sh` | Tests `sgt-validate`'s launch of coordinator-owned no-mistakes against varying installed-build capability surfaces (stubbed CLI, `--intent-file` support emulation). | decompose | **P7** |
| `tests/sgt-validation-worker-test.sh` | Tests `sgt-validation-worker`'s no-mistakes invocation, intent-file passing (content must not appear in argv), and exemption of capability probes from run-counting. | decompose | **P7** |

## `tests/` — worker/pane lifecycle (dense with tmux-pane mechanism)

| Path | What it is | Disposition | Partition / Reason |
|---|---|---|---|
| `tests/sgt-worker-test.sh` | Core test suite for `sgt-interactive-worker`'s pane-launch and lifecycle behavior. | decompose | **P7** — flagged: exercises the same pane-ownership mechanism as `bin/sgt-interactive-worker`; primary obsolete-mechanism-stress-test input, kept in decompose so any surviving checkpoint intent (readiness, escalation) is still extracted. |
| `tests/sgt-worker-drain-test.sh` | Tests the cooperative drain checkpoint inside `sgt-interactive-worker` (td-6c9911): a drained-status worker exits cleanly with `td` handoff. | decompose | **P7** — same flag as above. |
| `tests/sgt-drain-worker-test.sh` | Behavioral tests for cooperative drain in `sgt-interactive-worker`. | decompose | **P7** — same flag as above. |
| `tests/sgt-worker-handshake-test.sh` | Per-harness end-to-end notification handshake regression (td-db6323 / GH #175): a harness reaching its pane, its notification delivered. | decompose | **P7** — same flag as above. |
| `tests/sgt-worker-readiness-test.sh` | Regression: the interactive worker's readiness wait is bounded and reported (td-ed15d5 / GH #175 AC4); a harness that never renders must be caught, not hang. | decompose | **P7** — the "bounded and reported" *shape* is portable (echoes CLAUDE.md's fail-closed invariant) even though the pane-readiness *mechanism* is not; same flag as above. |
| `tests/sgt-worker-model-tuple-test.sh` | Regression: the interactive worker passes the pinned provider/model tuple to the harness and records launch evidence matching it. | decompose | **P7** — durable launch-evidence precedent, same relevance as the dispatch model-tuple test. |
| `tests/sgt-dispatch-coordinator-pane-test.sh` | Regression: `sgt-dispatch` can bind a coordinator pane without already living in one, refusing every forged/stale/unreachable identity. | decompose | **P7** — same flag as above (coordinator-pane binding is pane-specific mechanism; the "refuse forged/stale/unreachable identity" invariant is portable). |

## `tests/` — Claude background-harness mechanism (obsolete-candidate)

These three tests exercise exactly the `claude --bg` + `attach` background-harness
mechanism that deviation register **D2** (`GAUNTLET.md`) already rules superseded:
"Daemon has no TTY/pane... headless turn sequence — `claude -p --output-format
stream-json`... Confirmed at M4." Nothing here adds extraction value beyond
what D2 already records, so these are the inventory's only **obsolete-candidate**
rows.

| Path | What it is | Disposition | Reason |
|---|---|---|---|
| `tests/sgt-claude-real-contract-test.sh` | Real-Claude contract test against a live `claude` binary for the Claude Background Harness PRD (CH-5) — the `--bg`/`attach` mechanism. | obsolete-candidate | D2: headless `-p`/`--resume` measured and shipped instead; no TTY/pane in the daemon. |
| `tests/sgt-claude-stop-bg-session-test.sh` | Regression for `_sgt_claude_stop_bg_session` (`bin/_sgt-drain.sh`), the CH-PRD's backstop for stopping a backgrounded/attached Claude session. | obsolete-candidate | Same D2 supersession; the function under test only exists to serve the superseded mechanism. |
| `tests/sgt-claude-worker-test.sh` | Fake-CLI regression suite for the full `claude --bg` background-harness lifecycle (CH-4). | obsolete-candidate | Same D2 supersession. |

---

# Partition Table

Every **decompose** row above, grouped into the eight N1 partitions. Counts
exclude helper-evidence/obsolete-candidate/reference-only rows. Verified by
script against the tables above (each of the 179 rows parsed exactly once;
139 decompose rows each carry exactly one `P1`–`P8` tag).

## P1 — root-instructions (5)

- `AGENTS.md`
- `README.md`
- `docs/what-is-sergeant.md`
- `docs/skills.md`
- `docs/repo-scoped-skills.md`

## P2 — dev-skills-a (7)

- `.agents/skills/code-review/SKILL.md`
- `.agents/skills/diagnosing-bugs/SKILL.md`
- `.agents/skills/implement/SKILL.md`
- `.agents/skills/no-mistakes/SKILL.md`
- `.agents/skills/tdd/SKILL.md`
- `.agents/skills/tdd/mocking.md`
- `.agents/skills/tdd/tests.md`

(`.agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh` is filed under
this same skill directory but is disposition **helper-evidence**, not
decompose — it is a deterministic HITL-loop script template, not itself a
procedural outcome — so it is excluded from this partition's count.)

## P3 — dev-skills-b (10)

- `.agents/skills/grill-with-docs/SKILL.md`
- `.agents/skills/grilling/SKILL.md`
- `.agents/skills/prototype/SKILL.md`
- `.agents/skills/prototype/LOGIC.md`
- `.agents/skills/prototype/UI.md`
- `.agents/skills/research/SKILL.md`
- `.agents/skills/resolving-merge-conflicts/SKILL.md`
- `.agents/skills/triage/SKILL.md`
- `.agents/skills/triage/AGENT-BRIEF.md`
- `.agents/skills/triage/OUT-OF-SCOPE.md`

## P4 — design-skills (9)

- `.agents/skills/codebase-design/SKILL.md`
- `.agents/skills/codebase-design/DEEPENING.md`
- `.agents/skills/codebase-design/DESIGN-IT-TWICE.md`
- `.agents/skills/domain-modeling/SKILL.md`
- `.agents/skills/domain-modeling/ADR-FORMAT.md`
- `.agents/skills/domain-modeling/CONTEXT-FORMAT.md`
- `.agents/skills/to-spec/SKILL.md`
- `.agents/skills/to-tickets/SKILL.md`
- `.agents/skills/wayfinder/SKILL.md`

## P5 — ops-skills (6)

- `.agents/skills/sergeant-setup/SKILL.md`
- `skills/cross-repo-work/SKILL.md`
- `skills/dispatch/SKILL.md`
- `skills/load-project/SKILL.md`
- `skills/sergeant-help/SKILL.md`
- `skills/wiki/SKILL.md`

## P6 — bin-machinery (36)

- `mise.toml`
- `opencode.json`
- `scripts/hooks/pre-push`
- `bin/_sgt-intent.sh`
- `bin/_sgt-drain.sh`
- `bin/_sgt-harness.sh`
- `bin/_sgt-response-lock.sh`
- `bin/sgt-ack-response`
- `bin/sgt-callback`
- `bin/sgt-cleanup`
- `bin/sgt-context`
- `bin/sgt-dag-dispatch-hook`
- `bin/sgt-dag-run`
- `bin/sgt-dispatch`
- `bin/sgt-drain`
- `bin/sgt-drain-force`
- `bin/sgt-graphify`
- `bin/sgt-interactive-worker`
- `bin/sgt-list`
- `bin/sgt-no-mistakes-finding`
- `bin/sgt-notify`
- `bin/sgt-recover`
- `bin/sgt-respond`
- `bin/sgt-review-findings`
- `bin/sgt-status`
- `bin/sgt-sync`
- `bin/sgt-td-create`
- `bin/sgt-td-list`
- `bin/sgt-td-memory`
- `bin/sgt-treehouse-init`
- `bin/sgt-undrain`
- `bin/sgt-validate`
- `bin/sgt-validation-worker`
- `bin/sgt-wake`
- `bin/sgt-watch`
- `bin/wiki-daily-digest`

## P7 — tests-schema (60)

- `schema/project.yaml.example`
- `templates/worker-brief.md`
- `tests/global-state-isolation-test.sh`
- `tests/instruction-policy-test.sh`
- `tests/mise-check-test.sh`
- `tests/mise-install-test.sh`
- `tests/no-remote-test.sh`
- `tests/repo-skills-test.sh`
- `tests/runtime-bash-test.sh`
- `tests/sergeant-setup-test.sh`
- `tests/sgt-ack-response-test.sh`
- `tests/sgt-callback-test.sh`
- `tests/sgt-lease-convergence-test.sh`
- `tests/sgt-lease-exit-branch-test.sh`
- `tests/sgt-lease-finalizer-test.sh`
- `tests/sgt-lib-notification-target-test.sh`
- `tests/sgt-lib-owned-file-test.sh`
- `tests/sgt-notify-test.sh`
- `tests/sgt-respond-drain-test.sh`
- `tests/sgt-respond-recovery-test.sh`
- `tests/sgt-respond-test.sh`
- `tests/sgt-response-lock-release-test.sh`
- `tests/sgt-review-findings-test.sh`
- `tests/sgt-no-mistakes-finding-test.sh`
- `tests/sgt-td-memory-worktree-test.sh`
- `tests/sgt-dispatch-adopt-branch-test.sh`
- `tests/sgt-dispatch-bash32-test.sh`
- `tests/sgt-dispatch-brief-test.sh`
- `tests/sgt-dispatch-identity-test.sh`
- `tests/sgt-dispatch-model-tuple-test.sh`
- `tests/sgt-dispatch-oc-target-test.sh`
- `tests/sgt-dispatch-td-test.sh`
- `tests/sgt-dispatch-unpushed-guard-test.sh`
- `tests/sgt-dispatch-worker-test.sh`
- `tests/sgt-cleanup-cross-filesystem-test.sh`
- `tests/sgt-cleanup-test.sh`
- `tests/sgt-drain-force-test.sh`
- `tests/sgt-drain-terminate-test.sh`
- `tests/sgt-drain-test.sh`
- `tests/sgt-graphify-test.sh`
- `tests/sgt-harness-test.sh`
- `tests/sgt-interrupted-fallback-test.sh`
- `tests/sgt-recover-drain-test.sh`
- `tests/sgt-recover-lease-owner-test.sh`
- `tests/sgt-recover-replacement-test.sh`
- `tests/sgt-recover-test.sh`
- `tests/sgt-wake-test.sh`
- `tests/sgt-watch-background-test.sh`
- `tests/sgt-watch-recycle-test.sh`
- `tests/sgt-watch-snapshot-test.sh`
- `tests/sgt-watch-test.sh`
- `tests/sgt-validate-test.sh`
- `tests/sgt-validation-worker-test.sh`
- `tests/sgt-worker-test.sh`
- `tests/sgt-worker-drain-test.sh`
- `tests/sgt-drain-worker-test.sh`
- `tests/sgt-worker-handshake-test.sh`
- `tests/sgt-worker-readiness-test.sh`
- `tests/sgt-worker-model-tuple-test.sh`
- `tests/sgt-dispatch-coordinator-pane-test.sh`

(`tests/run-drain-tests.sh` is disposition helper-evidence — a suite runner,
not itself a behavioral assertion — so it is excluded from this partition.
The three Claude-background-harness tests are obsolete-candidate, per the
table above, and are likewise excluded.)

## P8 — support-docs (6)

- `docs/README.md`
- `docs/callbacks.md`
- `docs/schema.md`
- `docs/getting-started.md`
- `docs/using-sergeant.md`
- `docs/troubleshooting.md`

---

# Counts

## By disposition

| Disposition | Count |
|---|---|
| decompose | 139 |
| helper-evidence | 20 |
| obsolete-candidate | 3 |
| reference-only | 17 |
| **Total files** | **179** |

**helper-evidence (20):** 14 `agents/openai.yaml` cross-harness metadata files
+ `bin/_sgt-bash-version.sh` + `bin/_sgt-lib.sh` + `bin/_sgt-review-axes.sh`
+ `Dockerfile.test` + `tests/run-drain-tests.sh` +
`.agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh`.

**reference-only (17):** `LICENSE`, `.gitignore` (2 root); `.agents/skills/PROVENANCE.md`,
`.agents/skills/THIRD_PARTY_NOTICES.md` (2 provenance); `docs/adr-oc-inject-deletion.md`,
`docs/audit-2026-07.md`, `docs/dead-code-2026-07.md`, `docs/prd-enforced-phased-dispatch.md`
(4 `docs/` root historical/draft-PRD); `docs/prds/*.md` (5); `docs/research/*.md` (3);
`bin/__pycache__/sgt-callbackcpython-312.pyc` (1 binary). 2+2+4+5+3+1 = 17.

## By partition (decompose only)

| Partition | Count |
|---|---|
| P1 root-instructions | 5 |
| P2 dev-skills-a | 7 |
| P3 dev-skills-b | 10 |
| P4 design-skills | 9 |
| P5 ops-skills | 6 |
| P6 bin-machinery | 36 |
| P7 tests-schema | 60 |
| P8 support-docs | 6 |
| **Total decompose** | **139** |

All four disposition counts and all eight partition counts were verified by
parsing every table row in this document by script; 139 (decompose) + 20
(helper-evidence) + 3 (obsolete-candidate) + 17 (reference-only) = 179, and
the eight partition counts sum to 139, matching the decompose total exactly.
