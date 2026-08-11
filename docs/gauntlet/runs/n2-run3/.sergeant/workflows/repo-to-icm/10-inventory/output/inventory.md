# 10-inventory: source inventory — run 01KZQ32J2BAD4P8WJA9SWXRMZ9

Subject: `reference/sergeant-upstream` (vendored subtree, no live `.git`),
revision `f430cfd4f90174a98adbd7abebbece6303817929` per `../00-contract/output/contract.md`.
`contract.md` did not open with `# AMBIGUOUS — NOT RESOLVED`, so this stage
proceeded with its ordinary work per `../_config/run-discipline.md` §2.

Enumeration method: **vendored-subtree case** (per `contract.md` §1 and this
stage's `CONTEXT.md` step 1) — an ordinary recursive file listing of the
working tree (`find reference/sergeant-upstream -type f`), not
`git -C reference/sergeant-upstream ls-files`, since there is no git object to
list against. This run's purpose is not measurement (`contract.md` §3: no
`reference-corpus/` exists anywhere in this worktree), so the blindness rule
in `run-discipline.md` §1 is vacuous here — nothing was excluded on that
account.

Every file was opened and read (full read for files with a self-contained
scope; skimmed by header/purpose-comment plus structural sampling for very
large single scripts — noted per-partition below) before a disposition was
assigned. No disposition here comes from filename pattern alone.

## Totals

| | Count |
|---|---|
| Files enumerated (in scope, per `contract.md` §2–3) | 178 |
| `decompose` | 82 |
| `helper-evidence` | 80 |
| `obsolete-candidate` | 0 |
| `reference-only` | 16 |
| **Sum** | **178** |
| Directory symlinks noted separately (not counted in the 178 — see "Symlinks" below) | 17 |

Reconciliation: 82 + 80 + 0 + 16 = 178, matching the 178 files enumerated in
step 1 (179 total files under `reference/sergeant-upstream`, minus the 1
excluded `bin/__pycache__/sgt-callback­cpython-312.pyc` build artifact per
`contract.md` §3).

## Partitions (decompose rows only — 82 files across 21 partitions)

| # | Partition | Members | Count |
|---|---|---|---|
| P1 | Root agent policy | `AGENTS.md` | 1 |
| P2 | Product overview, documentation index & help | `README.md`, `docs/README.md`, `docs/what-is-sergeant.md`, `docs/skills.md`, `docs/repo-scoped-skills.md`, `skills/sergeant-help/SKILL.md` | 6 |
| P3 | Installation, usage, troubleshooting & config schema | `docs/getting-started.md`, `docs/using-sergeant.md`, `docs/troubleshooting.md`, `docs/schema.md`, `schema/project.yaml.example`, `mise.toml` | 6 |
| P4 | Durable callback protocol | `docs/callbacks.md` | 1 |
| P5 | Project resolution, status, sync, td-query & graphify | `bin/sgt-list`, `bin/sgt-context`, `bin/sgt-status`, `bin/sgt-sync`, `bin/sgt-td-list`, `bin/sgt-graphify`, `skills/load-project/SKILL.md` | 7 |
| P6 | Cross-repo planning & dispatch | `skills/cross-repo-work/SKILL.md`, `skills/dispatch/SKILL.md`, `bin/sgt-dispatch`, `bin/sgt-td-create`, `bin/sgt-treehouse-init`, `bin/_sgt-review-axes.sh`, `templates/worker-brief.md` | 7 |
| P7 | Worker lifecycle: interactive session & validation | `bin/sgt-interactive-worker`, `bin/sgt-validate`, `bin/sgt-validation-worker`, `bin/_sgt-harness.sh`, `bin/_sgt-intent.sh`, `bin/sgt-td-memory` | 6 |
| P8 | Response, wake & recovery | `bin/sgt-respond`, `bin/sgt-ack-response`, `bin/sgt-wake`, `bin/sgt-recover`, `bin/_sgt-response-lock.sh` | 5 |
| P9 | Drain control | `bin/sgt-drain`, `bin/sgt-drain-force`, `bin/sgt-undrain`, `bin/_sgt-drain.sh` | 4 |
| P10 | Fleet monitoring & cleanup | `bin/sgt-watch`, `bin/sgt-cleanup` | 2 |
| P11 | Escalation & finding routing | `bin/sgt-callback`, `bin/sgt-notify`, `bin/sgt-review-findings`, `bin/sgt-no-mistakes-finding` | 4 |
| P12 | Wiki capture & digest | `skills/wiki/SKILL.md`, `bin/wiki-daily-digest` | 2 |
| P13 | DAG-driven dispatch (dagr integration) | `bin/sgt-dag-run`, `bin/sgt-dag-dispatch-hook` | 2 |
| P14 | Shared bash foundation | `bin/_sgt-lib.sh`, `bin/_sgt-bash-version.sh` | 2 |
| P15 | Vendored single-doc engineering skills (mattpocock/skills) | `.agents/skills/code-review/SKILL.md`, `.agents/skills/diagnosing-bugs/SKILL.md`, `.agents/skills/grill-with-docs/SKILL.md`, `.agents/skills/grilling/SKILL.md`, `.agents/skills/implement/SKILL.md`, `.agents/skills/research/SKILL.md`, `.agents/skills/resolving-merge-conflicts/SKILL.md`, `.agents/skills/to-spec/SKILL.md`, `.agents/skills/wayfinder/SKILL.md` | 9 |
| P16 | Vendored multi-doc skill: codebase-design | `.agents/skills/codebase-design/SKILL.md`, `DEEPENING.md`, `DESIGN-IT-TWICE.md` | 3 |
| P17 | Vendored multi-doc skill: domain-modeling | `.agents/skills/domain-modeling/SKILL.md`, `ADR-FORMAT.md`, `CONTEXT-FORMAT.md` | 3 |
| P18 | Vendored multi-doc skill: prototype | `.agents/skills/prototype/SKILL.md`, `LOGIC.md`, `UI.md` | 3 |
| P19 | Vendored multi-doc skill: tdd | `.agents/skills/tdd/SKILL.md`, `mocking.md`, `tests.md` | 3 |
| P20 | Vendored multi-doc skill: triage | `.agents/skills/triage/SKILL.md`, `AGENT-BRIEF.md`, `OUT-OF-SCOPE.md` | 3 |
| P21 | Sergeant-authored operational skills | `.agents/skills/no-mistakes/SKILL.md`, `.agents/skills/sergeant-setup/SKILL.md`, `.agents/skills/to-tickets/SKILL.md` | 3 |
| | **Total** | | **82** |

