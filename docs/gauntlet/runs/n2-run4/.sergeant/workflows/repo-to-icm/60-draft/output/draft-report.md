# 60-draft — draft report (N2 run, 21-partition corpus)

`../../50-synthesize/output/candidates.md` does not open with `# AMBIGUOUS — NOT RESOLVED`, so this stage proceeded with its ordinary work (`../../_config/run-discipline.md` §2 checked, not triggered).

This run's corpus (21 harvested partitions, 1333 classification records, 44 workflow candidates) is larger than any prior run of this stage (run 3 stopped at 6 partitions). Every one of the 44 workflow candidates from `../../50-synthesize/output/candidates.md` Bucket 1–3 was materialized below — nothing was truncated, sampled, or dropped for volume. Where the method itself licenses a bound (design-inference stages for candidates with no `stage`-rung member — see §2 below), that bound is recorded per-candidate in each package's own `provenance.md`, not applied silently.

## 1. Manifest — materialized draft workflow packages

**44** candidates from `../../50-synthesize/output/candidates.md` Bucket 1–3, all materialized under `.sergeant/drafts/workflows/` in this run's own worktree (never `.sergeant/workflows/`), each matching `../references/draft-package-template.md`: `index.md` (`status: draft`), `workflow.toml`, `CONTEXT.md`, `provenance.md`, and one `NN-<stage-name>/` per member stage with its own `CONTEXT.md` and (empty, templated) `output/README.md`. Name-collision check (step 1 of this stage's method) ran against `.sergeant/workflows/` (only `repo-to-icm` exists there), against every other candidate name in this same run, and against `repo-to-icm` itself — no collision found, nothing renamed.

| Candidate | Path | Stages | Ordering | Notes |
|---|---|---|---|---|
| `adopt-external-skill` | `.sergeant/drafts/workflows/adopt-external-skill/` | 1 | linear | single inferred stage (no `stage`-rung member) |
| `callback-protocol` | `.sergeant/drafts/workflows/callback-protocol/` | 7 | graph-shaped |  |
| `check-repo-status` | `.sergeant/drafts/workflows/check-repo-status/` | 1 | linear | single inferred stage (no `stage`-rung member) |
| `ci-verification` | `.sergeant/drafts/workflows/ci-verification/` | 1 | linear |  |
| `code-review` | `.sergeant/drafts/workflows/code-review/` | 3 | linear |  |
| `cross-repo-work` | `.sergeant/drafts/workflows/cross-repo-work/` | 4 | linear |  |
| `dag-run` | `.sergeant/drafts/workflows/dag-run/` | 4 | linear |  |
| `design-it-twice` | `.sergeant/drafts/workflows/design-it-twice/` | 3 | linear |  |
| `diagnose-bug` | `.sergeant/drafts/workflows/diagnose-bug/` | 5 | linear |  |
| `direct-mode` | `.sergeant/drafts/workflows/direct-mode/` | 4 | linear |  |
| `dispatch-mode` | `.sergeant/drafts/workflows/dispatch-mode/` | 19 | graph-shaped |  |
| `domain-modeling` | `.sergeant/drafts/workflows/domain-modeling/` | 2 | linear |  |
| `fleet-status-listing` | `.sergeant/drafts/workflows/fleet-status-listing/` | 1 | linear | single inferred stage (no `stage`-rung member) |
| `graphify` | `.sergeant/drafts/workflows/graphify/` | 2 | linear |  |
| `grilling` | `.sergeant/drafts/workflows/grilling/` | 1 | linear |  |
| `implement` | `.sergeant/drafts/workflows/implement/` | 3 | linear |  |
| `install-sergeant` | `.sergeant/drafts/workflows/install-sergeant/` | 4 | linear |  |
| `invoke-grill-with-docs` | `.sergeant/drafts/workflows/invoke-grill-with-docs/` | 1 | linear | single inferred stage (no `stage`-rung member) |
| `list-projects` | `.sergeant/drafts/workflows/list-projects/` | 1 | linear | single inferred stage (no `stage`-rung member) |
| `list-tasks` | `.sergeant/drafts/workflows/list-tasks/` | 1 | linear | single inferred stage (no `stage`-rung member) |
| `load-project` | `.sergeant/drafts/workflows/load-project/` | 3 | linear |  |
| `no-mistakes-finding-routing` | `.sergeant/drafts/workflows/no-mistakes-finding-routing/` | 2 | linear |  |
| `notify-primary-session` | `.sergeant/drafts/workflows/notify-primary-session/` | 2 | linear |  |
| `prototype` | `.sergeant/drafts/workflows/prototype/` | 7 | linear |  |
| `record-recovery-pointer` | `.sergeant/drafts/workflows/record-recovery-pointer/` | 1 | linear |  |
| `register-project` | `.sergeant/drafts/workflows/register-project/` | 1 | linear | single inferred stage (no `stage`-rung member) |
| `research` | `.sergeant/drafts/workflows/research/` | 1 | linear |  |
| `resolve-merge-conflict` | `.sergeant/drafts/workflows/resolve-merge-conflict/` | 3 | linear |  |
| `review-findings-routing` | `.sergeant/drafts/workflows/review-findings-routing/` | 3 | graph-shaped |  |
| `sergeant-help` | `.sergeant/drafts/workflows/sergeant-help/` | 2 | linear |  |
| `sergeant-setup` | `.sergeant/drafts/workflows/sergeant-setup/` | 10 | linear |  |
| `standard-workflow` | `.sergeant/drafts/workflows/standard-workflow/` | 8 | linear |  |
| `sync-project-repos` | `.sergeant/drafts/workflows/sync-project-repos/` | 2 | linear |  |
| `tdd` | `.sergeant/drafts/workflows/tdd/` | 2 | linear |  |
| `to-spec` | `.sergeant/drafts/workflows/to-spec/` | 3 | linear |  |
| `to-tickets` | `.sergeant/drafts/workflows/to-tickets/` | 6 | linear |  |
| `treehouse-init` | `.sergeant/drafts/workflows/treehouse-init/` | 1 | linear | single inferred stage (no `stage`-rung member) |
| `triage` | `.sergeant/drafts/workflows/triage/` | 8 | graph-shaped |  |
| `troubleshoot-failure` | `.sergeant/drafts/workflows/troubleshoot-failure/` | 1 | linear |  |
| `validation-pipeline-gate` | `.sergeant/drafts/workflows/validation-pipeline-gate/` | 14 | graph-shaped |  |
| `wayfinder` | `.sergeant/drafts/workflows/wayfinder/` | 4 | linear |  |
| `wiki-maintenance` | `.sergeant/drafts/workflows/wiki-maintenance/` | 2 | linear |  |
| `worker-contract` | `.sergeant/drafts/workflows/worker-contract/` | 2 | linear |  |
| `worker-lifecycle` | `.sergeant/drafts/workflows/worker-lifecycle/` | 26 | graph-shaped |  |

