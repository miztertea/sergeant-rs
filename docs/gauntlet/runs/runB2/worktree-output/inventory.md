# Source inventory — repo-to-icm runB2 (bounded measurement run)

Layer 4 artifact of `10-inventory`, disposition `promote`. Produced against
`../00-contract/output/contract.md` (which does **not** open with
`# AMBIGUOUS — NOT RESOLVED`; the fail-closed propagation of
`../_config/run-discipline.md` §2 was checked and does not apply).

## Enumeration method

Per `contract.md` §1 the subject is a **vendored subtree**
(`reference/sergeant-upstream/`, pinned by record at upstream SHA
`f430cfd4f90174a98adbd7abebbece6303817929` via `reference/UPSTREAM.md`), not a
live checkout — so the working tree itself is the pinned snapshot and was
enumerated with a recursive file listing (`find <path> -type f`), **not** with
git tooling inside the subtree (which would silently answer about the outer
repository).

The listing was restricted to exactly the contract §2 scope paths:

1. `reference/sergeant-upstream/AGENTS.md`
2. `reference/sergeant-upstream/README.md`
3. `reference/sergeant-upstream/bin/` (recursive)

Because the enumeration command named only these paths, it structurally could
not cross into `reference-corpus/` — the blindness boundary of
`../_config/run-discipline.md` §1 was honored during enumeration, not checked
after. No path under `reference-corpus/` was opened, listed, or searched for
at any point in this stage.

A symlink check (`find … -type l`) found **no symlinks** in scope; the
symlink/duplicate-tree rule in `references/dispositions.md` has nothing to
apply to this run.

**Excluded during enumeration, per contract §3.3 (generated output):**
`reference/sergeant-upstream/bin/__pycache__/sgt-callbackcpython-312.pyc`
(1 file, compiled Python bytecode cache of `bin/sgt-callback`). It is recorded
here for auditability but is out of scope by contract and is not counted in
the totals below.

**Total files enumerated in scope: 38** (2 root files + 36 files in `bin/`).

## Reading depth (stated per `references/dispositions.md`)

Every file was opened before its disposition was assigned. Read in full:
`AGENTS.md`, `README.md`, `bin/_sgt-bash-version.sh`, `bin/_sgt-intent.sh`.
The remaining 34 shell/Python scripts were sampled: the full leading
doc-comment block plus usage/contract text (20–30 lines each) — this
repository's scripts carry unusually complete header contracts (purpose,
usage, invariants, protocol notes), and the sampled portion is what each
row's description and reason rest on. This sampling is safe in the
`decompose` direction (the asymmetry rule): every sampled file is a
behavior-bearing entry point or a canonical protocol/format definition, and
`decompose` is robust to unread body detail. No file was assigned
`helper-evidence` on a sample alone — the single `helper-evidence` row was
read in full.

## Inventory

Order: directory-listing order within each contract scope (root files first,
then `bin/` alphabetically).

### Contract scope 1 — root operating instructions

| # | Path (under `reference/sergeant-upstream/`) | What it is | Disposition | Partition | Reason |
|---|---|---|---|---|---|
| 1 | `AGENTS.md` | Coordinator operating instructions: coordinator-vs-direct execution modes, toolbelt command table, skill trigger table, 9-step standard task workflow, no-op-outcome prohibitions, conventions (model pinning, harness selection, intent revision discipline) | decompose | `root-operating-instructions` | Dense procedural directives an agent follows and decides by; the repo's primary behavior surface |
| 2 | `README.md` | Project README: mental model and quick start, plus substantial normative procedure — `sgt-watch --snapshot` semantics, model-pinning transport contract, no-mistakes shipping-gate procedure (start/gates/finish/routing), independent-review axis and severity routing rules, drain locking prerequisites | decompose | `root-operating-instructions` | Well over half the file is procedural contract, not marketing; behavior someone follows or decides by |

### Contract scope 2 — the `bin/` fleet-dispatch partition