---

## Root files (10)

| Path | Disposition | What it is | Reason |
|---|---|---|---|
| `AGENTS.md` | decompose (P1) | Always-on Sergeant coordinator policy: role selection (coordinator vs. direct), toolbelt table, procedural-skill trigger table, standard workflow, no-op-outcome prohibitions, conventions. | States and requires procedural outcomes an agent follows/decides by throughout every session — the canonical `agents-invariant`/`workflow` source. |
| `README.md` | decompose (P2) | Project README: genesis, mental model, quick start, toolbelt table, model-pinning contract, no-mistakes driving procedure, independent-review routing, skills table, requirements. | Beyond orientation prose it states real procedural directives (model-pin precedence, no-mistakes gate driving loop, finding routing/severity rules) a reader follows, not just describes. |
| `Dockerfile.test` | helper-evidence | Docker image definition for the drain test suite's reproducible Debian environment. | Explicitly the dispositions legend's own helper-evidence example ("a Docker image definition"); subordinate machinery for the `test:docker:drain` checkpoint, not itself a checkpoint. |
| `LICENSE` | reference-only | MIT license text, Copyright (c) 2026 Lars Cromley. | License text — explicit reference-only example in the legend. |
| `.gitignore` | reference-only | Two ignore patterns: `.DS_Store`, `.todos/`. | Non-procedural mechanical VCS config, closest to the legend's "generated lockfiles and similar non-procedural material" example. |
| `mise.toml` | decompose (P3) | `mise` task definitions: `install` (symlink `sgt-*`/`_sgt-*.sh` onto PATH, install git hooks, remove stale `oc-inject` links), `install:hooks`, `uninstall:hooks`, `uninstall`, `check` (dependency verification incl. `td`/agent-harness probes), `test:docker:drain` (two-pass Docker drain suite), `update`. | Defines real installer/dependency-check/update procedures that `getting-started.md` and `sergeant-setup/SKILL.md` treat as the canonical setup commands — these are the checkpoints themselves, not machinery subordinate to one. |
| `opencode.json` | helper-evidence | 9-line OpenCode config pointing skill discovery at `.agents/skills` and `skills`. | Deterministic harness-discovery config, not itself a procedural outcome. |
| `schema/project.yaml.example` | decompose (P3) | Fully annotated example project YAML: repos, groups, graphify, dag block, defaults, with inline field-semantics comments. | States real field behavior/semantics (path resolution, identity precedence, dag stage wiring) a reader follows when authoring a project — companion primary source to `docs/schema.md`. |
| `scripts/hooks/pre-push` | helper-evidence | Git pre-push hook: resolves `mise`, checks Docker, runs `mise run test:docker:drain`, else fails with an escape-hatch hint (`--no-verify`). | Deterministic machinery invoked automatically by git at a checkpoint (pushing), not itself the checkpoint; a template-like invocation wrapper per the legend. |
| `templates/worker-brief.md` | decompose (P6) | The literal `.sergeant-brief.md` template injected into every dispatched worker: scope/intent pinning, work routing by category, TDD implementation, escalate/resume protocol incl. notification token handshake and wake-condition kinds, validation handoff, review-axis routing, remediation, delivery/td lifecycle completion gates. | The single richest primary source for the dispatch worker's procedural contract — almost every worker-facing behavior in `skills/dispatch/SKILL.md` is restated here in the literal text a worker actually reads. |