**Totals:** 44 packages, 182 stage directories (174 directly evidenced by a `stage`-rung classification record + 8 single-stage design inferences — see §2). Of the 44: 36 have at least one directly-evidenced `stage`-rung member; 8 do not (5 `stage`/`stage-context`/`helper`-only clusters with no `stage`-rung member, plus the 3 standalone single-behavior candidates) and were given exactly one design-inference stage each, per §2.

## 2. How the volume was bounded, and what was not truncated

This stage's method (`CONTEXT.md` "How to do it" and `../references/draft-package-template.md`) requires **every** workflow candidate from `50-synthesize` to be materialized — it does not license skipping, sampling, or capping the candidate set itself, so none of that happened: all 44 candidates got full packages, regardless of the corpus being 3.5x larger (21 vs. 6 partitions) than the prior run.

The one place a bound was genuinely needed, and *is* licensed by the template, is candidates with **no** `stage`-rung member at all: 5 workflow-rung clusters (`check-repo-status`, `fleet-status-listing`, `list-projects`, `list-tasks`, `treehouse-init`) whose `../../50-synthesize/output/candidates.md` entry literally reads "(no stage candidates for this workflow value)", plus 3 standalone single-behavior candidates (`adopt-external-skill`, `invoke-grill-with-docs`, `register-project`) that have no member stage by construction. The template requires every candidate package to have at least one `NN-<stage-name>/` directory (a runnable workflow needs a checkpoint to execute), but these 8 have zero directly-evidenced ones. Per `../references/draft-package-template.md`'s own instruction ("a stage or workflow candidate with no source evidence is either a justified design inference ... or unsupported invention"), each of these 8 got **exactly one** design-inference stage rather than being padded with invented multi-stage structure or left without a runnable checkpoint — recorded plainly as such in that package's own `provenance.md`, never given an invented citation. This is the only bound applied in this stage, it is per-candidate (not a corpus-wide sampling), and it is the same shape for all 8 candidates it applies to (one stage, sourced from that candidate's own workflow-level helpers or sole behavior — never a number picked to save effort).

## 3. Carried-through candidate lists (verbatim, per `../../50-synthesize/output/candidates.md`)

This stage does not edit these — copied as-is from `../../50-synthesize/output/candidates.md` for `90-reconcile`'s use, per this stage's own `CONTEXT.md` step 6.

### 3.1 Permanent-instruction candidates (`agents-invariant`) — Bucket 4

## Bucket 4: Permanent-instruction candidates (`agents-invariant`)

**126** records. Listed, not drafted into any workflow package — per
synthesis-method.md bucket 4 and the ladder's own framing, `AGENTS.md`
changes are the promotion reviewer's call, not this run's.

- `BU-0001` (`AGENTS.md (AGENTS.md L3-5)`): Before acting on a project, resolve its repositories, roles, inherited instructions, and configured paths with the project context-resolution step.
- `BU-0002` (`AGENTS.md (AGENTS.md L3-5)`): Ownership of a project is never inferred from the current working directory.
- `BU-0003` (`AGENTS.md (AGENTS.md L11-13)`): The primary Sergeant session coordinates multi-repository work by default rather than implementing directly.
- `BU-0004` (`AGENTS.md (AGENTS.md L11-13)`): Direct implementation in the primary session is permitted only when the user explicitly asks to work in-session (or says not to dispatch) and one repository owns the complete outcome.
- `BU-0005` (`AGENTS.md (AGENTS.md L15-20)`): Dispatch mode is used when work spans repositories, contains two or more independent repository-owned tasks, needs an isolated independent review worker, or the user asks for workers.
- `BU-0009` (`AGENTS.md (AGENTS.md L22-36)`): Direct mode is used only when the user explicitly requests it and the work has one clear owning repository.
- `BU-0017` (`AGENTS.md (AGENTS.md L38-41)`): The coordinator role is never used as a reason to stop at a plan, status report, or dispatch suggestion when the user asked for an implemented outcome.
- `BU-0018` (`AGENTS.md (AGENTS.md L38-41)`): Direct mode is never used to edit several repositories in one checkout, or to bypass repository instructions, task ownership, review independence, or shipping gates.
- `BU-0019` (`AGENTS.md (AGENTS.md L60-61)`): When a toolbelt command covers an operation, it is used instead of reproducing the operation with ad hoc shell commands.
- `BU-0020` (`AGENTS.md (AGENTS.md L99-103)`): The bare `sgt-*` command name is used when it resolves on PATH; otherwise the matching script is run from this repository's `bin/` directory.
- `BU-0021` (`AGENTS.md (AGENTS.md L99-103)`): Manual fallback operations are used only when no toolbelt command covers the operation, or the command returns an explicit unsupported-case error; the fallback and the original error evidence are reported.
- `BU-0022` (`AGENTS.md (AGENTS.md L120-124)`): For every listed procedural-skill trigger, the repository-local SKILL.md file is read directly; it is canonical and takes precedence over any same-named registry skill.
- `BU-0023` (`AGENTS.md (AGENTS.md L125-126)`): A harness registry's omission of a skill does not make the skill unavailable, and the owner is not asked or the task stopped solely because the registry omits it.
- `BU-0024` (`AGENTS.md (AGENTS.md L127-128)`): The session stops and reports the exact repository-local skill path only when that file is absent or unreadable, and does not reconstruct a partial protocol from memory in that case.
- `BU-0036` (`AGENTS.md (AGENTS.md L148)`): `in_progress`, `needs_input`, `blocked`, and `waiting` are treated as nonterminal worker states; a waiting worker may remain alive or may exit after a durable handoff.
- `BU-0037` (`AGENTS.md (AGENTS.md L148)`): Deferred waits are resumed through the wake-condition step when a durable `.sergeant-wake-condition` has been published; human decisions are resumed through the worker response-delivery step.
- `BU-0038` (`AGENTS.md (AGENTS.md L148)`): Progress is never inferred from liveness alone, an expected blocked exit is never rewritten as orphaned, and a waiting worktree is never cleaned.
- `BU-0039` (`AGENTS.md (AGENTS.md L148)`): The worker response-delivery step, the wake-condition step, or supported recovery are used only after reconciling status, response generation, pane identity, and handoff evidence.
- `BU-0040` (`AGENTS.md (AGENTS.md L150-153)`): Every dispatched implementation, independent review, PR description, successor, recovery, and final shipping gate must use the same canonical intent revision from `.sergeant-intent.md`.
- `BU-0041` (`AGENTS.md (AGENTS.md L150-153)`): Workers and remediation loops never run the validation pipeline themselves.
- `BU-0044` (`AGENTS.md (AGENTS.md L161-162)`): A plan, task, finding, or worker launch is not treated as the requested outcome unless the user asked only for planning or dispatch.
- `BU-0045` (`AGENTS.md (AGENTS.md L163-164)`): A known blocker is not repeatedly reported once its decision and remediation path are approved; the next safe step is executed instead.
- `BU-0046` (`AGENTS.md (AGENTS.md L165-166)`): Duplicate tasks, findings, PRs, workers, or review passes are not created when a canonical preserved owner already exists.
- `BU-0047` (`AGENTS.md (AGENTS.md L167-168)`): A worker is not called active solely because its process or pane exists; recent meaningful progress evidence is required.
- `BU-0048` (`AGENTS.md (AGENTS.md L169-170)`): A completed, merged, blocked, or abandoned task is never left recorded as `in_progress`; the task tracker and fleet state are reconciled truthfully.
- `BU-0049` (`AGENTS.md (AGENTS.md L171-172)`): Tool absence produces an actionable fallback or explicit blocker, never a silent skip, false success, or indefinite wait.
- `BU-0050` (`AGENTS.md (AGENTS.md L173-175)`): Standing authorization may remove repetitive dispatch confirmation, but never authorizes risk acceptance, gate skipping, force operations, secret exposure, or destruction of preserved state.
- `BU-0054` (`AGENTS.md (AGENTS.md L182)`): Repositories under `~/.config/sergeant/` are never modified — that location is config, not code.
- `BU-0055` (`AGENTS.md (AGENTS.md L183)`): Secrets are never committed; project YAMLs may contain paths but must not contain credentials.
- `BU-0056` (`AGENTS.md (AGENTS.md L184-185)`): A bare `sgt-*` command is used when `command -v <name>` succeeds; otherwise the equivalent `bin/<name>` from this repository is run.
- `BU-0061` (`README.md (README.md L145-149)`): The interactive fleet-watch loop and `--sync-all` reconcile lifecycle state and may kill panes, so neither is safe for a coordinator or bridge that only wants to observe.
- `BU-0065` (`README.md (README.md L172-178)`): The dispatch step/`SERGEANT_AGENT` selects the harness executable and the dispatch step/`SERGEANT_MODEL` pins what that harness runs, and the two are orthogonal, with model precedence `--model` > `SERGEANT_MODEL` > the harness's own ambient default.
- `BU-0078` (`README.md (README.md L257-260)`): The managed coordinator pane is reused across dispatches and runs a reader that displays each line it receives and never executes it, so a tmux-injected notification can never become a shell command in the coordinator's pane.
- `BU-0106` (`docs/README.md (docs/README.md L28-36)`): Documentation authority is layered by ownership: `AGENTS.md` owns always-on agent execution/safety policy, `skills/*/SKILL.md` and `.agents/skills/*/SKILL.md` own trigger-specific procedures, `docs/schema.md` owns project configuration fields and path resolution, and the rest of this documentation set owns user installation/operating instructions.
- `BU-0107` (`docs/README.md (docs/README.md L34-36)`): Command `--help` output wins when the command implements it; otherwise the command's emitted usage/error contract and its tests win, and a task is filed when prose disagrees with released behavior.
- `BU-0108` (`docs/README.md (docs/README.md L38-39)`): Documentation examples must not contain real credentials, private repository names, prompt bodies, response bodies, or secret-bearing environment values.
- `BU-0109` (`docs/what-is-sergeant.md (docs/what-is-sergeant.md L10-12)`): Sergeant is designed for one developer per installation; adoption by a larger organization means each developer installs Sergeant independently — it does not turn one installation into a shared team service.
- `BU-0110` (`docs/what-is-sergeant.md (docs/what-is-sergeant.md L23-24)`): Sergeant does not provide central tenancy, organization RBAC, shared credentials, cross-machine worker leases, or a team-wide fleet database.
- `BU-0111` (`docs/what-is-sergeant.md (docs/what-is-sergeant.md L51-52)`): A worker is an agent running in an isolated worktree and tmux pane; a live process is not proof of progress, and recent meaningful progress evidence is required.
- `BU-0112` (`docs/what-is-sergeant.md (docs/what-is-sergeant.md L54-58)`): A decision request is a `needs_input`, `blocked`, or validation ask-user gate that requires a human product, security, privacy, destructive-action, or risk decision; mechanical findings are not human decision requests.
- `BU-0113` (`docs/what-is-sergeant.md (docs/what-is-sergeant.md L64-66)`): Direct mode still requires a task, TDD, repository-native checks, independent review, shipping validation, and handoff even though it runs in the current session.
- `BU-0114` (`docs/what-is-sergeant.md (docs/what-is-sergeant.md L78)`): Sergeant is not permission to push directly to default branches.
- `BU-0115` (`docs/what-is-sergeant.md (docs/what-is-sergeant.md L79-82)`): Sergeant does not make a worker healthy merely because its process exists, does not treat a plan, task, worker launch, or finding as delivered work, and does not authorize validation agents to modify source while reporting findings.
- `BU-0116` (`docs/skills.md (docs/skills.md L19-20)`): Skill provenance is never inferred from a folder name; `.skill-lock.json`, a package lock, plugin metadata, or the source repository is checked instead.
- `BU-0117` (`docs/skills.md (docs/skills.md L55-57)`): The Claude plugin route installs a managed read-only bundle; plugin-owned files are never edited, since updates are not expected to preserve those edits.
- `BU-0118` (`docs/skills.md (docs/skills.md L119-122)`): Every directive in a Sergeant-owned skill must contain a trigger, action, prohibition, observable evidence, or stop condition; slogans such as 'be thorough' are replaced with commands, failure behavior, acceptance criteria, ownership, or review evidence.
- `BU-0120` (`docs/skills.md (docs/skills.md L142-144)`): Sergeant-owned skills are updated through this repository via a reviewed PR and by running `bash tests/instruction-policy-test.sh` plus the full Sergeant test suite.
- `BU-0121` (`docs/repo-scoped-skills.md (docs/repo-scoped-skills.md L4-10)`): `.agents/skills/` is the canonical Agent Skills tree discovered directly by Codex; OpenCode discovers the same tree through `opencode.json`; Claude discovers it through repository-local links in `.claude/skills/`, which resolve only to `.agents/skills/` — no install step writes to a user's global agent configuration.
- `BU-0122` (`docs/repo-scoped-skills.md (docs/repo-scoped-skills.md L38-40)`): Workers are instructed never to invoke the validation pipeline directly; the validation pipeline skill is vendored only so workers can load and understand the coordinator-owned shipping gate contract when a brief references it.
- `BU-0130` (`docs/getting-started.md (docs/getting-started.md L82-83)`): Sergeant does not install harness-specific conversation-injection plugins; worker updates are surfaced from durable fleet state through the interactive fleet-watch loop.
- `BU-0172` (`docs/troubleshooting.md (docs/troubleshooting.md L3-4)`): Supported Sergeant commands are used before manual process, tmux, Git, or fleet-file operations, and exact errors and state are preserved before recovery.
- `BU-0183` (`docs/troubleshooting.md (docs/troubleshooting.md L144-146)`): Parsing proof of Bash 3.2 compatibility does not replace runtime proof unless the task acceptance explicitly permits parsing only.
- `BU-0194` (`docs/schema.md (docs/schema.md L21-24)`): Durable callback implementations are executable profiles under `~/.config/sergeant/callbacks/`; they are not project YAML fields, and fleet requests cannot supply paths.
- `BU-0259` (`skills/load-project/SKILL.md (skills/load-project/SKILL.md L39)`): Project YAML files never contain credentials, tokens, or secret values.
- `BU-0265` (`skills/load-project/SKILL.md (skills/load-project/SKILL.md L74)`): When a required executable is missing, the skill reports the executable and a platform-neutral installation requirement rather than inventing a fallback parser.
- `BU-0266` (`skills/load-project/SKILL.md (skills/load-project/SKILL.md L75)`): If the project context-resolution step output and the raw YAML disagree, the project context-resolution step failure is treated as blocking and the YAML is preserved for diagnosis.
- `BU-0817` (`skills/wiki/SKILL.md (skills/wiki/SKILL.md L19-21)`): Curated wiki pages never contain raw prompts, response bodies, credentials, tokens, or secrets copied from source material.
- `BU-0818` (`skills/wiki/SKILL.md (skills/wiki/SKILL.md L19-21)`): Task, repository, PR, merge, decision, and blocker facts are preserved into curated pages only when the wiki schema permits them.
- `BU-0819` (`skills/wiki/SKILL.md (skills/wiki/SKILL.md L27-31)`): Automatic wiki captures are owned exclusively by three commands, each for its own event: the dispatch step captures fleet launch/task/project/branch/repository/brief metadata, the notify step captures escalation or terminal outcome plus any PR URL, and the fleet cleanup step captures worktree/fleet cleanup and final status.
- `BU-0820` (`skills/wiki/SKILL.md (skills/wiki/SKILL.md L33-34)`): A missing automatic capture is fixed by reproducing the owning command in a fixture or repairing its capture adapter; it is never fixed by manually synthesizing a capture as a substitute.
- `BU-0877` (`bin/_sgt-bash-version.sh (bin/_sgt-bash-version.sh L4-19 (_sgt_bash_version_supported / _sgt_require_bash_version))`): Sergeant's Bash entry points refuse to continue when the running Bash interpreter is older than 3.2, printing an error to stderr and returning failure instead of proceeding under an unsupported interpreter.
- `BU-0891` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L426)`): A failure while writing a wiki capture document never fails or blocks the Sergeant operation it was documenting; wiki-write failures are silently swallowed.
- `BU-0898` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L501-506 (SGT_MANAGED_COORDINATOR_COMMAND))`): The managed coordinator pane runs a reader loop that only echoes every line it receives back out and never executes it, so a tmux-injected notification can never become a shell command running in the coordinator's own pane.
- `BU-0942` (`.agents/skills/diagnosing-bugs/SKILL.md (.agents/skills/diagnosing-bugs/SKILL.md L8-8)`): A phase of the diagnosing-bugs discipline may be skipped only when there is an explicit justification for skipping it.
- `BU-1001` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L17-17)`): In human-facing narration and the map's Decisions-so-far, a map or ticket is referred to by its name (title), never by a bare id, number, or slug — the id and URL still exist but ride inside the name link rather than standing in for it.
- `BU-1002` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L21-21)`): The map is a single issue on the repo's issue tracker labelled `wayfinder:map`, and its tickets are child issues of that map.
- `BU-1003` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L23-23)`): The map itself only gists a decision and links to it; the map is an index, not a store, so the decision's actual detail lives in exactly one place — its ticket.
- `BU-1005` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L29-29)`): Open tickets are not listed inline in the map body — they are found by querying open child issues instead, keeping the loaded map view low-resolution.
- `BU-1006` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L65-65)`): Every ticket carries exactly one `wayfinder:<type>` label from the set research, prototype, grilling, task.
- `BU-1008` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L69-69)`): Ticket dependencies use the tracker's native dependency relationship (so the frontier renders visually in the tracker's own UI) unless the tracker lacks native blocking, in which case a body convention is the fallback; a ticket is unblocked once every ticket blocking it is closed, and the frontier is the set of open, unblocked, unclaimed children.
- `BU-1010` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L75-75)`): Every ticket is either HITL — resolvable only through a live exchange with the human, who the agent never stands in for — or AFK, driven by the agent alone; an agent answering its own HITL questions has broken this.
- `BU-1015` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L84-84)`): The map does not chart what can't yet be seen (the fog of war); resolving a ticket clears the fog ahead of it, graduating whatever becomes specifiable into fresh tickets one at a time.
- `BU-1033` (`.agents/skills/codebase-design/SKILL.md (.agents/skills/codebase-design/SKILL.md L12)`): When discussing or designing module boundaries, use the codebase-design glossary terms (module, interface, implementation, depth, seam, adapter, leverage, locality) exactly, rather than substituting generic terms like component, service, API, or boundary.
- `BU-1034` (`.agents/skills/codebase-design/SKILL.md (.agents/skills/codebase-design/SKILL.md L54-58)`): When designing an interface, ask whether the number of methods can be reduced, whether the parameters can be simplified, and whether more complexity can be hidden inside.
- `BU-1035` (`.agents/skills/codebase-design/SKILL.md (.agents/skills/codebase-design/SKILL.md L63)`): To judge whether a module earns its keep, apply the deletion test: imagine deleting the module — if the complexity it held simply vanishes, it was a pass-through; if that complexity reappears spread across its N callers, the module was earning its keep.
- `BU-1036` (`.agents/skills/codebase-design/SKILL.md (.agents/skills/codebase-design/SKILL.md L65)`): Do not introduce a seam unless something actually varies across it: one adapter means the seam is only hypothetical, two adapters means it is real.
- `BU-1037` (`.agents/skills/codebase-design/SKILL.md (.agents/skills/codebase-design/SKILL.md L71)`): For testability, a module should accept its dependencies as parameters rather than constructing them internally.
- `BU-1038` (`.agents/skills/codebase-design/SKILL.md (.agents/skills/codebase-design/SKILL.md L83)`): For testability, a module should return results rather than produce side effects.
- `BU-1039` (`.agents/skills/codebase-design/SKILL.md (.agents/skills/codebase-design/SKILL.md L95)`): Interfaces should be kept to a small surface area, because fewer methods require fewer tests and fewer parameters require simpler test setup.
- `BU-1040` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L9-11)`): A candidate module whose dependencies are in-process (pure computation, in-memory state, no I/O) is always deepenable: merge the modules and test through the new interface directly, with no adapter needed.
- `BU-1041` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L13-15)`): A candidate module whose dependency has a local test stand-in (e.g. PGLite for Postgres, an in-memory filesystem) is deepenable if that stand-in exists: the deepened module is tested with the stand-in in the test suite, and the seam stays internal with no port at the module's external interface.
- `BU-1042` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L17-19)`): A candidate module whose dependency is a remote but owned service (e.g. an internal microservice) is deepened by defining a port at the seam so the deep module owns the logic while the transport is injected as an adapter; tests use an in-memory adapter and production uses an HTTP/gRPC/queue adapter.
- `BU-1043` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L23-25)`): A candidate module whose dependency is a true external, third-party service (e.g. Stripe, Twilio) is deepened by taking the dependency as an injected port, with tests supplying a mock adapter.
- `BU-1044` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L29)`): Do not introduce a port at a seam unless at least two adapters are justified (typically production and test); a single-adapter seam is just indirection.
- `BU-1045` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L30)`): Internal seams — private to a module's own implementation and used only by its own tests — should not be exposed through the module's external interface just because tests happen to use them.
- `BU-1046` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L34)`): Once tests exist at a deepened module's interface, the old unit tests on the shallow modules it replaced become waste and should be deleted.
- `BU-1047` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L35-36)`): New tests written for a deepened module are written at its interface and assert on observable outcomes through that interface, not on internal state.
- `BU-1048` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L37)`): A test that must change when a module's implementation changes without any corresponding interface change is a signal that the test is testing past the interface rather than describing behavior.
- `BU-1064` (`.agents/skills/domain-modeling/SKILL.md (.agents/skills/domain-modeling/SKILL.md L64)`): CONTEXT.md is restricted to glossary content: it must not be treated as a spec, a scratch pad, or a repository for implementation decisions, and must stay devoid of implementation details.
- `BU-1080` (`.agents/skills/prototype/SKILL.md (.agents/skills/prototype/SKILL.md L21)`): Prototype code is located close to where it will actually be used, but named so a casual reader can tell it is a prototype, not production, and throwaway UI routes follow the project's existing routing convention rather than inventing a new top-level structure.
- `BU-1081` (`.agents/skills/prototype/SKILL.md (.agents/skills/prototype/SKILL.md L22)`): A prototype must be runnable with one command, using whatever task runner the project already supports, so the user can start it without having to think about how.
- `BU-1082` (`.agents/skills/prototype/SKILL.md (.agents/skills/prototype/SKILL.md L23)`): A prototype has no persistence by default (state lives in memory); if the question explicitly involves a database, the prototype hits a scratch DB or local file with a clear "PROTOTYPE — wipe me" name rather than a real data store.
- `BU-1083` (`.agents/skills/prototype/SKILL.md (.agents/skills/prototype/SKILL.md L24)`): A prototype skips polish: no tests, no error handling beyond what's needed to make it runnable, and no abstractions, because the point is to learn something fast.
- `BU-1084` (`.agents/skills/prototype/SKILL.md (.agents/skills/prototype/SKILL.md L25)`): A prototype surfaces its full relevant state after every action (logic branch) or on every variant switch (UI branch), so the user can see what changed.
- `BU-1126` (`.agents/skills/tdd/SKILL.md (.agents/skills/tdd/SKILL.md L8)`): This skill's sections on what a good test is, where tests go, the anti-patterns, and the rules of the loop are consulted before and during every TDD cycle, not only afterward.
- `BU-1128` (`.agents/skills/tdd/SKILL.md (.agents/skills/tdd/SKILL.md L14)`): A good test verifies behavior through public interfaces rather than implementation details, reads like a specification of a capability, and survives refactors because it does not depend on internal structure.
- `BU-1129` (`.agents/skills/tdd/SKILL.md (.agents/skills/tdd/SKILL.md L20)`): A test lives at a seam — the public boundary where behavior is observed without reaching inside — and never against internals.
- `BU-1131` (`.agents/skills/tdd/SKILL.md (.agents/skills/tdd/SKILL.md L28)`): An implementation-coupled test — one that mocks internal collaborators, tests private methods, or verifies through a side channel like querying the database instead of the interface — is an anti-pattern, tellingly breaking on refactors that don't change behavior.
- `BU-1132` (`.agents/skills/tdd/SKILL.md (.agents/skills/tdd/SKILL.md L29)`): A tautological test — one whose expected value is recomputed the same way the code computes it, so it passes by construction — is an anti-pattern; expected values must instead come from an independent source of truth such as a known-good literal, a worked example, or the spec.
- `BU-1137` (`.agents/skills/tdd/mocking.md (.agents/skills/tdd/mocking.md L3-8)`): Mocking is used only at system boundaries: external APIs, databases (sometimes, a test DB is preferred), time/randomness, and the filesystem (sometimes).
- `BU-1138` (`.agents/skills/tdd/mocking.md (.agents/skills/tdd/mocking.md L10-14)`): Your own classes/modules, internal collaborators, and anything you control are never mocked.
- `BU-1139` (`.agents/skills/tdd/mocking.md (.agents/skills/tdd/mocking.md L20-22)`): For mockability, external dependencies at system boundaries are passed into a function/module (dependency injection) rather than being constructed internally by it.
- `BU-1140` (`.agents/skills/tdd/mocking.md (.agents/skills/tdd/mocking.md L37-39)`): For mockability, SDK-style interfaces (a specific function per external operation) are preferred over one generic fetcher with conditional logic, because each mock then returns one specific shape, test setup needs no conditional logic, it's easier to see which endpoints a test exercises, and type safety is per-endpoint.
- `BU-1141` (`.agents/skills/tdd/tests.md (.agents/skills/tdd/tests.md L17-23)`): A good test exhibits five characteristics together: it tests behavior users/callers care about, uses only the public API, survives internal refactors, describes WHAT rather than HOW, and makes one logical assertion.
- `BU-1142` (`.agents/skills/tdd/tests.md (.agents/skills/tdd/tests.md L38-45)`): A bad, implementation-detail test is recognized by any of six red flags: mocking internal collaborators, testing private methods, asserting on call counts/order, breaking on refactors without a behavior change, a name that describes HOW rather than WHAT, or verifying through external means instead of the interface (e.g. querying the database directly rather than using the createUser/getUser interface).
- `BU-1143` (`.agents/skills/triage/SKILL.md (SKILL.md L11)`): When the subject repository treats external pull requests as a request surface, triage handles a PR through the same category/state roles and the same state machine as an issue, with only a small set of PR-specific deltas.
- `BU-1145` (`.agents/skills/triage/SKILL.md (SKILL.md L13-17)`): Every comment or issue the triage skill posts to the issue tracker must begin with a disclaimer stating it was generated by AI during triage.
- `BU-1146` (`.agents/skills/triage/SKILL.md (SKILL.md L41)`): Every triaged issue carries exactly one category role and exactly one state role.
- `BU-1196` (`.agents/skills/no-mistakes/SKILL.md (SKILL.md L8-13)`): Workers and remediation loops must never invoke the validation pipeline; the Sergeant coordinator alone owns every validation pipeline gate.
- `BU-1260` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L9-10)`): The setup skill orchestrates only supported Sergeant and bootstrap commands and must not substitute undocumented workarounds for a capability that is missing.
- `BU-1261` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L9-10)`): Missing capabilities encountered during setup are surfaced as separate task tracker issues rather than worked around.
- `BU-1262` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L20-21)`): This skill must not be loaded when the user wants documentation only or is asking about a specific command; `sergeant-help` is used instead in both cases.
- `BU-1263` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L25-29)`): This skill writes only to Sergeant-owned paths: `~/.config/sergeant/config.yaml` (global config) and `~/.config/sergeant/<project>.yaml` (project YAML files).
- `BU-1264` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L30-37)`): This skill must never write to opencode's config, Claude's config or `CLAUDE.md` or any `.claude/` directory, Codex's config, Goose's config, any repository's `AGENTS.md`/`.github/`/other agent configuration paths, or any path outside `~/.config/sergeant/` the user has not explicitly named.
- `BU-1265` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L39-41)`): The skill does not automatically initialize the task tracker, Graphify, or Treehouse; each requires an explicit confirmation prompt before any command runs, and if consent is declined the skill leaves state unchanged and reports what was skipped.
- `BU-1295` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L296-298)`): Re-running this skill after a successful setup must produce the same final state: each phase skips steps that already pass verification, and no phase destroys existing working configuration to reach the same end state.
- `BU-1297` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L15)`): Prefer vertical slices that produce independently verifiable behavior when drafting tickets.
- `BU-1298` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L16)`): Keep each ticket small enough for one fresh agent context.
- `BU-1299` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L17)`): Assign exactly one owning repository to each implementation ticket.
- `BU-1300` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L18-19)`): Represent cross-repository delivery with counterpart tickets and explicit merge order, not one ambiguous shared ticket.
- `BU-1301` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L20-21)`): Use expand-migrate-contract for mechanical changes that cannot remain green as a vertical slice.
- `BU-1302` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L22)`): Create epics for coherent programs of work, not as substitutes for executable tickets.
- `BU-1303` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L23)`): Never duplicate an existing task tracker task or GitHub issue.
- `BU-1304` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L24)`): Preserve stable finding IDs such as `RBAC-P1-004` or `DATA-P0-002` in ticket titles.
- `BU-1305` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L25-26)`): A ticket is not ready unless its acceptance criteria are observable and its blockers are accurate.
- `BU-1311` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L46)`): Do not automatically add the task tracker instructions to repository guidance files.

