# Source inventory — `reference/sergeant-upstream` @ `f430cfd4f90174a98adbd7abebbece6303817929`

Produced by `10-inventory` per `../CONTEXT.md`, against `../00-contract/output/contract.md`
(not `# AMBIGUOUS — NOT RESOLVED`; ordinary work proceeded) and the legend in
`references/dispositions.md`.

**Enumeration method:** vendored-subtree case (contract §1 — no `.git` under the
subtree; the working tree at the outer worktree's current commit is itself the
pinned snapshot). Enumerated with `find reference/sergeant-upstream -type f | sort`,
restricted to contract §2/§3 scope and exclusions. `reference-corpus/` does not
exist anywhere under this outer worktree (confirmed by `find` before enumerating,
per this run's blindness rule — `../_config/run-discipline.md` §1 — and per
contract §3's own check) so nothing needed excluding on that account, and no
directory or file under it was opened.

Every row below was opened and read (not guessed from a filename), except where a
row states it stands for a uniform group and names the sampled members. `tests/*`
rows read at minimum the file's header comment / seam-under-test description (all
62 files); most were read further as needed to confirm what a filename alone would
not settle.

## Discrepancy noted against `contract.md`

Contract §3 excludes "VCS internals / build-dependency output (`.git/`, `target/`,
`node_modules/`, `dist/`, vendored lock caches, and the like)" as a **category**,
and separately states as a checked fact that "no build/dependency-output directory
is currently present under it." That checked fact is not accurate:
`bin/__pycache__/sgt-callbackcpython-312.pyc` exists — a compiled Python bytecode
cache (`file` reports "Byte-compiled Python module for CPython 3.12"), the `.py`
timestamp/size sidecar kind of artifact that category exists to describe. Rather
than treat the contract's inaccurate "checked" line as controlling over its own
stated category rule, this stage applied the category as written and **excluded**
this one file from the inventory below. This is a fact for `90-reconcile` to fold
back into the contract record, not a `# AMBIGUOUS — NOT RESOLVED` condition — the
exclusion rule itself resolves the question; only the contract's supporting
"checked" prose was wrong.

## Symlinks (not separately inventoried)

`reference/sergeant-upstream/.claude/skills/<name>` is a directory symlink to
`../../.agents/skills/<name>` for all 17 skills below (`code-review`,
`codebase-design`, `diagnosing-bugs`, `domain-modeling`, `grill-with-docs`,
`grilling`, `implement`, `no-mistakes`, `prototype`, `research`,
`resolving-merge-conflicts`, `sergeant-setup`, `tdd`, `to-spec`, `to-tickets`,
`triage`, `wayfinder`). `find -type f` does not traverse a symlinked directory and
does not list the symlink itself as a file, so none of the files reachable only
through `.claude/skills/*` appear a second time in the 179-file enumeration or in
the rows below — every one of them already has exactly one row under its
`.agents/skills/<name>/...` path. No separate disposition is recorded for the
`.claude/skills/*` symlinks themselves; per `references/dispositions.md`'s
symlink guidance, they carry whatever disposition their target file carries and
are not re-partitioned.

## Totals

- Files found by enumeration: **179**
- Excluded per contract §3 build-dependency-output category (see Discrepancy
  note above): **1** (`bin/__pycache__/sgt-callbackcpython-312.pyc`)
- Files inventoried below: **178**

| Disposition | Count |
|---|---:|
| decompose | 136 |
| helper-evidence | 27 |
| obsolete-candidate | 0 |
| reference-only | 15 |
| **Total** | **178** |

136 + 27 + 0 + 15 = 178. ✓ matches files inventoried.