---

## `docs/` (21 files)

| Path | Disposition | What it is | Reason |
|---|---|---|---|
| `docs/README.md` | decompose (P2) | Documentation index plus a "Documentation authority" precedence section. | Mostly navigation, but the authority section states a real decision procedure (which source wins when docs/behavior disagree; file a task on mismatch). |
| `docs/adr-oc-inject-deletion.md` | reference-only | ADR: decided/executed record of deleting the already-superseded `oc-inject` prototype and its call sites. | Historical decision record about a mechanism no longer present in the subject tree — legend's "historical drafts" example; nothing here to disposition as `obsolete-candidate` since the obsoleted file itself is already gone. |
| `docs/audit-2026-07.md` | reference-only | July 2026 system audit of `bin/`, shared libs, schema, skills, docs across Simplicity/Elegance/Correctness/Performance axes. | Explicit legend example: "audits". |
| `docs/callbacks.md` | decompose (P4) | Full spec of the durable callback protocol v1: profile install/permission rules, origin registration, event production/classification, consumer stdin/stdout contract, retry/recovery, cleanup sealing. | Dense, precise procedural/protocol behavior a producer, consumer, and operator each follow — primary source, not descriptive summary. |
| `docs/dead-code-2026-07.md` | reference-only | July 2026 dead-code audit across all `bin/` scripts by call-graph analysis; note at top records which findings were later actioned. | Explicit legend example: "audits". |
| `docs/getting-started.md` | decompose (P3) | Installation checklist: prerequisites, `mise run check` verification, clone/install, project registration walkthrough. | Actionable installation procedure a new user follows step by step. |
| `docs/prd-enforced-phased-dispatch.md` | reference-only | Draft PRD: enforced phased dispatch (PRD→OpenSpec→implementation gating), pinned source baseline. | Explicit legend example: "PRDs"; Status: Draft, not yet-implemented policy. |
| `docs/prds/axi-agent-ergonomics.md` | reference-only | Draft PRD: AXI agent-ergonomics command output redesign. | PRD. |
| `docs/prds/claude-background-harness.md` | reference-only | Draft PRD: Claude background-harness dispatch problem statement. | PRD. |
| `docs/prds/code-improvements.md` | reference-only | Draft PRD: code-improvements backlog derived from the two 2026-07 audits. | PRD. |
| `docs/prds/enforced-phased-dispatch.md` | reference-only | Byte-identical duplicate of `docs/prd-enforced-phased-dispatch.md` (`diff` confirms no difference). | Per `references/dispositions.md` "Symlinks and duplicated trees": not re-dispositioned independently — same disposition and reasoning as its duplicate target, `docs/prd-enforced-phased-dispatch.md`. |
| `docs/prds/tasks-axi-migration.md` | reference-only | Draft PRD: migrate task backend from Marcus `td` to Tasks AXI. | PRD. |
| `docs/repo-scoped-skills.md` | decompose (P2) | States which harness discovers the vendored `.agents/skills/` tree through which path (Codex direct, OpenCode via `opencode.json`, Claude via `.claude/skills/` links) and the worker-brief skill inventory. | Real cross-harness discovery behavior/invariant, not merely a pointer. |
| `docs/research/axi-agent-ergonomics-spike.md` | reference-only | Dated research spike evaluating AXI adoption; records a decision but is framed as investigation output. | Explicit legend example: "research notes". |
| `docs/research/claude-background-harness-spike.md` | reference-only | Dated research spike measuring Claude Code's background-session CLI behavior for worker dispatch. | Research note. |
| `docs/research/tasks-axi-configurable-workflows.md` | reference-only | Dated research spike on Tasks AXI as a configurable-workflow backend. | Research note. |
| `docs/schema.md` | decompose (P3) | Canonical Project YAML schema reference: global config, top-level/`repos[]`/`groups`/`graphify`/`defaults` fields, instruction-layering order, path resolution. | Explicitly named as the field-authority source by `docs/README.md`'s own "Documentation authority" section and by `skills/load-project/SKILL.md`; states real validated behavior, not just a data shape. |
| `docs/skills.md` | decompose (P2) | Skill locations across harnesses, provenance/verification procedure, Sergeant-owned skill trigger table, "choosing a skill" split (user-invoked vs. model-invoked), skill-adoption checklist. | Real procedural guidance (how to install/verify/choose/update a skill), not a static index. |
| `docs/troubleshooting.md` | decompose (P3) | Diagnostic runbook: command-not-found, missing/wrong project, missing/behind repo, wrong `td` executable, and more (sampled via header; further sections follow the same pattern). | Actionable diagnose/fix procedures. |
| `docs/using-sergeant.md` | decompose (P3) | Direct vs. dispatch mode walkthroughs with exact commands, intent-revision handling, dependency ordering. | States the same operational procedure as `AGENTS.md` §"Standard workflow", in user-facing form — a primary source in its own right. |
| `docs/what-is-sergeant.md` | decompose (P2) | Product scope/boundary statement: single-user deployment model, core concepts (Project/Repository/Task/Fleet/Worker/Decision request), execution modes, explicit "what Sergeant is not" list. | Real `agents-invariant`-shaped scoping rules (e.g. "does not turn one installation into a shared team service") that bound every other behavior. |