### 3.2 Obsolete-mechanism findings — Bucket 6, and Engine-pressure candidates — Bucket 7

## Bucket 6: Obsolete-mechanism findings

**0** records this run. None — bucket reported empty per "what must not happen."

## Bucket 7: Engine-pressure candidates

**0** records this run. None — bucket reported empty per "what must not happen."

## 4. Out-of-scope content noted, not silently dropped

`../../50-synthesize/output/candidates.md` carries two further sections this stage's `CONTEXT.md` does not name as either "materialize" or "carry forward in `draft-report.md`": Bucket 5 (shared helper/context candidates — 23 `shared-helper`, 0 `shared-context`) and the "Unattached records" section (synthesis-time defects: 4 records missing a `workflow` field entirely, plus 1 `workflow`+`stage` pair with no matching `stage` candidate). Neither is a workflow candidate (so §60-draft's materialize-as-a-package instruction does not apply to them), and neither is named among the three carried-through categories in this stage's own `CONTEXT.md` ("permanent-instruction, obsolete-mechanism, and engine-pressure") or in this same directory's own `README.md` (this stage's `output/README.md`, describing the expected artifact). Per `../../_config/run-discipline.md`'s spirit of never silently dropping something a stage's own contract does not have a slot for, and per this stage's own mandated meta-level-grammar-pressure practice (§5 below): both sections are reproduced verbatim here so `90-reconcile` can see them, rather than being left for a later reader to discover only by re-opening `50-synthesize`'s own output. This is an observation, not a claim that either section is itself an `engine-gap` — that adjudication is `90-reconcile`'s to make, per `../../90-reconcile/references/reconciliation-method.md` §3.

### 4.1 Bucket 5: Shared helper/context candidates

## Bucket 5: Shared helper/context candidates

`shared-helper`: **23** records. `shared-context`: **0** records (none this run).

No `.sergeant/common/contexts/` or `.sergeant/common/scripts/` directory
exists in this worktree yet (checked directly) — every candidate below is a
new promotion candidate, not a name collision to reconcile against an
existing entry.

**Over-promotion tell check** (`../_config/icm-ladder.md` §6.6): grouping the
23 `shared-helper` records by contract below yields 9 groups; the only
groups whose membership is entirely drawn from one source file
(`bin/_sgt-lib.sh`) are "tmux pane-identity verification" (2 of that file's
8 records), "owned state-file read validation" (3 of 8), and half of
"owned state-file atomic write / publish" (1 of 8, paired with a
`bin/sgt-watch`-adjacent record) — none is *all* of that file's records, so
the tell (a bucket-5 group == one whole source file's own unit set) is not
triggered for this corpus.

### dev_root-relative repo path resolution

Contract: given a repo `path` value from a project YAML (or a CLI path argument), resolve it to an absolute path — used verbatim if already absolute or home-relative (`~...`), otherwise resolved relative to `dev_root`. Same contract stated three times from three source files (`AGENTS.md`, `docs/schema.md`, `bin/_sgt-lib.sh`) — genuine cross-file behavior-shape clustering, not file mirroring.

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0051` (`AGENTS.md (AGENTS.md L179)`): `dev_root`, set in `~/.config/sergeant/config.yaml`, is the base against which repo paths in project YAMLs are resolved as relative paths.
- `BU-0193` (`docs/schema.md (docs/schema.md L19)`): Repo `path` values that are not absolute or home-relative are resolved relative to `dev_root`, so project YAMLs stay portable across machines by changing `dev_root` in one place.
- `BU-0889` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L401-410 (_resolve_path))`): A repository path argument that is already absolute or home-relative (~...) is used verbatim; any other form is resolved relative to the configured development root (DEV_ROOT).

### project name/filename identity rule

Contract: a project's `name` field (or identity) must equal its YAML filename without extension. Same contract from `AGENTS.md` and `docs/schema.md`.

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0052` (`AGENTS.md (AGENTS.md L180)`): A project's name is the YAML filename without its extension.
- `BU-0195` (`docs/schema.md (docs/schema.md L40)`): A project's `name` field must match its filename.

### agent-instruction layer concatenation order

Contract: instruction layers concatenate in a fixed order — defaults, then group, then repo — with later layers appearing later (later-wins on conflict). Same contract from `AGENTS.md` and `docs/schema.md`.

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0053` (`AGENTS.md (AGENTS.md L181)`): The project context-resolution step resolves instructions in order — `defaults.agent_instructions`, then group instructions, then repo instructions — with later layers overriding earlier ones for the same repo.
- `BU-0200` (`docs/schema.md (docs/schema.md L101-107)`): Agent instruction layers are concatenated in order (defaults, group, repo) with later layers appearing later in the block, and when directives conflict the later, more specific repository-level directive is the intended authority; Sergeant does not structurally merge or deduplicate the free-form prose.

### fleet-watch snapshot busy/basis contract

Contract: the `--snapshot` / interactive fleet-watch loop is read-only, constant-size, and reports `busy` from a closed two-value `basis` allowlist — `busy:true` only on a verified-active match, `busy:null` with `basis: no_verified_active_match` otherwise, any unrecognised condition mapping to the null basis rather than a guess. Same contract documented in `README.md` and implemented in `bin/sgt-watch` — 5 records, 2 source files, genuine behavior-shape clustering.

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0062` (`README.md (README.md L151-168)`): The interactive fleet-watch loop is strictly read-only and emits constant-size versioned JSON with `busy: true` only when all three hold: a stable `in_progress` status, an exact live Sergeant worker pane identity, and progress attributable to that pane within `SERGEANT_SNAPSHOT_RECENT_SECONDS` (default 300).
- `BU-0063` (`README.md (README.md L151-168)`): Every snapshot outcome other than the verified-busy case reports `busy: null` with `basis: no_verified_active_witness`; version 1 never emits `busy: false`, because absence of a verified witness is not proof of idleness.
- `BU-0064` (`README.md (README.md L151-168)`): `basis` is a closed allowlist of exactly two values, so an unrecognised condition maps to the null basis rather than inventing a new one.
- `BU-0570` (`bin/sgt-watch (bin/sgt-watch L46-49)`): A --snapshot observation reports busy:true only when a stable active status, an exact live worker identity match, and recently attributable progress all hold together; any other outcome reports busy:null, never busy:false, because absence of a verified witness is not proof of idleness.
- `BU-0571` (`bin/sgt-watch (bin/sgt-watch L51-52)`): The --snapshot basis field is restricted to a closed set of exactly two values; an unrecognized condition maps to the null basis rather than a newly invented one.

### worker-brief independent-review-axis contract

Contract: one shared axis/severity definition drives both the brief-rendering and dispatch-instruction halves of the independent-review requirement; an axis with no defined reviewer guidance fails brief-rendering rather than emitting an unreviewed brief. `README.md` + `bin/_sgt-review-axes.sh`.

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0095` (`README.md (README.md L314)`): Every worker brief requires one independent review per axis named in `SGT_REVIEW_AXES_REQUIRED`, and frontend/UI/visual/interaction/accessibility/user-facing-output language in the mission, repo role, or repo group additionally requires the conditional Accessibility axis; the one definition drives both what the brief demands and what `--axis` values the review-findings router accepts, so the two cannot drift apart.
- `BU-0300` (`bin/_sgt-review-axes.sh (bin/_sgt-review-axes.sh L4-9)`): One shared definition drives both halves of the independent-review contract — the axes/severities dispatch instructs a worker to produce, and the axes/severities the review-findings router accepts for routing — because previously writing them out separately let them drift (dispatch mandated a readiness review the router rejected outright, td-61a0c8).
- `BU-0301` (`bin/_sgt-review-axes.sh (bin/_sgt-review-axes.sh L41)`): An axis with no defined reviewer guidance fails the brief-rendering step rather than silently emitting an unexplained axis name.

### coordinator notify-marker wake contract

Contract: workers wake the coordinator by updating one shared per-task notify marker that the interactive fleet-watch loop polls. Single record (`docs/using-sergeant.md`) — no second record shares this exact contract, so it stays its own group rather than being force-merged into the pane-identity or notify-target-pointer groups below, which cover a related but distinct mechanism (see note).

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0143` (`docs/using-sergeant.md (docs/using-sergeant.md L157-159)`): Workers wake the coordinator by updating one shared per-task notify marker in fleet state; the interactive fleet-watch loop polls that marker, so simultaneous repo updates can at worst collapse into a delayed wakeup rather than duplicate delivery.

### tmux pane-identity verification

Contract: a tmux pane (or a previously recorded target pane) is only treated as the correct live destination if the live tmux server confirms the pane exists, is not dead, and its identity matches what was recorded — never inferred from pane position/index alone. Both records are from `bin/_sgt-lib.sh`; this group is 2 of that file's 8 `shared-helper` records, not all of them, so the over-promotion tell (a group == one whole source file's unit set) does not apply here.

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0896` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L484-495 (_sgt_verify_pane_identity))`): A tmux pane's identity is accepted only when the live tmux server confirms the pane exists, is not dead, and reports back the exact pane id that was asked for; an absent, dead, or mismatched pane fails the check rather than being adopted.
- `BU-0905` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L732-744 (_sgt_pane_identity_matches))`): A target pane is treated as still the correct destination only if its live identity matches a previously recorded expected identity (read via the strict or legacy-migration reader), and the live identity is re-checked once more immediately after that lookup before being trusted.