| # | Path (under `reference/sergeant-upstream/bin/`) | What it is | Disposition | Partition | Reason |
|---|---|---|---|---|---|
| 3 | `_sgt-bash-version.sh` | Shared minimum-Bash-version check (requires Bash ≥ 3.2) sourced by every entry point | helper-evidence | — | Pure deterministic gate mechanism (read in full, 23 lines); the requirement it enforces is stated as behavior in `README.md` (row 2), so no procedural outcome lives only here |
| 4 | `_sgt-drain.sh` | Sourced library defining the drain protocol: drain state file locations and key=value format, admission-check locking (atomic hard-link), lock records, process-liveness checks, Claude background-session stop helpers | decompose | `bin-drain` | Canonical definition of drain state format and admission semantics — protocol facts other commands and operators decide by, not mere mechanism |
| 5 | `_sgt-harness.sh` | Sourced registry: the single definition of every accepted interactive harness (gate + readiness probe + launch args per row), with the `tui` readiness contract and registry validation | decompose | `bin-dispatch-worker-lifecycle` | Explicitly the one canonical policy source for which harnesses are accepted and what "ready" means; behavior-bearing by design (created to stop three-way drift) |
| 6 | `_sgt-intent.sh` | Sourced library for canonical intent: sha256 intent revision, intent transport resolution (private `--intent-file` vs consent-gated argv), the required eight-section intent document format in exact order, safety-sensitive-objective keyword gate forcing `--intent-file`, path/size/control-char validation | decompose | `bin-validation-review` | Read in full; carries real policy (intent format contract, safety gate, transport privacy rules) — procedural outcomes, not just hashing mechanics |
| 7 | `_sgt-lib.sh` | Shared library sourced by all `sgt-*` scripts: `SERGEANT_CONFIG`/`FLEET_DIR` defaults, agent detection order (`SERGEANT_AGENT` → OpenCode env → …), `_die`/`_require_*` helpers, path resolution | decompose | `bin-dispatch-worker-lifecycle` | Defines fleet-wide conventions (config/fleet locations, harness detection precedence) that other behavior depends on; per the asymmetry rule, extracted rather than risk-missed. Sourced by every entry point; partitioned here where its conventions chiefly bind |
| 8 | `_sgt-response-lock.sh` | Sourced library: the single definition of the consumed-response archive format (`body`, `gate_generation`, `applied_status`, `proof`) and its serialization/parsing helpers | decompose | `bin-fleet-supervision-response` | Canonical cross-command format contract (published by `sgt-ack-response`, validated by `sgt-cleanup`) — a protocol fact, not mere mechanism |
| 9 | `_sgt-review-axes.sh` | Sourced vocabulary: canonical independent-review axes (`standards spec readiness` + conditional `accessibility`) and severity set driving both dispatch briefs and findings routing | decompose | `bin-validation-review` | The normative axis/severity definition `README.md` cites by name; one definition both halves of the review contract decide by |
| 10 | `sgt-ack-response` | CLI: acknowledge and clear one consumed worker response for a task/repo, publishing the four-field response archive | decompose | `bin-fleet-supervision-response` | Behavior-bearing operator/coordinator command in the response lifecycle |
| 11 | `sgt-callback` | Python CLI: durable, profile-bound callback events for fleet tasks (register, enqueue, drain, verify) with locked JSON state | decompose | `bin-fleet-supervision-response` | Behavior-bearing command; the callback event protocol is a procedural surface |
| 12 | `sgt-cleanup` | CLI: remove worktrees and fleet state for a completed task (treehouse return or `git worktree remove`); validates task-id against traversal/symlink aliases; preserves and replays terminal evidence atomically | decompose | `bin-dispatch-worker-lifecycle` | Behavior-bearing lifecycle command with stated safety invariants |
| 13 | `sgt-context` | CLI: emit a structured agent context block for a project, resolving instruction layering defaults → group → repo | decompose | `bin-project-registry-context` | Behavior-bearing command; defines the instruction-layering resolution order |
| 14 | `sgt-dag-dispatch-hook` | CLI: stage hook called by dagr when a stage becomes ready; wraps `sgt-dispatch` and writes dagr tracking files into fleet state so `sgt-watch` can auto-advance the DAG | decompose | `bin-dag` | Behavior-bearing integration procedure between dagr and the fleet |
| 15 | `sgt-dag-run` | CLI: create and start a dagr run from a project YAML `dag:` block and dispatch initially ready stages | decompose | `bin-dag` | Behavior-bearing command; defines the project-YAML DAG block format |
| 16 | `sgt-dispatch` | CLI (1005 lines): dispatch subagents across a project's repos — worktree per repo, mission brief, tmux worker spawn; options for td-sourced briefs, dependency ordering, harness/model pinning, coordinator-pane binding, intent files, callback correlation | decompose | `bin-dispatch-worker-lifecycle` | The central dispatch procedure of the fleet; heavily behavior-bearing |
| 17 | `sgt-drain` | CLI: set/remove/show a persistent cooperative drain on a project or globally, with `--wait`/`--timeout` semantics and generation-safe response storage while drained | decompose | `bin-drain` | Behavior-bearing operator command defining drain semantics |
| 18 | `sgt-drain-force` | CLI: force-stop workers that failed cooperative drain; requires an active drain, displays exact identity, never runs automatically (`--yes` or `--dry-run` required) | decompose | `bin-drain` | Behavior-bearing command with explicit safety gating |
| 19 | `sgt-graphify` | CLI: run graphify across all repos in a project and publish the merged graph atomically (readers never observe partial state) | decompose | `bin-project-registry-context` | Behavior-bearing command with a stated atomic-publication invariant |
| 20 | `sgt-interactive-worker` | CLI (1136 lines): own one persistent interactive agent pane — harness launch via the registry, readiness probing, drain cooperation, response consumption | decompose | `bin-dispatch-worker-lifecycle` | The worker side of the dispatch contract; heavily behavior-bearing |
| 21 | `sgt-list` | CLI: list all known projects from `~/.config/sergeant/` (excluding `config.yaml`) | decompose | `bin-project-registry-context` | Behavior-bearing (small) registry command in the documented toolbelt |
| 22 | `sgt-no-mistakes-finding` | CLI: apply a disposition (`gate`/`td`/`ignore`/`ask-user`) to one no-mistakes finding, creating/updating owning-repo td work | decompose | `bin-validation-review` | Behavior-bearing findings-routing command; disposition semantics are policy |
| 23 | `sgt-notify` | CLI: inject a worker update into the primary session — durable metadata-only wake marker by default, raw tmux injection as explicit compatibility option | decompose | `bin-fleet-supervision-response` | Behavior-bearing escalation/outcome transport with a stated message convention |
| 24 | `sgt-recover` | CLI: one bounded stall recovery for a stalled in-progress worker — gated on stall proof, replacement-before-kill ordering, second invocation escalates to `needs_input` | decompose | `bin-dispatch-worker-lifecycle` | Behavior-bearing recovery procedure with explicit invariants |
| 25 | `sgt-respond` | CLI: deliver a response (from stdin, via a private tempfile) and resume a dead waiting worker; refuses in-progress workers | decompose | `bin-fleet-supervision-response` | Behavior-bearing response-delivery procedure with privacy handling |
| 26 | `sgt-review-findings` | CLI (744 lines): route structured independent-review findings to td — axis/severity normalization, per-repo dedup scope, revision digests with superseded-revision preservation and `needs-reconciliation` labeling, sanitized retry artifacts | decompose | `bin-validation-review` | Behavior-bearing router; its normalization and preservation rules are policy |
| 27 | `sgt-status` | CLI: git status across every repo in a project | decompose | `bin-project-registry-context` | Behavior-bearing (small) toolbelt command |
| 28 | `sgt-sync` | CLI: clone missing repos and pull existing ones for a project | decompose | `bin-project-registry-context` | Behavior-bearing (small) toolbelt command |
| 29 | `sgt-td-create` | CLI: create td tasks across project repos for a cross-repo brief, all-or-nothing with rollback | decompose | `bin-td-integration` | Behavior-bearing command with a stated atomicity contract |
| 30 | `sgt-td-list` | CLI: unified td task view across all repos in a project, with status/priority/repo filters and `--json` | decompose | `bin-td-integration` | Behavior-bearing toolbelt command |
| 31 | `sgt-td-memory` | CLI: record non-secret worker recovery pointers (handoff/response) in td; silently exits when no td task is bound | decompose | `bin-td-integration` | Behavior-bearing recovery-evidence procedure |
| 32 | `sgt-treehouse-init` | CLI: run `treehouse init` in each project repo lacking `treehouse.toml`, optionally filtered by group | decompose | `bin-project-registry-context` | Behavior-bearing setup command that changes subsequent dispatch behavior |
| 33 | `sgt-undrain` | CLI: remove a drain record for a project or globally; idempotent | decompose | `bin-drain` | Behavior-bearing operator command in the drain lifecycle |
| 34 | `sgt-validate` | CLI (1003 lines): launch coordinator-owned no-mistakes beside an interactive worker; ownership claim/release, default `--skip review,document` profile, argv-intent consent gating | decompose | `bin-validation-review` | The validation-boundary procedure; heavily behavior-bearing |
| 35 | `sgt-validation-worker` | CLI: run the coordinator-owned no-mistakes validation in an interactive pane against an expected intent revision and transport | decompose | `bin-validation-review` | Behavior-bearing worker side of the validation boundary |
| 36 | `sgt-wake` | CLI: evaluate a durable `.sergeant-wake-condition` (not_before, github_check, fleet/td dependency, deployment, human_response) with strict field allowlists, deadlines, attempt limits, and backoff; resumes via `sgt-respond` | decompose | `bin-fleet-supervision-response` | Behavior-bearing wake protocol; the condition format and allowlists are policy |
| 37 | `sgt-watch` | CLI (738 lines): monitor a dispatched fleet — foreground watch, `--background` managed monitor, `--sync-all` reconciliation, `--list`, and the strictly read-only `--snapshot` JSON observation | decompose | `bin-fleet-supervision-response` | The fleet-supervision procedure; its mode semantics are policy `README.md` documents at length |
| 38 | `wiki-daily-digest` | CLI: synthesize a daily wiki session digest from opencode/goose/claude session history plus PR/td enrichment, via the Anthropic API, into `~/wiki/sessions/YYYY-MM-DD.md` | decompose | `bin-wiki` | Behavior-bearing standalone pipeline with dates, sources, and backfill semantics |