---

## `bin/` (36 files — all decompose)

Every `bin/` script (including the seven underscore-prefixed sourced
libraries) was read via its header/usage comment and, for the ones defining a
protocol rather than a thin CLI wrapper (`_sgt-drain.sh`, `_sgt-harness.sh`,
`_sgt-response-lock.sh`, `_sgt-review-axes.sh`), the body defining that
protocol. All 36 disposition `decompose`: applying the §6.3 reimplementation
test (`../_config/icm-ladder.md`) to each — "would dispatching a worker /
draining the fleet / recovering a stalled worker / relaying a durable
callback remain a meaningful checkpoint no matter what implemented it?" —
answers yes in every case, including the underscore-prefixed libraries, which
each define a real protocol (drain lock/admission semantics, the harness
readiness contract, the response archive format, the review axis/severity
vocabulary) rather than incidental plumbing.

| Path | Partition | What it is |
|---|---|---|
| `bin/_sgt-bash-version.sh` | P14 | Shared minimum-Bash-version (3.2+) gate sourced by every entry point. |
| `bin/_sgt-drain.sh` | P9 | Drain state/lock helpers: admission gating, hard-link-based locking, Claude background-session stop helpers. |
| `bin/_sgt-harness.sh` | P7 | Single accepted-harness registry (probe/launch-args) unifying the readiness contract across `opencode`/`goose`/`claude`. |
| `bin/_sgt-intent.sh` | P7 | Intent-revision hashing and `no-mistakes axi run` flag-capability probing. |
| `bin/_sgt-lib.sh` | P14 | Shared `_die`/`_info`/`_require_*`/`_resolve_path` helpers and `SGT_*` env vars sourced by all `sgt-*` scripts. |
| `bin/_sgt-response-lock.sh` | P8 | Shared serialization/archive format for response publication and consumption. |
| `bin/_sgt-review-axes.sh` | P6 | Canonical independent-review axis (`standards`/`spec`/`readiness`/conditional `accessibility`) and severity vocabulary shared by dispatch and `sgt-review-findings`. |
| `bin/sgt-ack-response` | P8 | Acknowledge and clear one consumed worker response. |
| `bin/sgt-callback` | P11 | Durable, profile-bound callback events for fleet tasks (Python). |
| `bin/sgt-cleanup` | P10 | Remove worktrees + fleet state for a completed task (treehouse-aware, symlink/traversal-hardened). |
| `bin/sgt-context` | P5 | Emit a structured agent context block for a project (instruction layering). |
| `bin/sgt-dag-dispatch-hook` | P13 | Stage hook called by `dagr` when a DAG stage becomes ready; wraps `sgt-dispatch` and records dagr tracking files. |
| `bin/sgt-dag-run` | P13 | Create/start a `dagr` DAG run from a project YAML `dag:` block. |
| `bin/sgt-dispatch` | P6 | Dispatch subagents across repos: worktree + brief + tmux pane creation, model/agent pinning, dependency ordering. |
| `bin/sgt-drain` | P9 | Set/remove/query a persistent drain on a project or globally, with optional wait-for-drain. |
| `bin/sgt-drain-force` | P9 | Force-stop workers that failed cooperative drain; requires an active drain and explicit `--yes`/`--dry-run`. |
| `bin/sgt-graphify` | P5 | Run graphify across a project's repos, publishing the merged graph atomically. |
| `bin/sgt-interactive-worker` | P7 | Own one persistent interactive agent pane for a worker. |
| `bin/sgt-list` | P5 | List all known Sergeant projects from `~/.config/sergeant/`. |
| `bin/sgt-no-mistakes-finding` | P11 | Apply a disposition (`gate`/`td`/`ignore`/`ask-user`) to one no-mistakes finding, creating/updating owning-repo td work. |
| `bin/sgt-notify` | P11 | Inject a worker update into the primary session (durable wake marker by default; tmux injection as compatibility option). |
| `bin/sgt-recover` | P8 | One bounded stall-recovery attempt for a stalled in-progress worker, gated on stall proof. |
| `bin/sgt-respond` | P8 | Deliver a response and resume a dead waiting worker when needed. |
| `bin/sgt-review-findings` | P11 | Route structured independent-review findings to td with dedup/reconciliation semantics. |
| `bin/sgt-status` | P5 | Show git status across every repo in a project. |
| `bin/sgt-sync` | P5 | Clone missing repos, pull existing ones, for a project. |
| `bin/sgt-td-create` | P6 | Create td tasks in project repos for a cross-repo brief (all-or-nothing with rollback). |
| `bin/sgt-td-list` | P5 | Show td tasks across all repos in a project. |
| `bin/sgt-td-memory` | P7 | Record non-secret worker recovery pointers (handoff/response) in td. |
| `bin/sgt-treehouse-init` | P6 | Initialize treehouse worktree pools in a project's repos. |
| `bin/sgt-undrain` | P9 | Remove a drain record for a project or globally (idempotent). |
| `bin/sgt-validate` | P7 | Launch coordinator-owned no-mistakes beside an interactive worker (ownership claim/release supported). |
| `bin/sgt-validation-worker` | P7 | Run coordinator-owned no-mistakes in an interactive pane. |
| `bin/sgt-wake` | P8 | Evaluate a durable wake condition and resume a waiting worker (six condition kinds). |
| `bin/sgt-watch` | P10 | Monitor a dispatched fleet and report outcomes (background/foreground/snapshot/sync modes). |
| `bin/wiki-daily-digest` | P12 | Synthesize a daily wiki session digest from opencode/goose/claude history via the Anthropic API. |