### owned state-file read validation

Contract: a state file is read as 'owned' only if it is a regular file (not a symlink), owned by the current user, and — for a hard-linked pair — both paths resolve to the same inode with matching content; a looser historical permission mode (640/644/660/664) is still accepted for backward compatibility. All 3 records are from `bin/_sgt-lib.sh` (3 of that file's 8 `shared-helper` records — again not the file's full set).

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0901` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L591-619 (_sgt_read_owned_file))`): A strict-mode owned state file is read only if it is a regular file (not a symlink), owned by the current user, and mode 600; its identity (inode:device) and mode are re-verified immediately after opening and again after the read completes, and the read is rejected if either check reveals the path was swapped or its mode/ownership changed underneath it.
- `BU-0902` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L620-677 (_sgt_read_matching_legacy_pane_identity))`): A state file stored under a looser historical permission mode (640/644/660/664) is still read for backward-compatible migration, but only when its value exactly matches an already-known expected value; once confirmed, it is atomically rewritten at the strict mode 600 and the migrated value is re-verified before being trusted.
- `BU-0903` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L678-724 (_sgt_read_same_owned_files))`): Reading a hard-linked pair of owned state files requires both paths to resolve to the same inode and to yield the same value, with identity (inode:device) and mode re-verified before and after both reads; any mismatch, mode drift, or broken hardlink relationship fails the read.

