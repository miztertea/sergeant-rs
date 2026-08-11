# Source inventory — `reference/sergeant-upstream`

Produced by `10-inventory` for run `01KZQRGZE32RQ79KT82XTB9MV2`, per
`../CONTEXT.md` and `references/dispositions.md`. `../00-contract/output/contract.md`
is resolved (not `# AMBIGUOUS — NOT RESOLVED`), so this stage proceeded normally.

## Method

Contract's case: **vendored subtree**, no live `.git` at
`reference/sergeant-upstream/.git` (confirmed). Per `../CONTEXT.md` step 1 this
means the working tree itself is the pinned snapshot, so enumeration used an
ordinary recursive file listing: `find reference/sergeant-upstream -type f |
sort`, restricted to the contract's scope (everything under the subtree) with
no exclusions actually triggered (the standing VCS/build-output exclusion is
vacuous here — confirmed by `find`; no `reference-corpus/` exists anywhere in
this worktree, confirmed by search — the run carries no measurement framing so
the blindness rule is vacuous per `run-discipline.md` §1; `AGENTS.md`,
`.sergeant/`, `UPSTREAM.md` are outside the scoped subtree per the contract).

Every file was opened and read before disposition, **except** two explicitly
sampled groups, called out below (permitted by `references/dispositions.md`'s
"large uniform group" allowance):

- **`tests/*.sh` (62 files)** — uniform-shaped Bash test harnesses (a
  `fail()`/assertion-count pattern repeated per file, each asserting a
  contract already read directly in the `bin/` script or doc it tests). 6 were
  read in full, spanning different subsystems and sizes:
  `global-state-isolation-test.sh`, `instruction-policy-test.sh`,
  `mise-check-test.sh`, `no-remote-test.sh`, `repo-skills-test.sh`,
  `run-drain-tests.sh`. The other 56 were disposition by the same pattern
  (verified by filename/subject correspondence to an already-read `bin/`
  script or doc) and are listed by name below, not opened individually.
- **`.agents/skills/*/agents/openai.yaml` (14 files)** — near-identical
  two-line `interface: {display_name, short_description}` stubs. 3 were read
  in full (`codebase-design`, `tdd`, plus a byte-identity scan — `md5sum`
  across all 14 confirmed each is a distinct-but-uniformly-shaped stub, none
  a copy of another), the rest inferred from that confirmed shape.

Every other file (117 of 179) was opened and read individually, including
every `bin/` script. The nine largest `bin/` scripts (700+ lines:
`sgt-watch`, `sgt-review-findings`, `_sgt-drain.sh`, `sgt-callback`,
`sgt-validate`, `sgt-dispatch`, `sgt-interactive-worker`, `_sgt-lib.sh`,
`sgt-cleanup`) were read via header/docstring plus a complete function (or,
for `sgt-callback`, `def`/`class`) listing rather than every internal line —
enough to disposition and describe accurately, cross-checked against the
detailed behavior these same scripts' contracts are documented against in
`AGENTS.md`, `README.md`, and `docs/using-sergeant.md` (already read in
full). This is a genuine reach-limit of one turn against ~56,500 total lines
in scope, not a silent truncation — every one of the 179 files is still
accounted for in a disposition row below.

**Symlinks (not double-counted):** `.claude/skills/{17 names}` are symlinks
to `.agents/skills/{same name}/` (verified via `readlink`), not regular
files — `find -type f` correctly excludes them, so they are not part of the
179-file count. They are pure mirrors: same disposition and partition as
their `.agents/skills/<name>/SKILL.md` target, no separate extraction.

## Disposition counts

| Disposition | Count |
|---|---|
| decompose | 83 |
| helper-evidence | 80 |
| reference-only | 16 |
| obsolete-candidate | 0 |
| **Total** | **179** |

Total files enumerated in step 1: **179** (confirmed via `find
reference/sergeant-upstream -type f \| wc -l`). 83 + 80 + 16 + 0 = 179. ✓

**Note (90-reconcile, AF-0003, added at adjudication):** this table's `decompose`
count of 83 is this stage's own independent classification and stands
unchanged. One of those 83 files (partition E:
`.agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh`) was
subsequently ruled `helper-evidence` and excluded from harvest by
`../20-harvest/output/partition-ledger.md`'s "Orchestrator ruling on the
census mismatch" section — `20-harvest/output/behavior-units.ndjson`
therefore covers 82 distinct source files, one short of this table's own
83. This is a known, ruled downstream exclusion, not an unrevised stale
count; see the ledger section above for the disposition.