No `obsolete-candidate` rows: nothing in scope was found to be structurally
replaced by an already-settled fact this stage could cite (the register/ruling
kind of citation `references/dispositions.md` requires); anything that looked
superseded-ish (e.g. the ADR recording `oc-inject`'s deletion) documents a
decision rather than being itself a mechanism some settled fact replaces, so it
is `reference-only`, not `obsolete-candidate`.

---

## 1. Root files (5)

| Path | Disposition | What it is | Reason |
|---|---|---|---|
| `.gitignore` | helper-evidence | Two ignore patterns (`.DS_Store`, `.todos/`) | Deterministic VCS mechanics, not a procedure anyone follows |
| `AGENTS.md` | decompose | Coordinator/executor role definition, dispatch-mode triggers, repo/project resolution rules | States procedural outcomes an agent session follows at start |
| `Dockerfile.test` | helper-evidence | Minimal Debian image definition for the drain test suite | Build/test infra mechanism, not itself a checkpoint |
| `LICENSE` | reference-only | MIT license text, Copyright (c) 2026 Lars Cromley | License text |
| `README.md` | decompose | Project overview: genesis/inspiration, what Sergeant is, top-level orientation | Authored procedural/orientation content, same class as `AGENTS.md` |

## 2. `bin/` (37)

### 2a. Compiled artifact — excluded, not a row (1)

`bin/__pycache__/sgt-callbackcpython-312.pyc` — see Discrepancy note above.

### 2b. Sourced helper libraries (7) — `helper-evidence`

Each header states "Source this file; do not execute it directly" — shared
mechanics consumed by the `sgt-*` commands below, not independent procedures.

| Path | What it is |
|---|---|
| `bin/_sgt-bash-version.sh` | Minimum-Bash-version guard shared by every entry point |
| `bin/_sgt-drain.sh` | Drain-state helper functions (is-drained, state-dir resolution, drain write) |
| `bin/_sgt-harness.sh` | Single registry of accepted interactive harnesses (capability gate, readiness probe, launch invocation) |
| `bin/_sgt-intent.sh` | Intent-revision hashing and `no-mistakes --version` helpers |
| `bin/_sgt-lib.sh` | Shared `_die`/`_info`/`_require_*`/path-resolution helpers and `SGT_*` env vars |
| `bin/_sgt-response-lock.sh` | Response-archive format and serialization/locking for publish/consume |
| `bin/_sgt-review-axes.sh` | Canonical independent-review axis/severity vocabulary shared by dispatch and review-findings |

### 2c. `sgt-*` command surface (29) — `decompose`, partition **bin: the `sgt-*` CLI**, split into five named sub-groups for coherence

**bin: fleet dispatch & lifecycle** (14) — `bin/sgt-dispatch`, `bin/sgt-dag-dispatch-hook`, `bin/sgt-dag-run`, `bin/sgt-cleanup`, `bin/sgt-drain`, `bin/sgt-drain-force`, `bin/sgt-undrain`, `bin/sgt-recover`, `bin/sgt-respond`, `bin/sgt-ack-response`, `bin/sgt-watch`, `bin/sgt-wake`, `bin/sgt-interactive-worker`, `bin/sgt-notify`

**bin: validation & review gates** (4) — `bin/sgt-validate`, `bin/sgt-validation-worker`, `bin/sgt-no-mistakes-finding`, `bin/sgt-review-findings`

**bin: project & config commands** (5) — `bin/sgt-list`, `bin/sgt-context`, `bin/sgt-status`, `bin/sgt-sync`, `bin/sgt-treehouse-init`

**bin: td task-tracker integration** (3) — `bin/sgt-td-create`, `bin/sgt-td-list`, `bin/sgt-td-memory`

**bin: callback, graph, and wiki utilities** (3) — `bin/sgt-callback`, `bin/sgt-graphify`, `bin/wiki-daily-digest`

| Path | What it is | Reason |
|---|---|---|
| `bin/sgt-dispatch` | Creates a worktree per repo, writes a mission brief, spawns an agent in tmux | States the dispatch procedure end to end |
| `bin/sgt-dag-dispatch-hook` | Stage hook `dagr` calls when a DAG stage becomes ready; calls `sgt-dispatch` and writes tracking files | Procedural glue between two durable subsystems (dagr, fleet state) |
| `bin/sgt-dag-run` | Reads a project's `dag:` block, creates/starts a `dagr` run, dispatches initially-ready stages | States a multi-step run-creation procedure |
| `bin/sgt-cleanup` | Removes worktrees/fleet state for a completed task (treehouse-leased and plain worktrees) | Terminal-state procedure with explicit atomicity requirements |
| `bin/sgt-drain` | Set/remove a persistent drain on a project or globally | Admission-control procedure with its own usage grammar |
| `bin/sgt-drain-force` | Force-stop workers that failed cooperative drain | Explicit-consent escalation procedure |
| `bin/sgt-undrain` | Remove a drain record, idempotently | Small but real procedure (restores admission) |
| `bin/sgt-recover` | One bounded stall-recovery attempt: kill stalled pane, relaunch, update fleet metadata, notify | Recovery procedure with stated gating condition |
| `bin/sgt-respond` | Deliver a response and resume a dead waiting worker when needed | Core response-delivery procedure |
| `bin/sgt-ack-response` | Acknowledge and clear one consumed worker response | Small explicit procedure |
| `bin/sgt-watch` | Monitor a dispatched fleet and report outcomes | Observability procedure with its own state machine |
| `bin/sgt-wake` | Evaluate a durable wake condition, resume via `sgt-respond` or record a failed attempt | Conditional-resume procedure |
| `bin/sgt-interactive-worker` | Own one persistent interactive agent pane | Core worker-lifecycle procedure |
| `bin/sgt-notify` | Inject a worker update into the primary session; durable wake-marker transport | Escalation/notification procedure |
| `bin/sgt-validate` | Launch coordinator-owned `no-mistakes` beside an interactive worker | Validation-launch procedure |
| `bin/sgt-validation-worker` | Run coordinator-owned `no-mistakes` in an interactive pane | Companion procedure to `sgt-validate` |
| `bin/sgt-no-mistakes-finding` | Apply a disposition to one no-mistakes finding | Explicit disposition procedure |
| `bin/sgt-review-findings` | Route structured independent-review findings to `td` | Routing procedure |
| `bin/sgt-list` | List all known Sergeant projects | Trivial but real command |
| `bin/sgt-context` | Emit a structured agent-context block (instruction layering) for a project | Orientation procedure agents run at session start |
| `bin/sgt-status` | Show git status across every repo in a project | Trivial but real command |
| `bin/sgt-sync` | Clone missing repos, pull existing ones, for a project | Real procedure |
| `bin/sgt-treehouse-init` | Initialize treehouse pools in a project's repos | Real procedure with group filtering |
| `bin/sgt-td-create` | Create `td` tasks in project repos for a cross-repo brief, all-or-nothing | Real procedure with explicit failure semantics |
| `bin/sgt-td-list` | Show `td` tasks across all repos in a project | Real query procedure |
| `bin/sgt-td-memory` | Record non-secret worker recovery pointers in `td` | Real procedure |
| `bin/sgt-callback` | Durable, profile-bound callback events for fleet tasks (Python) | Substantial (960-line) protocol implementation, clearly behavior-bearing |
| `bin/sgt-graphify` | Run graphify across all repos in a project, publish atomically | Real procedure with an atomicity guarantee |
| `bin/wiki-daily-digest` | Synthesize a daily wiki digest from AI agent session history across three tools | Real synthesis procedure |

## 3. `docs/` (21)

| Path | Disposition | What it is | Reason |
|---|---|---|---|
| `docs/README.md` | decompose | Doc-set orientation, "Start here" table | Navigational procedure |
| `docs/adr-oc-inject-deletion.md` | reference-only | ADR: decided/executed deletion of `oc-inject` prototype | Historical decision record, not a procedure to follow (the mechanism it discusses is already gone) |
| `docs/audit-2026-07.md` | reference-only | "Sergeant — System Audit, July 2026" | Explicitly a dated audit document |
| `docs/callbacks.md` | decompose | Durable Callback Protocol v1 specification | Genuine protocol spec someone implements/consumes by |
| `docs/dead-code-2026-07.md` | reference-only | "Sergeant — Dead Code Audit, July 2026", items D-1…D-9, most already actioned | Dated audit document |
| `docs/getting-started.md` | decompose | Install checklist: prerequisites, steps, first project | Procedural checklist |
| `docs/prd-enforced-phased-dispatch.md` | reference-only | PRD, "Status: Draft, awaiting explicit human PRD approval" | Draft PRD |
| `docs/prds/axi-agent-ergonomics.md` | reference-only | PRD, "Status: Draft — Phase 1 implementation in progress" | Draft PRD |
| `docs/prds/claude-background-harness.md` | reference-only | PRD, "Status: Draft" | Draft PRD |
| `docs/prds/code-improvements.md` | reference-only | PRD, "Status: Draft" | Draft PRD |
| `docs/prds/enforced-phased-dispatch.md` | reference-only (duplicate) | Byte-for-byte identical to `docs/prd-enforced-phased-dispatch.md` (verified: `diff -q` reports no difference) | Points at the same content as `docs/prd-enforced-phased-dispatch.md`'s row; same disposition, not re-partitioned, per dispositions.md's duplicate-tree guidance |
| `docs/prds/tasks-axi-migration.md` | reference-only | PRD, "Status: Draft" | Draft PRD |
| `docs/repo-scoped-skills.md` | decompose | Explains how `.agents/skills/` is discovered by Codex/OpenCode/Claude | States the actual discovery mechanism agents rely on |
| `docs/research/axi-agent-ergonomics-spike.md` | reference-only | Dated research spike with a recorded decision | Research spike (historical draft) |
| `docs/research/claude-background-harness-spike.md` | reference-only | Dated research spike with a recorded decision | Research spike (historical draft) |
| `docs/research/tasks-axi-configurable-workflows.md` | reference-only | Dated research spike, sources inspected | Research spike (historical draft) |
| `docs/schema.md` | decompose | Project YAML schema documentation (global config + per-project fields) | Specification a project author follows |
| `docs/skills.md` | decompose | Skill locations and their sources/trust levels | Real usage/governance documentation |
| `docs/troubleshooting.md` | decompose | Command-not-found and other diagnostic procedures | Procedural troubleshooting steps |
| `docs/using-sergeant.md` | decompose | Core usage guide: project context, direct vs. dispatch mode | Core procedural documentation |
| `docs/what-is-sergeant.md` | decompose | Product definition, audience, deployment model | Foundational orientation content, same class as `README.md` |

## 4. Root config & task automation (2)

| Path | Disposition | What it is | Reason |
|---|---|---|---|
| `mise.toml` | decompose | `[tasks.install]`, `install:hooks`, `uninstall:hooks`, `uninstall`, `check`, `test:docker:drain`, `update` — each with an embedded bash procedure | Each task body is a real, followable installation/maintenance procedure (e.g. `install` links `sgt-*` scripts onto `PATH` and installs git hooks) |
| `schema/project.yaml.example` | decompose | Fully-commented example project config (required/optional fields, groups, dag block) | A specification a project author follows field by field, not just a mechanism |

## 5. `opencode.json`, `scripts/hooks/pre-push`, `templates/worker-brief.md` (3) — `helper-evidence`

| Path | What it is | Reason |
|---|---|---|
| `opencode.json` | Declares OpenCode's skill discovery paths (`.agents/skills`, `skills`) | Deterministic config, not a procedure |
| `scripts/hooks/pre-push` | Git pre-push hook that runs `mise run test:docker:drain`, skippable via `SKIP_DRAIN_TESTS` | Pure wiring/mechanism installed by `mise run install`, no independent decision procedure of its own |
| `templates/worker-brief.md` | `{{PLACEHOLDER}}`-driven brief template filled in by `sgt-dispatch` | The dispositions legend's own example of helper-evidence ("a template script invoked by an actor elsewhere") |

## 6. `.agents/skills/` (44)

Top-level provenance/license files (2) — `reference-only`:

| Path | What it is |
|---|---|
| `.agents/skills/PROVENANCE.md` | Records the upstream lock source (`mattpocock/skills`) and per-skill locked folder hashes |
| `.agents/skills/THIRD_PARTY_NOTICES.md` | MIT notice text for the 14 redistributed skills |

Skill definitions (17) — `decompose`, partition **agents-skills: vendored skill definitions**:

`code-review`, `codebase-design`, `diagnosing-bugs`, `domain-modeling`,
`grill-with-docs`, `grilling`, `implement`, `no-mistakes`, `prototype`,
`research`, `resolving-merge-conflicts`, `sergeant-setup`, `tdd`, `to-spec`,
`to-tickets`, `triage`, `wayfinder` — each contributes one
`.agents/skills/<name>/SKILL.md`. Each states a real trigger condition and
procedure (e.g. `diagnosing-bugs/SKILL.md`: "Diagnosis loop for hard bugs and
performance regressions"; `sergeant-setup/SKILL.md`: idempotent
bootstrap/repair procedure with explicit consent gates).

Skill supporting references (10) — `decompose`, partition **agents-skills: vendored skill supporting references**:

| Path | What it is |
|---|---|
| `.agents/skills/codebase-design/DEEPENING.md` | How to deepen a cluster of shallow modules, given dependencies |
| `.agents/skills/codebase-design/DESIGN-IT-TWICE.md` | Parallel sub-agent pattern for exploring alternative interfaces |
| `.agents/skills/domain-modeling/ADR-FORMAT.md` | ADR file location, numbering, and template |
| `.agents/skills/domain-modeling/CONTEXT-FORMAT.md` | `CONTEXT.md` structure template |
| `.agents/skills/prototype/LOGIC.md` | Terminal-app prototyping approach for business logic/state |
| `.agents/skills/prototype/UI.md` | Multi-variant UI prototyping approach |
| `.agents/skills/tdd/mocking.md` | When to mock (system boundaries only) vs. not |
| `.agents/skills/tdd/tests.md` | Good vs. bad test shape, with examples |
| `.agents/skills/triage/AGENT-BRIEF.md` | How to write an agent brief for a ready-for-agent issue/PR |
| `.agents/skills/triage/OUT-OF-SCOPE.md` | Purpose and format of a repo's `.out-of-scope/` knowledge base |

Skill display-metadata (14) — `helper-evidence`:

`.agents/skills/{code-review,codebase-design,diagnosing-bugs,domain-modeling,
grill-with-docs,grilling,implement,prototype,research,
resolving-merge-conflicts,tdd,to-spec,triage,wayfinder}/agents/openai.yaml` —
each is a 2-3 line `interface: display_name / short_description` stanza; the
dispositions legend's own example of helper-evidence ("a display-metadata
file"). (`no-mistakes`, `sergeant-setup`, `to-tickets` have no `agents/`
subdirectory — confirmed by the enumeration, not an omission here.)