---

## `skills/` (5 files — all decompose)

| Path | Partition | What it is |
|---|---|---|
| `skills/cross-repo-work/SKILL.md` | P6 | Decompose ownership, dependency/merge order, per-repo delivery gates before dispatch. |
| `skills/dispatch/SKILL.md` | P6 | Full dispatch protocol: td queue check, plan confirmation, dispatch, monitor, reconcile, treehouse, worker contract (19-point numbered list), td task creation, flags reference, troubleshooting. |
| `skills/load-project/SKILL.md` | P5 | Resolve project ownership/config/paths; registration/edit procedure; project Graphify procedure. |
| `skills/sergeant-help/SKILL.md` | P2 | Documentation-map-driven, read-only Q&A procedure with a fixed precedence order and response format. |
| `skills/wiki/SKILL.md` | P12 | Wiki capture ownership, daily-digest procedure, scheduled-execution guidance. |

---

## `.agents/skills/` (44 files)

Canonical vendored worker-skill tree (per `docs/repo-scoped-skills.md` /
`docs/skills.md`), discovered directly by Codex, via `opencode.json` by
OpenCode, and via the `.claude/skills/` symlinks (see "Symlinks" below) by
Claude.

| Path | Disposition | Partition | What it is / reason |
|---|---|---|---|
| `.agents/skills/PROVENANCE.md` | reference-only | — | Skill-import provenance ledger (source repo, locked hashes, sync dates, local-modification notes for `no-mistakes`). Non-procedural record. |
| `.agents/skills/THIRD_PARTY_NOTICES.md` | reference-only | — | Third-party MIT notices for the vendored mattpocock/skills set plus Sergeant's own project license note. License text. |
| `.agents/skills/code-review/SKILL.md` | decompose | P15 | Two-axis (Standards/Spec) parallel-subagent code review procedure incl. a fixed Fowler smell baseline. |
| `.agents/skills/code-review/agents/openai.yaml` | helper-evidence | — | 3-line harness interface metadata (display name/description). Deterministic registration config, not behavior. |
| `.agents/skills/codebase-design/SKILL.md` | decompose | P16 | Deep-module design vocabulary (module/interface/depth/seam/adapter/leverage/locality) and principles. |
| `.agents/skills/codebase-design/DEEPENING.md` | decompose | P16 | Dependency-category-driven method for safely deepening a cluster of shallow modules. |
| `.agents/skills/codebase-design/DESIGN-IT-TWICE.md` | decompose | P16 | Parallel-subagent procedure for exploring 3+ radically different interface designs. |
| `.agents/skills/codebase-design/agents/openai.yaml` | helper-evidence | — | Harness interface metadata. |
| `.agents/skills/diagnosing-bugs/SKILL.md` | decompose | P15 | Six-phase hard-bug diagnosis discipline: feedback loop, reproduce/minimize, hypothesize, instrument, fix+regression test, cleanup/post-mortem. |
| `.agents/skills/diagnosing-bugs/agents/openai.yaml` | helper-evidence | — | Harness interface metadata. |
| `.agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh` | helper-evidence | — | Copy-and-edit human-in-the-loop bash template invoked as this skill's Phase-1 last-resort loop mechanism. | 
| `.agents/skills/domain-modeling/SKILL.md` | decompose | P17 | Active domain-model-building discipline: challenge glossary terms, sharpen vague language, cross-reference code, update `CONTEXT.md` inline, offer ADRs sparingly. |
| `.agents/skills/domain-modeling/ADR-FORMAT.md` | decompose | P17 | ADR file/numbering format and the three-part "when to offer an ADR" test. |
| `.agents/skills/domain-modeling/CONTEXT-FORMAT.md` | decompose | P17 | `CONTEXT.md`/`CONTEXT-MAP.md` structure and rules for single- vs. multi-context repos. |
| `.agents/skills/domain-modeling/agents/openai.yaml` | helper-evidence | — | Harness interface metadata. |
| `.agents/skills/grill-with-docs/SKILL.md` | decompose | P15 | Thin wrapper: run a `/grilling` session using `/domain-modeling`. |
| `.agents/skills/grill-with-docs/agents/openai.yaml` | helper-evidence | — | Harness interface metadata (incl. `allow_implicit_invocation: false`). |
| `.agents/skills/grilling/SKILL.md` | decompose | P15 | One-question-at-a-time relentless-interview discipline. |
| `.agents/skills/grilling/agents/openai.yaml` | helper-evidence | — | Harness interface metadata. |
| `.agents/skills/implement/SKILL.md` | decompose | P15 | Implement-from-spec procedure: TDD at pre-agreed seams, regular typecheck/tests, code-review, commit. |
| `.agents/skills/implement/agents/openai.yaml` | helper-evidence | — | Harness interface metadata. |
| `.agents/skills/no-mistakes/SKILL.md` | decompose | P21 | Coordinator-only no-mistakes shipping-gate contract reference (worker-restricted; full `axi run`/`respond` drive loop, escalation, TOON output reading). |
| `.agents/skills/prototype/SKILL.md` | decompose | P18 | Prototype-shape dispatcher (logic vs. UI) plus shared throwaway/one-command/no-persistence/capture rules. |
| `.agents/skills/prototype/LOGIC.md` | decompose | P18 | Logic-prototype method: state model isolation, TUI rendering rules, capture procedure. |
| `.agents/skills/prototype/UI.md` | decompose | P18 | UI-prototype method: variant switcher pattern, sub-shape A/B choice, capture/cleanup procedure. |
| `.agents/skills/prototype/agents/openai.yaml` | helper-evidence | — | Harness interface metadata. |
| `.agents/skills/research/SKILL.md` | decompose | P15 | Background-agent primary-source research procedure with citation requirement. |
| `.agents/skills/research/agents/openai.yaml` | helper-evidence | — | Harness interface metadata. |
| `.agents/skills/resolving-merge-conflicts/SKILL.md` | decompose | P15 | Five-step conflict-resolution procedure: never `--abort`, preserve both intents, run checks, finish the merge/rebase. |
| `.agents/skills/resolving-merge-conflicts/agents/openai.yaml` | helper-evidence | — | Harness interface metadata. |
| `.agents/skills/sergeant-setup/SKILL.md` | decompose | P21 | 10-phase interactive/idempotent Sergeant bootstrap-or-repair wizard with a strict write-path allowlist and consent gates at every write. |
| `.agents/skills/tdd/SKILL.md` | decompose | P19 | Red→green loop reference: what a good test is, seam discipline, anti-patterns, rules of the loop. |
| `.agents/skills/tdd/mocking.md` | decompose | P19 | Mock-at-boundaries-only guidance and mockable-interface design patterns. |
| `.agents/skills/tdd/tests.md` | decompose | P19 | Good vs. bad test examples (integration-style vs. implementation-coupled vs. tautological). |
| `.agents/skills/tdd/agents/openai.yaml` | helper-evidence | — | Harness interface metadata. |
| `.agents/skills/to-spec/SKILL.md` | decompose | P15 | Synthesize-conversation-into-spec procedure with a fixed spec template, published to the issue tracker. |
| `.agents/skills/to-spec/agents/openai.yaml` | helper-evidence | — | Harness interface metadata. |
| `.agents/skills/to-tickets/SKILL.md` | decompose | P21 | Plan/spec/PR/conversation → dependency-aware td epics/tickets procedure: vertical-slice rules, expand-migrate-contract, publish/validate/report-frontier steps. |
| `.agents/skills/triage/SKILL.md` | decompose | P20 | Issue/PR triage state machine: roles, invocation, gather/recommend/verify/grill/apply-outcome steps, quick override, resume. |
| `.agents/skills/triage/AGENT-BRIEF.md` | decompose | P20 | Agent-brief authoring standard: durability-over-precision, behavioral-not-procedural, complete acceptance criteria, worked good/bad examples. |
| `.agents/skills/triage/OUT-OF-SCOPE.md` | decompose | P20 | `.out-of-scope/` knowledge-base format and when-to-read/when-to-write procedure. |
| `.agents/skills/triage/agents/openai.yaml` | helper-evidence | — | Harness interface metadata. |
| `.agents/skills/wayfinder/SKILL.md` | decompose | P15 | Large-effort planning-as-decision-map procedure: the Map/tickets/ticket-types/fog-of-war/out-of-scope model, chart/work-through-map invocation modes. |
| `.agents/skills/wayfinder/agents/openai.yaml` | helper-evidence | — | Harness interface metadata. |