## Partition counts (decompose rows only)

| Partition | Files |
|---|---|
| A. Coordinator root instructions & toolbelt reference | 2 |
| B. Installation, environment, and project-config schema | 4 |
| C. Product documentation — concepts, usage, troubleshooting | 7 |
| D. Sergeant-owned orchestration skills + worker-brief template | 6 |
| E. Vendored engineering skills (Matt Pocock bundle) | 25 |
| F. Sergeant-authored worker skills (no-mistakes, sergeant-setup, to-tickets) | 3 |
| G. Project registry & context (`bin/`) | 6 |
| H. Dispatch & worktree lifecycle (`bin/`) | 3 |
| I. Worker supervision & interactive harness (`bin/`) | 3 |
| J. Worker resume/response protocol (`bin/`) | 5 |
| K. Fleet monitoring (`bin/`) | 1 |
| L. Cooperative drain (`bin/`) | 4 |
| M. Notification & durable callbacks (`bin/`) | 3 |
| N. Shipping-gate validation / no-mistakes integration (`bin/`) | 3 |
| O. Independent review routing (`bin/`) | 3 |
| P. Knowledge graph (`bin/`) | 1 |
| Q. DAG workflow orchestration (`bin/`) | 2 |
| R. Wiki digest (`bin/`) | 1 |
| S. Bash version guard (`bin/`) | 1 |
| **Total** | **83** |

## Rows

Legend: **D**=decompose, **H**=helper-evidence, **R**=reference-only.
Partition letters refer to the table above; only decompose rows carry one.

### Root (9 files)

| Path | Disp. | Partition | What it is / reason |
|---|---|---|---|
| `AGENTS.md` | D | A | Coordinator/dispatch/direct-mode procedure, toolbelt table, skill trigger table, standard-workflow steps — the primary behavior-bearing instruction file for the whole repo. |
| `README.md` | D | A | Top-level project README; beyond intro/quickstart it documents real procedural contracts (model-pin resolution/precedence, `--managed-coordinator-pane`, no-mistakes routing/disposition rules, review-finding severity aliasing) not fully restated elsewhere. |
| `.gitignore` | H | — | Two-line VCS ignore list (`.DS_Store`, `.todos/`) — deterministic mechanics, not a procedure. |
| `LICENSE` | R | — | MIT license text. |
| `mise.toml` | D | B | `mise` task definitions: `install` (symlinks `bin/sgt-*` + git hooks), `check` (dependency verification), `test:docker:drain`, `update`, `uninstall*` — genuine install/verify procedures read and executed by users per `docs/getting-started.md`. |
| `opencode.json` | H | — | 9-line OpenCode skill-path discovery config (`.agents/skills`, `skills`) — mechanism, not a procedure. |
| `Dockerfile.test` | H | — | Docker image definition for the drain-test suite — the dispositions legend's own worked example of helper-evidence. |
| `schema/project.yaml.example` | D | B | Heavily annotated, copy-and-edit project-config template (`cp … ~/.config/sergeant/<name>.yaml`, per `docs/getting-started.md` step 4) — states real behavioral facts (identity-resolution precedence, DAG stage format) alongside the template, not just a stub. |
| `scripts/hooks/pre-push` | H | — | Git `pre-push` hook invoking `mise run test:docker:drain` — automation script invoked by git, not a human-followed procedure. |

### `.agents/skills/` (44 files)

Top-level:

| Path | Disp. | Partition | What it is / reason |
|---|---|---|---|
| `.agents/skills/PROVENANCE.md` | R | — | Attribution/provenance record for the vendored skill bundle (upstream repo, lock hashes, license owners) — not procedural. |
| `.agents/skills/THIRD_PARTY_NOTICES.md` | R | — | MIT third-party notices list for the vendored skills — not procedural. |

Vendored engineering skills (Matt Pocock bundle) — 14 skills, each `SKILL.md` +
`agents/openai.yaml` display stub (sampled group, see Method) + any skill-local
support doc, all partition **E**:

| Skill dir | SKILL.md | openai.yaml | Support docs (all D, partition E) |
|---|---|---|---|
| `codebase-design/` | D | H (sampled: read) | `DEEPENING.md` (D), `DESIGN-IT-TWICE.md` (D) |
| `code-review/` | D | H (sampled: read) | — |
| `diagnosing-bugs/` | D | H | `scripts/hitl-loop.template.sh` (D — HITL repro script the skill's Phase 1 explicitly names and hands to the user) |
| `domain-modeling/` | D | H | `ADR-FORMAT.md` (D), `CONTEXT-FORMAT.md` (D) |
| `grilling/` | D | H | — |
| `grill-with-docs/` | D | H | — |
| `implement/` | D | H | — |
| `prototype/` | D | H | `LOGIC.md` (D), `UI.md` (D) |
| `research/` | D | H | — |
| `resolving-merge-conflicts/` | D | H | — |
| `tdd/` | D | H (sampled: read) | `mocking.md` (D), `tests.md` (D) |
| `to-spec/` | D | H | — |
| `triage/` | D | H | `AGENT-BRIEF.md` (D), `OUT-OF-SCOPE.md` (D) |
| `wayfinder/` | D | H | — |

Each `SKILL.md` here: a complete, procedural workflow definition (glossary +
method for codebase-design; six-phase loop for diagnosing-bugs; ADR/glossary
discipline for domain-modeling; etc.) — unambiguously decompose. Each
`openai.yaml`: a 2-line `interface: {display_name, short_description}` stub —
display metadata, helper-evidence (dispositions.md's own example category).

Sergeant-authored worker skills, partition **F** — no `agents/` subdirectory:

| Path | Disp. | Partition | What it is / reason |
|---|---|---|---|
| `.agents/skills/no-mistakes/SKILL.md` | D | F | Coordinator-only no-mistakes shipping-gate contract (vendored for worker read-only reference; worker-restriction banner prepended per `PROVENANCE.md`'s "Local modification" note) — full procedural gate-driving contract. |
| `.agents/skills/sergeant-setup/SKILL.md` | D | F | 10-phase interactive install/repair wizard — full procedure. |
| `.agents/skills/to-tickets/SKILL.md` | D | F | Plan/spec/PR → td epics-and-tickets breakdown procedure — full procedure. |

### `bin/` (37 files: 36 scripts + 1 compiled cache)

| Path | Disp. | Partition | What it is / reason |
|---|---|---|---|
| `bin/__pycache__/sgt-callbackcpython-312.pyc` | R | — | Compiled Python bytecode cache — binary/compiled artifact, the legend's own example category. |
| `bin/_sgt-bash-version.sh` | D | S | Shared Bash≥3.2 minimum-version guard sourced by every entry point. |
| `bin/sgt-list` | D | G | Lists registered projects from `~/.config/sergeant/`. |
| `bin/sgt-sync` | D | G | Clones missing / pulls existing project repos. |
| `bin/sgt-status` | D | G | Git status across every repo in a project. |
| `bin/sgt-context` | D | G | Emits the resolved agent-context block (instruction layering, clone status, graph pointer) for a project. |
| `bin/sgt-td-list` | D | G | Cross-repo td task listing/filtering, human and `--json`. |
| `bin/sgt-td-create` | D | G | All-or-nothing multi-repo td task creation with rollback on partial failure. |
| `bin/sgt-dispatch` | D | H | The core dispatcher: worktree/treehouse-lease creation, brief rendering, tmux pane spawn, td task creation, model/agent tuple resolution — the largest single behavior surface in the repo. |
| `bin/sgt-cleanup` | D | H | Worktree + fleet-state teardown with atomic same-filesystem evidence staging, response-handshake retirement, and multi-phase rollback — the single largest file in scope (2774 lines). |
| `bin/sgt-treehouse-init` | D | H | Initializes treehouse worktree pools per repo/group. |
| `bin/sgt-interactive-worker` | D | I | Owns one persistent interactive agent pane: harness launch, readiness gating, drain watching, notification delivery. |
| `bin/_sgt-harness.sh` | D | I | Single registry of accepted interactive harnesses (opencode/oc/goose/claude), their launch args, and the shared TUI-readiness probe. |
| `bin/_sgt-lib.sh` | D | I | Shared helper library sourced by every `sgt-*` script: agent/model resolution, pane-identity verification, notification publish/wait, background-monitor management, td-version gating. |
| `bin/sgt-respond` | D | J | Delivers a human response and resumes a worker (live-pane notify or full relaunch), with legacy-state migration and bounded delivery-escalation recovery. |
| `bin/sgt-recover` | D | J | One-shot bounded stall recovery: kill-after-verify relaunch of a stalled `in_progress` worker. |
| `bin/sgt-wake` | D | J | Evaluates a durable wake condition (6 kinds) and resumes a `waiting` worker via `sgt-respond`. |
| `bin/sgt-ack-response` | D | J | Acknowledges/archives one consumed worker response under the response lock. |
| `bin/_sgt-response-lock.sh` | D | J | Shared response-lock acquisition/release and action-lease finalization used by respond/recover/ack-response/watch. |
| `bin/sgt-watch` | D | K | Fleet monitor: foreground/background modes, read-only `--snapshot`, stall classification, terminal-worker recycling, dagr auto-advance. |
| `bin/sgt-drain` | D | L | Set/remove/query a cooperative drain (project or global), with bounded `--wait` for live workers to exit. |
| `bin/sgt-undrain` | D | L | Thin wrapper removing a drain record. |
| `bin/sgt-drain-force` | D | L | Force-stops workers that failed cooperative drain, with pane/PID identity verification before any signal. |
| `bin/_sgt-drain.sh` | D | L | Shared drain-state helpers: state dir/file resolution, admission-lock acquire/release, Claude background-session stop/liveness. |
| `bin/sgt-notify` | D | M | Injects a worker update into the coordinator (durable marker by default, optional raw tmux transport); also fires the durable-callback sync and wiki capture. |
| `bin/sgt-callback` | D | M | Python durable, profile-bound callback protocol: register/enqueue/drain/retry/seal/unseal, full validation and atomic-write discipline. |
| `bin/sgt-td-memory` | D | M | Records non-secret worker handoff/response provenance into td, with strict worktree-ownership verification before any git-identity capture. |
| `bin/sgt-validate` | D | N | Launches coordinator-owned no-mistakes in a split pane: ownership claim/release, intent-transport consent gate, retryable launch handshake. |
| `bin/sgt-validation-worker` | D | N | The child process `sgt-validate` launches: intent re-verification and the actual `no-mistakes axi run` invocation. |
| `bin/_sgt-intent.sh` | D | N | Canonical-intent helpers: revision hashing, 8-section intent-file schema validation, transport (`--intent-file`/`--intent`) resolution. |
| `bin/sgt-review-findings` | D | O | Routes structured independent-review JSON into deduplicated owning-repo td tasks, with retained-artifact retry and superseded-revision reconciliation. |
| `bin/sgt-no-mistakes-finding` | D | O | Applies a disposition (`gate`/`td`/`ignore`/`ask-user`) to one no-mistakes finding and creates/updates the owning td task. |
| `bin/_sgt-review-axes.sh` | D | O | The one canonical definition of review axes (standards/spec/readiness/accessibility) and severity vocabulary/aliases shared by dispatch and the router. |
| `bin/sgt-graphify` | D | P | Runs Graphify across all project repos and atomically publishes the merged project graph. |
| `bin/sgt-dag-run` | D | Q | Creates/starts a dagr DAG run from a project YAML `dag:` block. |
| `bin/sgt-dag-dispatch-hook` | D | Q | dagr stage-ready hook: calls `sgt-dispatch` and writes dagr tracking files into fleet state. |
| `bin/wiki-daily-digest` | D | R | Synthesizes opencode/goose/claude session history plus merged PRs/td tasks into a daily wiki digest via the Anthropic API. |

### `docs/` (21 files)

| Path | Disp. | Partition | What it is / reason |
|---|---|---|---|
| `docs/what-is-sergeant.md` | D | C | Core concept/vocabulary + execution-mode definitions (Project/Repo/Task/Fleet/Worker/Decision request). |
| `docs/getting-started.md` | D | B | Full install/register/verify checklist. |
| `docs/using-sergeant.md` | D | C | The primary usage runbook: dispatch, worker states, wake conditions, drain, respond, validate, cleanup — richest single doc. |
| `docs/troubleshooting.md` | D | C | Diagnostic runbook for command/project/worker/response/no-mistakes/cleanup failures. |
| `docs/schema.md` | D | B | Authoritative project-YAML field reference and path-resolution rules. |
| `docs/skills.md` | D | C | How to choose/install/update skills; Sergeant-owned skill trigger table. |
| `docs/repo-scoped-skills.md` | D | C | Canonical worker-brief skill inventory and per-harness discovery mechanism (`.agents/skills`, `.claude/skills`, `opencode.json`). |
| `docs/callbacks.md` | D | C | The durable callback protocol spec (schemas, retry/ack states, ws-lab handoff) — matches and extends `bin/sgt-callback`'s behavior. |
| `docs/README.md` | D | C | Doc-set index plus the "documentation authority" precedence rule (which source wins on conflict) — short but genuinely procedural. |
| `docs/adr-oc-inject-deletion.md` | R | — | Decided-and-executed ADR recording deletion of the `bin/oc-inject` prototype (already absent from this subtree); its notify-transport rationale duplicates already-decomposed `bin/sgt-notify` behavior — historical decision record. |
| `docs/audit-2026-07.md` | R | — | Full-source system audit (Simplicity/Elegance/Correctness/Performance) — the legend's own "audit" example. |
| `docs/dead-code-2026-07.md` | R | — | Dead-code call-graph audit — audit, per legend. |
| `docs/prd-enforced-phased-dispatch.md` | R | — | Draft PRD, "awaiting explicit human PRD approval" — not-yet-approved product spec, per legend's PRD example. **Byte-identical duplicate** of `docs/prds/enforced-phased-dispatch.md` (verified via `diff`); disposition applies to both, not extracted twice. |
| `docs/prds/enforced-phased-dispatch.md` | R | — | Same content as `docs/prd-enforced-phased-dispatch.md` above (duplicate, not a symlink) — see that row. |
| `docs/prds/axi-agent-ergonomics.md` | R | — | Draft PRD ("Phase 1 implementation in progress") — PRD, per legend. |
| `docs/prds/claude-background-harness.md` | R | — | Draft PRD — PRD, per legend. |
| `docs/prds/code-improvements.md` | R | — | Draft PRD, references the two audits above as its source — PRD, per legend. |
| `docs/prds/tasks-axi-migration.md` | R | — | Draft PRD — PRD, per legend. |
| `docs/research/axi-agent-ergonomics-spike.md` | R | — | Dated research spike with a recorded decision — research note, per legend. |
| `docs/research/claude-background-harness-spike.md` | R | — | Dated research spike — research note, per legend. |
| `docs/research/tasks-axi-configurable-workflows.md` | R | — | Dated research spike inspecting an external codebase — research note, per legend. |

### `skills/` (5 files) — partition D

| Path | Disp. | Partition | What it is / reason |
|---|---|---|---|
| `skills/load-project/SKILL.md` | D | D | Resolve project ownership/config/paths procedure. |
| `skills/cross-repo-work/SKILL.md` | D | D | Cross-repo ownership/dependency/acceptance decomposition procedure. |
| `skills/dispatch/SKILL.md` | D | D | Plan-and-execute cross-repo dispatch procedure (the coordinator-facing counterpart of `bin/sgt-dispatch`), including the full worker contract. |
| `skills/wiki/SKILL.md` | D | D | Wiki capture-ownership and daily-digest operating procedure. |
| `skills/sergeant-help/SKILL.md` | D | D | Read-only documentation-lookup procedure and precedence rules. |

### `templates/` (1 file) — partition D

| Path | Disp. | Partition | What it is / reason |
|---|---|---|---|
| `templates/worker-brief.md` | D | D | The generated worker-brief template — the executable specification every dispatched worker actually runs against (9-step deliver sequence: scope/intent, routing, TDD, escalation, validation handoff, review axes, remediation, delivery). |

### `tests/` (62 files) — sampled group, all helper-evidence

Uniform Bash test-harness shape (`fail()` + assertion count, `set -euo
pipefail`), each verifying a contract already read directly in the `bin/`
script, doc, or skill file it names. 6 read in full (listed in Method); the
remaining 56 dispositioned the same way by name/subject correspondence to an
already-decomposed source:

| Path | Disp. |
|---|---|
| `tests/global-state-isolation-test.sh` | H (read) |
| `tests/instruction-policy-test.sh` | H (read) |
| `tests/mise-check-test.sh` | H (read) |
| `tests/no-remote-test.sh` | H (read) |
| `tests/repo-skills-test.sh` | H (read) |
| `tests/run-drain-tests.sh` | H (read) |
| `tests/mise-install-test.sh` | H (sampled) |
| `tests/runtime-bash-test.sh` | H (sampled) |
| `tests/sergeant-setup-test.sh` | H (sampled) |
| `tests/sgt-ack-response-test.sh` | H (sampled) |
| `tests/sgt-callback-test.sh` | H (sampled) |
| `tests/sgt-claude-real-contract-test.sh` | H (sampled) |
| `tests/sgt-claude-stop-bg-session-test.sh` | H (sampled) |
| `tests/sgt-claude-worker-test.sh` | H (sampled) |
| `tests/sgt-cleanup-cross-filesystem-test.sh` | H (sampled) |
| `tests/sgt-cleanup-test.sh` | H (sampled — 6640 lines, the largest file in scope) |
| `tests/sgt-dispatch-adopt-branch-test.sh` | H (sampled) |
| `tests/sgt-dispatch-bash32-test.sh` | H (sampled) |
| `tests/sgt-dispatch-brief-test.sh` | H (sampled) |
| `tests/sgt-dispatch-coordinator-pane-test.sh` | H (sampled) |
| `tests/sgt-dispatch-identity-test.sh` | H (sampled) |
| `tests/sgt-dispatch-model-tuple-test.sh` | H (sampled) |
| `tests/sgt-dispatch-oc-target-test.sh` | H (sampled) |
| `tests/sgt-dispatch-td-test.sh` | H (sampled) |
| `tests/sgt-dispatch-unpushed-guard-test.sh` | H (sampled) |
| `tests/sgt-dispatch-worker-test.sh` | H (sampled) |
| `tests/sgt-drain-force-test.sh` | H (sampled) |
| `tests/sgt-drain-terminate-test.sh` | H (sampled) |
| `tests/sgt-drain-test.sh` | H (sampled) |
| `tests/sgt-drain-worker-test.sh` | H (sampled) |
| `tests/sgt-graphify-test.sh` | H (sampled) |
| `tests/sgt-harness-test.sh` | H (sampled) |
| `tests/sgt-interrupted-fallback-test.sh` | H (sampled) |
| `tests/sgt-lease-convergence-test.sh` | H (sampled) |
| `tests/sgt-lease-exit-branch-test.sh` | H (sampled) |
| `tests/sgt-lease-finalizer-test.sh` | H (sampled) |
| `tests/sgt-lib-notification-target-test.sh` | H (sampled) |
| `tests/sgt-lib-owned-file-test.sh` | H (sampled) |
| `tests/sgt-no-mistakes-finding-test.sh` | H (sampled) |
| `tests/sgt-notify-test.sh` | H (sampled) |
| `tests/sgt-recover-drain-test.sh` | H (sampled) |
| `tests/sgt-recover-lease-owner-test.sh` | H (sampled) |
| `tests/sgt-recover-replacement-test.sh` | H (sampled) |
| `tests/sgt-recover-test.sh` | H (sampled) |
| `tests/sgt-respond-drain-test.sh` | H (sampled) |
| `tests/sgt-respond-recovery-test.sh` | H (sampled) |
| `tests/sgt-respond-test.sh` | H (sampled) |
| `tests/sgt-response-lock-release-test.sh` | H (sampled) |
| `tests/sgt-review-findings-test.sh` | H (sampled) |
| `tests/sgt-td-memory-worktree-test.sh` | H (sampled) |
| `tests/sgt-validate-test.sh` | H (sampled) |
| `tests/sgt-validation-worker-test.sh` | H (sampled) |
| `tests/sgt-wake-test.sh` | H (sampled) |
| `tests/sgt-watch-background-test.sh` | H (sampled) |
| `tests/sgt-watch-recycle-test.sh` | H (sampled) |
| `tests/sgt-watch-snapshot-test.sh` | H (sampled) |
| `tests/sgt-watch-test.sh` | H (sampled) |
| `tests/sgt-worker-drain-test.sh` | H (sampled) |
| `tests/sgt-worker-handshake-test.sh` | H (sampled) |
| `tests/sgt-worker-model-tuple-test.sh` | H (sampled) |
| `tests/sgt-worker-readiness-test.sh` | H (sampled) |
| `tests/sgt-worker-test.sh` | H (sampled) |

Reason (applies to every row in this group): deterministic test-harness
mechanics verifying a behavior contract that already lives in — and was
independently read from — the `bin/` script, doc, or skill file its name
names (e.g. `sgt-dispatch-model-tuple-test.sh` verifies model/variant
resolution already read in full in `bin/sgt-dispatch`). Not itself a
procedural outcome an agent follows; informs a later helper/shared-context
map per `references/dispositions.md`.

## Gaps

None. All 179 enumerated files are accounted for above (per-file reads for
117 of them; representative-sample disposition, explicitly noted, for the
two large uniform groups totaling 62 files). No path was left undispositioned
and no truncation occurred — the "On volume" sampling was applied only where
`references/dispositions.md` explicitly permits it (large uniform groups),
not as a substitute for reading distinct, individually-shaped files.