Skill template script (1) — `helper-evidence`:

| Path | What it is | Reason |
|---|---|---|
| `.agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh` | "Copy this file, edit the steps below, and run it" human-in-the-loop reproduction template | A template invoked by an actor elsewhere, not itself a procedure |

## 7. `skills/` (top-level, 5) — `decompose`, partition **skills: Sergeant-owned project skills**

| Path | What it is |
|---|---|
| `skills/cross-repo-work/SKILL.md` | Decompose ownership/dependencies/merge order across repos before dispatch |
| `skills/dispatch/SKILL.md` | Plan and execute a cross-repo task via `sgt-dispatch` |
| `skills/load-project/SKILL.md` | Resolve project ownership/config/paths before work begins |
| `skills/sergeant-help/SKILL.md` | Answer installation/usage/troubleshooting questions from repo-owned docs |
| `skills/wiki/SKILL.md` | Maintain automatic activity captures and the daily session digest |

## 8. `tests/` (62) — `decompose`, partition **tests: behavioral regression & contract suite**, split into seven named sub-groups

Every row's `decompose` call rests on the same basis: these are not generic
harness mechanics but named regression/contract tests, most citing a specific
incident, GitHub issue, or `td-` ticket and stating the exact behavioral
guarantee being locked in (e.g. `sgt-drain-terminate-test.sh`: a `kill -TERM
"$BASHPID"` that only killed a backgrounded watcher subshell, leaving 11
workers "drained" with live panes). Per `references/dispositions.md`'s
stated asymmetry, that is exactly the behavior-bearing content the
`decompose`/`helper-evidence` line is meant to catch on the `decompose` side.