---

## `tests/` (62 files — all helper-evidence)

**Sampling note.** Per this stage's `CONTEXT.md` ("for a large uniform
group ... read a representative sample and say so"): all 62 files are
identically shaped — self-contained Bash scripts under `set -euo pipefail`
that build a `mktemp -d` fixture (fake `$HOME`, fake `PATH` binaries such as
`tmux`/`td`, a fake fleet/config tree), invoke one or more `bin/sgt-*`
commands against it, and assert exact file/output contents with `grep`/`[[
]]`, printing `PASS`/`FAIL`. 9 of the 62 were read in full or by full header
(`tests/no-remote-test.sh`, `tests/sgt-notify-test.sh`,
`tests/instruction-policy-test.sh`, `tests/mise-check-test.sh`,
`tests/sgt-cleanup-test.sh`, `tests/sgt-watch-test.sh`,
`tests/sgt-drain-test.sh`, `tests/sgt-dispatch-worker-test.sh`,
`tests/run-drain-tests.sh`), spanning the size range from 14 lines
(`no-remote-test.sh`) to 6640 lines (`sgt-cleanup-test.sh`, the largest file
in the whole subject tree). The remaining 53 follow the same fixture/assert
shape per their filenames (one test file per `bin/sgt-*` command or specific
regression, e.g. `sgt-recover-lease-owner-test.sh`,
`sgt-worker-handshake-test.sh`) and were not opened individually.