### owned state-file atomic write / publish

Contract: an owned/published state value is updated by writing to a private (mode 600) temporary file and then atomically renaming it into place — `BU-0904` states the general owned-state-file form, `BU-0906` applies the same temp-file+atomic-rename pattern specifically to publishing the current notification target via a nonce-bearing pointer file. Grouped as the same mechanism applied to two named artifacts, not force-merged into one contract — `BU-0906`'s consumer is the notify-target lookup the pane-identity group above feeds, `BU-0904`'s is any owned state file generally.

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0904` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L725-731 (_sgt_replace_owned_file))`): An owned state file is updated by writing the new value to a private (mode 600) temporary file and then atomically renaming it into place, so a concurrent reader never observes a partially-written value.
- `BU-0906` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L748-781 (_sgt_notification_target_create))`): The current notification target is published by atomically renaming a nonce-bearing pointer file into place, and the publish is then re-verified by re-reading that pointer; if a concurrent publisher's write landed after this rename, this attempt's own target-directory state is removed and the call reports failure rather than claiming to have set the target.

`shared-context`: 0 records this run — bucket reported empty, not omitted.

### 4.2 Unattached records

## Unattached records (synthesis-time defects surfaced here, not silently resolved)

Per synthesis-method.md buckets 1 and 3: a `helper`/`stage`/`stage-context`
record with no `workflow` value, or naming a `workflow`+`stage` pair with no
matching `stage` candidate, is a `40-classify`-stage defect surfacing here —
recorded plainly, not invented a home for.

### Missing `workflow` field entirely

- `BU-0878` (`helper`, `workflow=null`, `stage=null`) —
  `bin/_sgt-lib.sh (bin/_sgt-lib.sh L12-13)`: Sourcing the shared library helper file more than once in the same shell process is a no-op after the first source.
- `BU-0890` (`helper`, `workflow=null`, `stage=null`) —
  `bin/_sgt-lib.sh (bin/_sgt-lib.sh L419-420 (_die / _info))`: An unrecoverable error in any sgt-* script prints an ERROR-prefixed message to stderr and terminates the process with a non-zero exit code.
- `BU-0892` (`helper`, `workflow=null`, `stage=null`) —
  `bin/_sgt-lib.sh (bin/_sgt-lib.sh L438-449 (_sgt_wiki_write))`: Wiki capture only runs at all if the configured wiki-writer script exists and is executable and the operator has not set SGT_WIKI_DISABLED=1; otherwise the capture call is a silent no-op.
- `BU-0893` (`helper`, `workflow=null`, `stage=null`) —
  `bin/_sgt-lib.sh (bin/_sgt-lib.sh L452-460 (_require_yq / _require_tmux / _require_git))`: Sergeant dies with an actionable installation hint (e.g. 'brew install yq') when a required external tool (yq, tmux, or git) is not found on PATH.

### `workflow`+`stage` names a checkpoint with no matching `stage` candidate

- **`standard-workflow` / `monitor-progress`** — no `stage`-rung record in this corpus
  classifies this checkpoint; the following record(s) name it anyway:
  - `BU-0032` (`stage-context`) — `AGENTS.md (AGENTS.md L144)`: Step 7 of the standard workflow: monitoring real progress requires recent meaningful events or an active child operation plus exact pane/process identity; parent-process liveness alone is insufficient.
  - `BU-0033` (`stage-context`) — `AGENTS.md (AGENTS.md L144)`: In OpenCode, run the interactive fleet-watch loop and verify the monitor started (unit identity printed); if managed background execution is unavailable, use bounded one-shot status checks rather than a blocking watch call.
  - `BU-0144` (`stage-context`) — `docs/using-sergeant.md (docs/using-sergeant.md L161-166)`: `in_progress` is never equated with health; the interactive fleet-watch loop requires exact live worker-pane identity plus recent meaningful progress evidence, preferring tmux `pane_activity`, falling back to the worker's recorded `progress_ts`, and using `.sergeant-status` mtime only when no better timestamp exists.
  - `BU-0145` (`stage-context`) — `docs/using-sergeant.md (docs/using-sergeant.md L165-168)`: When progress evidence stays older than the default 300-second grace window, the interactive fleet-watch loop keeps the repo `in_progress` and records a nonterminal `live worker stalled` diagnostic instead of declaring it done, failed, or orphaned.
  - `BU-0174` (`stage-context`) — `docs/troubleshooting.md (docs/troubleshooting.md L61-68)`: A live parent process is insufficient evidence of progress; `in_progress` plus a `live worker stalled` diagnostic is still nonterminal and must be reconciled through the progress rules before killing or relaunching anything, preserving worktree/branch/task/response-generation/handoff first, and using the stalled-worker recovery step only for that exact stall classification.
  - `BU-0274` (`stage-context`) — `skills/dispatch/SKILL.md (skills/dispatch/SKILL.md L92)`: `needs_input` and `blocked` are distinct nonterminal states; a worker waiting on CI, review threads, or dependencies remains `in_progress` unless it needs to escalate.
  - `BU-0568` (`helper`) — `bin/sgt-watch (bin/sgt-watch L14-23)`: The interactive fleet-watch loop accepts the --background flag either before or after the task-id argument when entering background-watch mode, so both call-site conventions resolve to the same task.
  - `BU-0569` (`stage-context`) — `bin/sgt-watch (bin/sgt-watch L42-44)`: The --snapshot observation path is strictly read-only: it runs before any reconciliation, never writes fleet state, and deliberately avoids the pane-identity migration side effect that ordinary reconciliation performs.
  - `BU-0572` (`helper`) — `bin/sgt-watch (bin/sgt-watch L136-146)`: --snapshot validates every caller-supplied task-id and repo scope value against a fixed identifier pattern before observing any fleet state, so a malformed scope value can never reach the emitted document and cannot make the document unbounded.
  - `BU-0573` (`stage-context`) — `bin/sgt-watch (bin/sgt-watch L63-77)`: A --snapshot repo or task observation only counts a pane as a verified worker when the live tmux pane identity exactly matches the identity recorded in that repo's pane_identity file; an ambient pane occupying the recorded pane id is not sufficient.
  - `BU-0574` (`stage-context`) — `bin/sgt-watch (bin/sgt-watch L84-86)`: Only a repo whose recorded status is exactly in_progress can ever count as an active witness in a --snapshot observation; terminal, waiting, and unreconciled statuses never do.
  - `BU-0575` (`stage-context`) — `bin/sgt-watch (bin/sgt-watch L90-100)`: A --snapshot active-witness determination treats the more recent of the recorded progress_ts and the live tmux pane-activity timestamp as the last-event time, and only counts the witness as active if that time falls within a configurable recent-seconds window (default 300s).
  - `BU-0577` (`stage-context`) — `bin/sgt-watch (bin/sgt-watch L239-242)`: Once a repo has a persisted pane_identity file with content, the interactive fleet-watch loop verifies pane ownership against that exact recorded identity rather than recomputing eligibility criteria from scratch.
  - `BU-0578` (`stage-context`) — `bin/sgt-watch (bin/sgt-watch L256-263)`: The first time the interactive fleet-watch loop verifies a pane against inferred (pre-migration) worker criteria, it persists the verified identity to pane_identity, and a parallel pane_identity_migration record, so future checks use the exact recorded identity instead of re-deriving it every time.
  - `BU-0601` (`helper`) — `bin/sgt-watch (bin/sgt-watch L694-715)`: The interactive fleet-watch loop only reprints task status when the aggregated per-repo snapshot (status, stage, validation_status, message, diagnostic, plus the task's notify marker) actually differs from the last printed snapshot, rather than redrawing on every poll tick.
  - `BU-0602` (`stage-context`) — `bin/sgt-watch (bin/sgt-watch L716-735)`: The interactive fleet-watch loop exits 0 once every repo in the task reaches a terminal, non-failed status, and exits 1 as soon as any repo is failed or orphaned, even while others are still nonterminal; otherwise it keeps polling at POLL_INTERVAL (default 5s, SERGEANT_WATCH_INTERVAL).

## 5. Meta-level grammar pressure, for `90-reconcile`

The materialized packages under `.sergeant/drafts/workflows/` are this run's principal deliverable, yet the D9 disposition/finalize mechanism (`docs/icm/convention.md` §1a) only governs a stage's own `output/` — it has no lower-rung way to give per-run content written *elsewhere* in the worktree (the draft packages themselves) a disposition, or to bring it under `../../scripts/finalize.py`'s reach. This is a genuine could-not-express moment, not something this stage is silently accepting as fine: stated here per `../../_config/run-discipline.md` and `../../90-reconcile/references/reconciliation-method.md` §3, for `90-reconcile` to write up as the full six-field engine-gap template, which is not this stage's job.