**tests: dispatch & worker launch** (18) — `sgt-dispatch-adopt-branch-test.sh`,
`sgt-dispatch-bash32-test.sh`, `sgt-dispatch-brief-test.sh`,
`sgt-dispatch-coordinator-pane-test.sh`, `sgt-dispatch-identity-test.sh`,
`sgt-dispatch-model-tuple-test.sh`, `sgt-dispatch-oc-target-test.sh`,
`sgt-dispatch-td-test.sh`, `sgt-dispatch-unpushed-guard-test.sh`,
`sgt-dispatch-worker-test.sh`, `sgt-worker-handshake-test.sh`,
`sgt-worker-model-tuple-test.sh`, `sgt-worker-readiness-test.sh`,
`sgt-worker-test.sh`, `sgt-claude-worker-test.sh`,
`sgt-claude-real-contract-test.sh` (opt-in, real `claude` CLI),
`sgt-claude-stop-bg-session-test.sh`, `sgt-harness-test.sh`

**tests: drain lifecycle** (8) — `sgt-drain-test.sh`, `sgt-drain-force-test.sh`,
`sgt-drain-terminate-test.sh`, `sgt-drain-worker-test.sh`,
`sgt-worker-drain-test.sh`, `sgt-recover-drain-test.sh`,
`sgt-respond-drain-test.sh`, `run-drain-tests.sh` (the drain suite runner
itself, covering GH #81/#82 and adjacent functionality)

**tests: recovery & lease/notification finalization** (7) —
`sgt-recover-test.sh`, `sgt-recover-lease-owner-test.sh`,
`sgt-recover-replacement-test.sh`, `sgt-lease-convergence-test.sh`,
`sgt-lease-exit-branch-test.sh`, `sgt-lease-finalizer-test.sh`,
`sgt-interrupted-fallback-test.sh`

**tests: respond, wake, and callback delivery** (8) — `sgt-respond-test.sh`,
`sgt-respond-recovery-test.sh`, `sgt-response-lock-release-test.sh`,
`sgt-ack-response-test.sh`, `sgt-wake-test.sh`, `sgt-callback-test.sh`,
`sgt-notify-test.sh`, `sgt-lib-notification-target-test.sh`

**tests: watch, graphify, and fleet observability** (6) — `sgt-watch-test.sh`,
`sgt-watch-background-test.sh`, `sgt-watch-recycle-test.sh`,
`sgt-watch-snapshot-test.sh`, `sgt-graphify-test.sh`,
`sgt-td-memory-worktree-test.sh`

**tests: validation & review gates** (4) — `sgt-validate-test.sh`,
`sgt-validation-worker-test.sh`, `sgt-no-mistakes-finding-test.sh`,
`sgt-review-findings-test.sh`

**tests: environment, policy, and setup guardrails** (11) —
`global-state-isolation-test.sh`, `instruction-policy-test.sh`,
`mise-check-test.sh`, `mise-install-test.sh`, `no-remote-test.sh`,
`repo-skills-test.sh`, `runtime-bash-test.sh`, `sergeant-setup-test.sh`,
`sgt-lib-owned-file-test.sh`, `sgt-cleanup-test.sh`,
`sgt-cleanup-cross-filesystem-test.sh`

18+8+7+8+6+4+11 = 62. ✓

---

## Reconciliation

- Section 1 (root files): 5
- Section 2 (`bin/`): 37 found, 1 excluded, 36 inventoried (7 helper-evidence + 29 decompose)
- Section 3 (`docs/`): 21 (6 reference-only PRDs/audits/spikes stand alone + 1 duplicate reference-only + 9 decompose... see exact table above)
- Section 4 (root config & task automation): 2
- Section 5 (`opencode.json`/hook/template): 3
- Section 6 (`.agents/skills/`): 44 (2 reference-only + 17 decompose + 10 decompose + 14 helper-evidence + 1 helper-evidence)
- Section 7 (`skills/`): 5
- Section 8 (`tests/`): 62

5 + 36 + 21 + 2 + 3 + 44 + 5 + 62 = 178, matching "Files inventoried" above
(179 found − 1 excluded).

By disposition, counted directly from the rows/partitions above:

- **decompose** = 2 (root) + 29 (bin) + 9 (docs) + 2 (root config) + 17 + 10 (agents-skills) + 5 (skills/) + 62 (tests) = **136**
- **helper-evidence** = 7 (bin libs) + 3 (opencode.json, pre-push, worker-brief.md) + 14 + 1 (agents-skills) = **27**
- **obsolete-candidate** = **0**
- **reference-only** = 2 (LICENSE + root... wait: LICENSE(1) + agents-skills PROVENANCE/THIRD_PARTY(2) + docs (6: adr, audit, dead-code, prd-enforced-phased-dispatch, prds/enforced-phased-dispatch duplicate, + 4 more prds + 3 research = recount below) = **15**

Reference-only, listed explicitly for the final count (15): `LICENSE`,
`.agents/skills/PROVENANCE.md`, `.agents/skills/THIRD_PARTY_NOTICES.md`,
`docs/adr-oc-inject-deletion.md`, `docs/audit-2026-07.md`,
`docs/dead-code-2026-07.md`, `docs/prd-enforced-phased-dispatch.md`,
`docs/prds/axi-agent-ergonomics.md`, `docs/prds/claude-background-harness.md`,
`docs/prds/code-improvements.md`, `docs/prds/enforced-phased-dispatch.md`
(duplicate), `docs/prds/tasks-axi-migration.md`,
`docs/research/axi-agent-ergonomics-spike.md`,
`docs/research/claude-background-harness-spike.md`,
`docs/research/tasks-axi-configurable-workflows.md`. Count: 15. ✓

136 + 27 + 0 + 15 = 178. ✓ Matches files inventoried, matches the Totals table
above.

## Ambiguities recorded

- The `bin/__pycache__` discrepancy against `contract.md` (see "Discrepancy
  noted against `contract.md`" above) — resolved within this stage by applying
  the contract's own exclusion **category**, not escalated as
  `# AMBIGUOUS — NOT RESOLVED`, but flagged for `90-reconcile` to fold back
  into the contract record.
- No other ambiguity was hit. Every file in the 179-file enumeration was
  either excluded per the above or assigned exactly one disposition after
  being read.