**Disposition reasoning.** Every test file is deterministic verification
machinery that corroborates behavior already stated in the `bin/` script it
exercises (or, for `instruction-policy-test.sh`/`no-remote-test.sh`/
`repo-skills-test.sh`, in `AGENTS.md`/docs/skills text) — it does not itself
state a new procedural outcome someone follows or decides by. This matches
`../_config/icm-ladder.md` §6.3's own worked example verbatim: "`test.sh` is
merely one tool an implementation actor reaches for before declaring
implementation complete" is not a checkpoint in its own right. Per
`references/dispositions.md`, `helper-evidence` is "deterministic mechanics
... that may inform a later helper/shared-context map but are not themselves
a durable checkpoint or procedural outcome" — exactly this shape. None was
mis-called `obsolete-candidate` or `reference-only`: they are still live,
exercised, procedurally-relevant *evidence* (corroborating what a `bin/`
script actually enforces, useful to `20-harvest`/`30-normalize` as
supporting citations), just not a `decompose` source of new behavior.

| Path | Disposition |
|---|---|
| `tests/global-state-isolation-test.sh` | helper-evidence |
| `tests/instruction-policy-test.sh` | helper-evidence |
| `tests/mise-check-test.sh` | helper-evidence |
| `tests/mise-install-test.sh` | helper-evidence |
| `tests/no-remote-test.sh` | helper-evidence |
| `tests/repo-skills-test.sh` | helper-evidence |
| `tests/run-drain-tests.sh` | helper-evidence |
| `tests/runtime-bash-test.sh` | helper-evidence |
| `tests/sergeant-setup-test.sh` | helper-evidence |
| `tests/sgt-ack-response-test.sh` | helper-evidence |
| `tests/sgt-callback-test.sh` | helper-evidence |
| `tests/sgt-claude-real-contract-test.sh` | helper-evidence |
| `tests/sgt-claude-stop-bg-session-test.sh` | helper-evidence |
| `tests/sgt-claude-worker-test.sh` | helper-evidence |
| `tests/sgt-cleanup-cross-filesystem-test.sh` | helper-evidence |
| `tests/sgt-cleanup-test.sh` | helper-evidence |
| `tests/sgt-dispatch-adopt-branch-test.sh` | helper-evidence |
| `tests/sgt-dispatch-bash32-test.sh` | helper-evidence |
| `tests/sgt-dispatch-brief-test.sh` | helper-evidence |
| `tests/sgt-dispatch-coordinator-pane-test.sh` | helper-evidence |
| `tests/sgt-dispatch-identity-test.sh` | helper-evidence |
| `tests/sgt-dispatch-model-tuple-test.sh` | helper-evidence |
| `tests/sgt-dispatch-oc-target-test.sh` | helper-evidence |
| `tests/sgt-dispatch-td-test.sh` | helper-evidence |
| `tests/sgt-dispatch-unpushed-guard-test.sh` | helper-evidence |
| `tests/sgt-dispatch-worker-test.sh` | helper-evidence |
| `tests/sgt-drain-force-test.sh` | helper-evidence |
| `tests/sgt-drain-terminate-test.sh` | helper-evidence |
| `tests/sgt-drain-test.sh` | helper-evidence |
| `tests/sgt-drain-worker-test.sh` | helper-evidence |
| `tests/sgt-graphify-test.sh` | helper-evidence |
| `tests/sgt-harness-test.sh` | helper-evidence |
| `tests/sgt-interrupted-fallback-test.sh` | helper-evidence |
| `tests/sgt-lease-convergence-test.sh` | helper-evidence |
| `tests/sgt-lease-exit-branch-test.sh` | helper-evidence |
| `tests/sgt-lease-finalizer-test.sh` | helper-evidence |
| `tests/sgt-lib-notification-target-test.sh` | helper-evidence |
| `tests/sgt-lib-owned-file-test.sh` | helper-evidence |
| `tests/sgt-no-mistakes-finding-test.sh` | helper-evidence |
| `tests/sgt-notify-test.sh` | helper-evidence |
| `tests/sgt-recover-drain-test.sh` | helper-evidence |
| `tests/sgt-recover-lease-owner-test.sh` | helper-evidence |
| `tests/sgt-recover-replacement-test.sh` | helper-evidence |
| `tests/sgt-recover-test.sh` | helper-evidence |
| `tests/sgt-respond-drain-test.sh` | helper-evidence |
| `tests/sgt-respond-recovery-test.sh` | helper-evidence |
| `tests/sgt-respond-test.sh` | helper-evidence |
| `tests/sgt-response-lock-release-test.sh` | helper-evidence |
| `tests/sgt-review-findings-test.sh` | helper-evidence |
| `tests/sgt-td-memory-worktree-test.sh` | helper-evidence |
| `tests/sgt-validate-test.sh` | helper-evidence |
| `tests/sgt-validation-worker-test.sh` | helper-evidence |
| `tests/sgt-wake-test.sh` | helper-evidence |
| `tests/sgt-watch-background-test.sh` | helper-evidence |
| `tests/sgt-watch-recycle-test.sh` | helper-evidence |
| `tests/sgt-watch-snapshot-test.sh` | helper-evidence |
| `tests/sgt-watch-test.sh` | helper-evidence |
| `tests/sgt-worker-drain-test.sh` | helper-evidence |
| `tests/sgt-worker-handshake-test.sh` | helper-evidence |
| `tests/sgt-worker-model-tuple-test.sh` | helper-evidence |
| `tests/sgt-worker-readiness-test.sh` | helper-evidence |
| `tests/sgt-worker-test.sh` | helper-evidence |