No file in scope qualified for `obsolete-candidate` (no row could cite a
specific settled fact replacing a mechanism — per the legend, absent a
citation the row stays `decompose`) or for `reference-only` (the only
non-procedural candidate in scope, the `__pycache__` bytecode file, is
already excluded by contract §3.3 before disposition; `LICENSE` and similar
material sit outside contract scope entirely).

## Partition summary (`decompose` rows)

The contract names two top-level scopes; per `references/dispositions.md` the
36-file `bin/` scope is subdivided into coherent sub-partitions (each
summarizable in one sentence), all prefixed `bin-` to keep the mapping to
contract scope 2 explicit. Suggested harvest order as listed.

| Partition | One-sentence summary | Members | Count |
|---|---|---|---|
| `root-operating-instructions` | The repo's root agent-facing operating instructions and normative README procedure (contract scope 1) | rows 1–2 | 2 |
| `bin-dispatch-worker-lifecycle` | Dispatching workers into worktrees and owning their lifecycle: dispatch, the interactive worker, the harness registry, shared fleet conventions, recovery, cleanup | rows 5, 7, 12, 16, 20, 24 | 6 |
| `bin-fleet-supervision-response` | Watching a dispatched fleet and moving responses/wake events through it: watch, respond, ack, wake, notify, callbacks, and the response archive format | rows 8, 10, 11, 23, 25, 36, 37 | 7 |
| `bin-validation-review` | The validation boundary and review routing: intent contract, sgt-validate and its worker, no-mistakes findings dispositions, review axes, and the findings router | rows 6, 9, 22, 26, 34, 35 | 6 |
| `bin-drain` | Cooperative drain of the fleet: the drain protocol library and the drain/undrain/force-stop commands | rows 4, 17, 18, 33 | 4 |
| `bin-project-registry-context` | Project registry and per-project operations: list, status, sync, context emission, graphify publication, treehouse init | rows 13, 19, 21, 27, 28, 32 | 6 |
| `bin-td-integration` | Cross-repo td task integration: create, list, and recovery-pointer recording | rows 29, 30, 31 | 3 |
| `bin-dag` | dagr DAG execution over the fleet: run creation and the stage dispatch hook | rows 14, 15 | 2 |
| `bin-wiki` | The standalone daily wiki digest pipeline | row 38 | 1 |

Partition total: 2 + 6 + 7 + 6 + 4 + 6 + 3 + 2 + 1 = **37** ✓ (equals the
`decompose` count).

## Count reconciliation

| Disposition | Count |
|---|---|
| decompose | 37 |
| helper-evidence | 1 |
| obsolete-candidate | 0 |
| reference-only | 0 |
| **Total** | **38** |

38 dispositioned = 38 enumerated ✓. Every `decompose` row appears in exactly
one partition (37 = 37 ✓). No path in scope was left unreached; the
one-actor-turn volume limit was not hit, so no truncation gap is recorded and
the "every file in scope" durable outcome is fully met this run.