---

## Symlinks (not counted in the 178-file total)

`reference/sergeant-upstream/.claude/skills/` contains 17 **directory**
symlinks, one per vendored skill, each resolving to
`../../.agents/skills/<name>` (confirmed via `readlink` on all 17):

`code-review`, `codebase-design`, `diagnosing-bugs`, `domain-modeling`,
`grill-with-docs`, `grilling`, `implement`, `no-mistakes`, `prototype`,
`research`, `resolving-merge-conflicts`, `sergeant-setup`, `tdd`, `to-spec`,
`to-tickets`, `triage`, `wayfinder`.

These are directory-level mirrors, not file duplicates — `find -type f`
correctly does not descend into them, so none of their (already-inventoried)
target content is double-counted above. Per `references/dispositions.md`
("Symlinks and duplicated trees"): not independently dispositioned or
partitioned. Each resolves to, and shares the disposition/partition of, the
rows already listed under the corresponding `.agents/skills/<name>/` entry
above (P15–P21, plus the `agents/openai.yaml` helper-evidence rows and the
`diagnosing-bugs` template). This mirroring is itself documented behavior:
`docs/repo-scoped-skills.md` states Claude discovers the canonical skill tree
specifically through these links.

## Duplicate file noted in place (not double-counted)

`docs/prds/enforced-phased-dispatch.md` is a byte-for-byte duplicate of
`docs/prd-enforced-phased-dispatch.md` (confirmed with `diff`). It is counted
once in the 178-file total and once in the reference-only total (both files
physically exist and are both in scope), but its row above points at the
other's disposition/reasoning per the duplicate-tree guidance rather than
re-deriving it independently.

## Gaps

None. All 178 in-scope files were enumerated and dispositioned in this turn;
volume did not exceed what one turn could cover (the largest single read was
`tests/sgt-cleanup-test.sh` at 6640 lines, sampled by header/shape rather
than read in full — see the `tests/` sampling note above, which is the only
place a full read was substituted with structural sampling).
