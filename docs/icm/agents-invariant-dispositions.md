# `agents-invariant` unit dispositions — MVP-5 Lane F1

Every one of the 126 units N2 run 4's classifier assigned
`representation: agents-invariant` (verbatim, "listed, not drafted" per
`docs/gauntlet/runs/n2-run4/run-manifest.md:152`, and the re-triage's own
"awaiting owner spot-check" status — this document *is* that spot-check,
executed 2026-08-12 as part of MVP-5 Lane F1's `AGENTS.md` rewrite) gets
exactly one row below: which of `AGENTS.md`, a named skill/workflow,
`docs/DEVELOPMENT.md`, or **not-adopted** it lands in, and why. No unit is
silently dropped — a not-adopted row states the reason a reader could
challenge (inapplicable to this stack, already satisfied structurally,
superseded by sergeant-rs's own architecture, parked pending a skill that
doesn't exist yet, explicitly NOT-EVER per `NORTH-STAR.md`, or out of this
lane's file scope with the gap named for a future pass).

Source: `docs/gauntlet/runs/n2-run4/.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`
(filtered to `representation: agents-invariant`) joined against
`.../30-normalize/output/behavior-units.normalized.ndjson` for each unit's
`statement`.

## Tally

| Disposition | Count |
|---|---|
| `AGENTS.md` (cited by id, HTML comment) | 40 |
| `docs/DEVELOPMENT.md` ("Session conduct") | 9 |
| Named skill/workflow (already published under `.sergeant/workflows/`, not built by this lane) | 55 |
| `NOT-EVER` (per `NORTH-STAR.md`'s Never list — fleet-as-object, tmux-era supervision) | 4 |
| Not-adopted (other reasons, each stated) | 18 |
| **Total** | **126** |

Many rows are near-duplicates of an upstream document restating the same
rule in different words (e.g. the four liveness-is-not-progress units
BU-0038/BU-0047/BU-0111/BU-0115, or the two secrets-in-config units
BU-0055/BU-0259) — these consolidate onto one `AGENTS.md`/`docs/DEVELOPMENT.md`
citation rather than duplicating text, and the table below says so
explicitly rather than hiding the consolidation.

## Reading this table

- **`AGENTS.md`** rows are cited in the file itself as `<!-- BU-xxxx -->`
  next to the line(s) that satisfy them (several ids often share one line,
  where the units were duplicates or a consolidated group).
- **`skill: <name>`** rows belong to a workflow already published under
  `.sergeant/workflows/<name>/` per `docs/icm/retriage-2026-08-11.md`'s
  verdicts — this lane did not edit those packages; the disposition records
  where the unit's content structurally belongs, for whoever next revises
  that workflow.
- **`docs/DEVELOPMENT.md`** rows are cited under that file's "Session
  conduct" section the same way.
- **`NOT-EVER`** rows map to mechanisms `NORTH-STAR.md`'s "Never" list rules
  out outright (fleet as a domain object, reconstructed tmux-era
  supervision) — not merely deferred, structurally excluded.
- **`not-adopted`** rows are not a defect count against this pass; most are
  "the upstream Bash/tmux/wiki mechanism this unit describes has no
  sergeant-rs equivalent" or "already true here for a different, better
  reason" — each row says which.

## Full table

| BU id | Statement | Disposition | Note |
|---|---|---|---|
| BU-0001 | Before acting on a project, resolve its repositories, roles, inherited instructions, and configured paths with the project context-resolution step. | AGENTS.md | Standard workflow loop step 1 (load estate context). |
| BU-0002 | Ownership of a project is never inferred from the current working directory. | AGENTS.md | Standard workflow loop step 1 (never infer from cwd). |
| BU-0003 | The primary Sergeant session coordinates multi-repository work by default rather than implementing directly. | AGENTS.md | "When NOT to use sgt" / North Star ruling 4 — OS owns the routing judgment. |
| BU-0004 | Direct implementation in the primary session is permitted only when the user explicitly asks to work in-session (or says not to dispatch) and one repository owns the complete outcome. | AGENTS.md | "When NOT to use sgt" — single-turn-in-session criterion. |
| BU-0005 | Dispatch mode is used when work spans repositories, contains two or more independent repository-owned tasks, needs an isolated independent review worker, or the user asks for workers. | AGENTS.md | Standard workflow loop — when to reach for `sgt run` at all. |
| BU-0009 | Direct mode is used only when the user explicitly requests it and the work has one clear owning repository. | AGENTS.md | Duplicate of BU-0004; same citation, same routing-table section. |
| BU-0017 | The coordinator role is never used as a reason to stop at a plan, status report, or dispatch suggestion when the user asked for an implemented outcome. | AGENTS.md | Standard workflow loop step 8 (collect — a plan is not the outcome). |
| BU-0018 | Direct mode is never used to edit several repositories in one checkout, or to bypass repository instructions, task ownership, review independence, or shipping gates. | docs/DEVELOPMENT.md | "Session conduct" — direct-mode-in-session still single-scope, gate-bound. |
| BU-0019 | When a toolbelt command covers an operation, it is used instead of reproducing the operation with ad hoc shell commands. | AGENTS.md | Guardrails — prefer sgt verbs over ad hoc shell. |
| BU-0020 | The bare `sgt-*` command name is used when it resolves on PATH; otherwise the matching script is run from this repository's `bin/` directory. | AGENTS.md | Guardrails — dup with BU-0056; translated (no `bin/` scripts here, `sgt` on PATH is the one path) and folded into the same bullet as BU-0019/BU-0021. |
| BU-0021 | Manual fallback operations are used only when no toolbelt command covers the operation, or the command returns an explicit unsupported-case error; the fallback and the original error evidence are reported. | AGENTS.md | Guardrails — fallback only when no verb covers it, report the fallback + original error. |
| BU-0022 | For every listed procedural-skill trigger, the repository-local SKILL.md file is read directly; it is canonical and takes precedence over any same-named registry skill. | AGENTS.md | Skills/procedures section — repo-local skill file is canonical. |
| BU-0023 | A harness registry's omission of a skill does not make the skill unavailable, and the owner is not asked or the task stopped solely because the registry omits it. | AGENTS.md | Skills/procedures section — registry omission never blocks. |
| BU-0024 | The session stops and reports the exact repository-local skill path only when that file is absent or unreadable, and does not reconstruct a partial protocol from memory in that case. | AGENTS.md | Skills/procedures section — stop + report only on absent/unreadable file. |
| BU-0036 | `in_progress`, `needs_input`, `blocked`, and `waiting` are treated as nonterminal worker states; a waiting worker may remain alive or may exit after a durable handoff. | AGENTS.md | Standard workflow loop step 6 — Work states mapped onto sgt's real states. |
| BU-0037 | Deferred waits are resumed through the wake-condition step when a durable `.sergeant-wake-condition` has been published; human decisions are resumed through the worker response-delivery step. | AGENTS.md | Standard workflow loop step 7 — the human-response half (`sgt respond`) adopted; the deferred wake-condition half has no sergeant-rs mechanism yet (G1 scheduler is post-MVP, gated on "a promotion policy someone wants automated" per `NORTH-STAR.md`) so nothing exists to route to until G1 lands. |
| BU-0038 | Progress is never inferred from liveness alone, an expected blocked exit is never rewritten as orphaned, and a waiting worktree is never cleaned. | AGENTS.md | Standard workflow loop step 6 — liveness is not progress; translated from tmux panes/worktree-cleanup to sgt's process-vs-journal-state distinction. |
| BU-0039 | The worker response-delivery step, the wake-condition step, or supported recovery are used only after reconciling status, response generation, pane identity, and handoff evidence. | AGENTS.md | Standard workflow loop step 7 — respond/retry only after reading current state. |
| BU-0040 | Every dispatched implementation, independent review, PR description, successor, recovery, and final shipping gate must use the same canonical intent revision from `.sergeant-intent.md`. | not-adopted | Already satisfied structurally, not by new `AGENTS.md` text: sergeant-rs pins a single canonical intent per Work directly in the immutable journal (journal-is-only-truth + single-canonical-intent-per-Work, already on the R1 engine-capability shelf per `docs/icm/retriage-2026-08-11.md`'s second pass) — there is no separate `.sergeant-intent.md` file for a downstream action to disagree with. |
| BU-0041 | Workers and remediation loops never run the validation pipeline themselves. | docs/DEVELOPMENT.md | "Session conduct" — a workflow stage/actor never invokes the shipping gate itself; only the top-level session does. |
| BU-0044 | A plan, task, finding, or worker launch is not treated as the requested outcome unless the user asked only for planning or dispatch. | AGENTS.md | Standard workflow loop step 8 / guardrails — a plan/dispatch is not the deliverable. |
| BU-0045 | A known blocker is not repeatedly reported once its decision and remediation path are approved; the next safe step is executed instead. | AGENTS.md | Standard workflow loop step 8 — don't re-report an already-approved blocker. |
| BU-0046 | Duplicate tasks, findings, PRs, workers, or review passes are not created when a canonical preserved owner already exists. | AGENTS.md | Standard workflow loop step 2 — reuse a matching Work item, don't duplicate. |
| BU-0047 | A worker is not called active solely because its process or pane exists; recent meaningful progress evidence is required. | AGENTS.md | Standard workflow loop step 6 — a Work item isn't active merely because a process exists. |
| BU-0048 | A completed, merged, blocked, or abandoned task is never left recorded as `in_progress`; the task tracker and fleet state are reconciled truthfully. | AGENTS.md | Standard workflow loop step 2 — trust `sgt status`/`work list` over memory of prior state. |
| BU-0049 | Tool absence produces an actionable fallback or explicit blocker, never a silent skip, false success, or indefinite wait. | AGENTS.md | Guardrails — tool/capability absence surfaces via `sgt doctor`'s named remedy, never a silent skip. |
| BU-0050 | Standing authorization may remove repetitive dispatch confirmation, but never authorizes risk acceptance, gate skipping, force operations, secret exposure, or destruction of preserved state. | AGENTS.md | Guardrails — standing authorization never extends to gate-skipping, force ops, secrets, or destroying preserved state. |
| BU-0054 | Repositories under `~/.config/sergeant/` are never modified — that location is config, not code. | not-adopted | Path-specific to upstream's `~/.config/sergeant/`; sergeant-rs's structural analog predates this pass — the daemon exclusively owns its data dir (journal/blobs) and every client reaches it only through the API (`NORTH-STAR.md` Ownership, "One owner"), so hand-editing it is already excluded by architecture, not by an `AGENTS.md` reminder. |
| BU-0055 | Secrets are never committed; project YAMLs may contain paths but must not contain credentials. | docs/DEVELOPMENT.md | "Session conduct" and `AGENTS.md` guardrails both cite the general form: secrets never committed. |
| BU-0056 | A bare `sgt-*` command is used when `command -v <name>` succeeds; otherwise the equivalent `bin/<name>` from this repository is run. | AGENTS.md | Duplicate of BU-0020; same citation. |
| BU-0061 | The interactive fleet-watch loop and `--sync-all` reconcile lifecycle state and may kill panes, so neither is safe for a coordinator or bridge that only wants to observe. | NOT-EVER | Fleet-watch mutating-sync-vs-observer ambiguity has no sergeant-rs analog: there is no separate "sync" verb and every client (CLI/TUI/dashboard) is already read-only against the API by construction (`NORTH-STAR.md` "Clients are equal") — fleet-as-object is explicitly NOT-EVER (`NORTH-STAR.md` "Never"). |
| BU-0065 | The dispatch step/`SERGEANT_AGENT` selects the harness executable and the dispatch step/`SERGEANT_MODEL` pins what that harness runs, and the two are orthogonal, with model precedence `--model` > `SERGEANT_MODEL` > the harness's own ambient default. | not-adopted | Superseded structurally: sergeant-rs already has an adapter-neutral equivalent (`--backend`/`--profile` on `sgt run`, documented in README's Workflows section) — there is no separate harness-executable-selection env var to give a precedence rule for. |
| BU-0078 | The managed coordinator pane is reused across dispatches and runs a reader that displays each line it receives and never executes it, so a tmux-injected notification can never become a shell command in the coordinator's pane. | NOT-EVER | tmux-pane command-injection-safety mechanism; reconstructed tmux-era supervision is explicitly NOT-EVER (`NORTH-STAR.md` "Never"). |
| BU-0106 | Documentation authority is layered by ownership: `AGENTS.md` owns always-on agent execution/safety policy, `skills/*/SKILL.md` and `.agents/skills/*/SKILL.md` own trigger-specific procedures, `docs/schema.md` owns project configuration fields and path resolution, and the rest of this documentation set owns user installation/operating instructions. | AGENTS.md | `AGENTS.md` preamble — documentation authority layering, restated for this repo's actual doc set. |
| BU-0107 | Command `--help` output wins when the command implements it; otherwise the command's emitted usage/error contract and its tests win, and a task is filed when prose disagrees with released behavior. | docs/DEVELOPMENT.md | "Session conduct" — `--help`/tests win over prose; file an issue on mismatch. |
| BU-0108 | Documentation examples must not contain real credentials, private repository names, prompt bodies, response bodies, or secret-bearing environment values. | not-adopted | Belongs to `docs/icm/convention.md`'s workflow/skill authoring rules, out of this lane's file scope (`AGENTS.md`/`CLAUDE.md`-symlink/README only per the MVP-5 Lane F1 task). Flagged so it is not silently lost; a future `convention.md` revision should adopt it explicitly. |
| BU-0109 | Sergeant is designed for one developer per installation; adoption by a larger organization means each developer installs Sergeant independently — it does not turn one installation into a shared team service. | AGENTS.md | `AGENTS.md` opening paragraph / `NORTH-STAR.md` Ownership ("One owner") already encodes single-developer-per-install. |
| BU-0110 | Sergeant does not provide central tenancy, organization RBAC, shared credentials, cross-machine worker leases, or a team-wide fleet database. | not-adopted | Fully covered by `NORTH-STAR.md`'s NOT-EVER list and Ownership section already; redundant with BU-0109's disposition, no new text. |
| BU-0111 | A worker is an agent running in an isolated worktree and tmux pane; a live process is not proof of progress, and recent meaningful progress evidence is required. | AGENTS.md | Duplicate of BU-0047/BU-0038 in substance (liveness ≠ progress); same citation group in step 6. |
| BU-0112 | A decision request is a `needs_input`, `blocked`, or validation ask-user gate that requires a human product, security, privacy, destructive-action, or risk decision; mechanical findings are not human decision requests. | AGENTS.md | Standard workflow loop step 7 — `needs_input` reserved for genuine human-judgment gates. |
| BU-0113 | Direct mode still requires a task, TDD, repository-native checks, independent review, shipping validation, and handoff even though it runs in the current session. | docs/DEVELOPMENT.md | "Session conduct" and `AGENTS.md` "Working on sergeant-rs itself" — in-session mode still requires the full delivery discipline. |
| BU-0114 | Sergeant is not permission to push directly to default branches. | docs/DEVELOPMENT.md | "Session conduct" — never push directly to a default branch. |
| BU-0115 | Sergeant does not make a worker healthy merely because its process exists, does not treat a plan, task, worker launch, or finding as delivered work, and does not authorize validation agents to modify source while reporting findings. | AGENTS.md | Duplicate in substance of BU-0044/BU-0047/BU-0049; same citation group. |
| BU-0116 | Skill provenance is never inferred from a folder name; `.skill-lock.json`, a package lock, plugin metadata, or the source repository is checked instead. | skill: vet-external-skill | Skill provenance verification is that workflow's subject matter (published WORKFLOW per retriage). |
| BU-0117 | The Claude plugin route installs a managed read-only bundle; plugin-owned files are never edited, since updates are not expected to preserve those edits. | not-adopted | No plugin-install mechanism exists in sergeant-rs (skills are repo-local files, not an installable managed bundle); revisit if one is built. |
| BU-0118 | Every directive in a Sergeant-owned skill must contain a trigger, action, prohibition, observable evidence, or stop condition; slogans such as 'be thorough' are replaced with commands, failure behavior, acceptance criteria, ownership, or review evidence. | not-adopted | Belongs to `docs/icm/convention.md`'s skill/workflow authoring quality bar, out of this lane's scope; flagged for a future `convention.md` pass, same reasoning as BU-0108. |
| BU-0120 | Sergeant-owned skills are updated through this repository via a reviewed PR and by running `bash tests/instruction-policy-test.sh` plus the full Sergeant test suite. | docs/DEVELOPMENT.md | "Session conduct" — a change to `.sergeant/workflows/` content still goes through review and the test suites that read it. |
| BU-0121 | `.agents/skills/` is the canonical Agent Skills tree discovered directly by Codex; OpenCode discovers the same tree through `opencode.json`; Claude discovers it through repository-local links in `.claude/skills/`, which resolve only to `.agents/skills/` — no install step writes to a user's global agent configuration. | not-adopted | No `.agents/skills/` vendoring tree exists in sergeant-rs; skills are discovered as repo-local `SKILL.md` files directly via the harness's own native discovery, a different (simpler) mechanism than upstream's multi-harness vendoring tree. |
| BU-0122 | Workers are instructed never to invoke the validation pipeline directly; the validation pipeline skill is vendored only so workers can load and understand the coordinator-owned shipping gate contract when a brief references it. | docs/DEVELOPMENT.md | "Session conduct" — consolidated with BU-0041/BU-1196 (actors/workers never invoke the shipping gate themselves). |
| BU-0130 | Sergeant does not install harness-specific conversation-injection plugins; worker updates are surfaced from durable fleet state through the interactive fleet-watch loop. | NOT-EVER | Same family as BU-0078: harness conversation-injection plugin, tmux-era mechanism, NOT-EVER. |
| BU-0172 | Supported Sergeant commands are used before manual process, tmux, Git, or fleet-file operations, and exact errors and state are preserved before recovery. | AGENTS.md | Folded lightly into "Working on sergeant-rs itself" / doctor framing — use sgt's own surfaces before manual recovery, preserve evidence. |
| BU-0183 | Parsing proof of Bash 3.2 compatibility does not replace runtime proof unless the task acceptance explicitly permits parsing only. | not-adopted | Bash-3.2-compatibility-specific; sergeant-rs is a Rust codebase with no Bash-version-compatibility surface to prove at runtime. |
| BU-0194 | Durable callback implementations are executable profiles under `~/.config/sergeant/callbacks/`; they are not project YAML fields, and fleet requests cannot supply paths. | not-adopted | Callback profile path-injection safety is G3 (callbacks), explicitly gated post-MVP on "a consumer" (`NORTH-STAR.md` Waves); revisit when G3 is built. |
| BU-0259 | Project YAML files never contain credentials, tokens, or secret values. | AGENTS.md | Duplicate of BU-0055 (project/estate config carries paths, never credentials); same citation. |
| BU-0265 | When a required executable is missing, the skill reports the executable and a platform-neutral installation requirement rather than inventing a fallback parser. | AGENTS.md | Duplicate in substance of BU-0049 — `sgt doctor`'s honest-failure convention already is this rule for sergeant-rs. |
| BU-0266 | If the project context-resolution step output and the raw YAML disagree, the project context-resolution step failure is treated as blocking and the YAML is preserved for diagnosis. | not-adopted | Already satisfied structurally: MVP-1's estate manifest parse is #47-style fail-closed (`docs/gauntlet/notes/mvp-bucketing-2026-08-11.md`, MVP-1 row 1) — a manifest/raw-file disagreement already blocks and preserves evidence by construction. |
| BU-0817 | Curated wiki pages never contain raw prompts, response bodies, credentials, tokens, or secrets copied from source material. | not-adopted | Wiki skill: `docs/icm/retriage-2026-08-11.md`'s own verdict recommends parking it ("object is external wiki state, not sergeant's own") — not ported, so BU-0817–BU-0820 have nowhere to land yet. |
| BU-0818 | Task, repository, PR, merge, decision, and blocker facts are preserved into curated pages only when the wiki schema permits them. | not-adopted | Wiki skill parked; see BU-0817. |
| BU-0819 | Automatic wiki captures are owned exclusively by three commands, each for its own event: the dispatch step captures fleet launch/task/project/branch/repository/brief metadata, the notify step captures escalation or terminal outcome plus any PR URL, and the fleet cleanup step captures worktree/fleet cleanup and final status. | not-adopted | Wiki skill parked; see BU-0817. |
| BU-0820 | A missing automatic capture is fixed by reproducing the owning command in a fixture or repairing its capture adapter; it is never fixed by manually synthesizing a capture as a substitute. | not-adopted | Wiki skill parked; see BU-0817. |
| BU-0877 | Sergeant's Bash entry points refuse to continue when the running Bash interpreter is older than 3.2, printing an error to stderr and returning failure instead of proceeding under an unsupported interpreter. | not-adopted | Bash entry-point interpreter-version guard; N/A to a Rust codebase. |
| BU-0891 | A failure while writing a wiki capture document never fails or blocks the Sergeant operation it was documenting; wiki-write failures are silently swallowed. | not-adopted | Wiki skill parked; see BU-0817 (rule has no wiki mechanism to attach to). |
| BU-0898 | The managed coordinator pane runs a reader loop that only echoes every line it receives back out and never executes it, so a tmux-injected notification can never become a shell command running in the coordinator's own pane. | NOT-EVER | Duplicate of BU-0078: tmux pane injection-safety mechanism, NOT-EVER. |
| BU-0942 | A phase of the diagnosing-bugs discipline may be skipped only when there is an explicit justification for skipping it. | skill: diagnose-bug | Phase-skip-needs-justification is that workflow's own discipline (published WORKFLOW per retriage). |
| BU-1001 | In human-facing narration and the map's Decisions-so-far, a map or ticket is referred to by its name (title), never by a bare id, number, or slug — the id and URL still exist but ride inside the name link rather than standing in for it. | skill: wayfinder | Map/ticket/HITL-AFK/fog-of-war rules are that workflow's own subject matter (published WORKFLOW per retriage). |
| BU-1002 | The map is a single issue on the repo's issue tracker labelled `wayfinder:map`, and its tickets are child issues of that map. | skill: wayfinder | See BU-1001. |
| BU-1003 | The map itself only gists a decision and links to it; the map is an index, not a store, so the decision's actual detail lives in exactly one place — its ticket. | skill: wayfinder | See BU-1001. |
| BU-1005 | Open tickets are not listed inline in the map body — they are found by querying open child issues instead, keeping the loaded map view low-resolution. | skill: wayfinder | See BU-1001. |
| BU-1006 | Every ticket carries exactly one `wayfinder:<type>` label from the set research, prototype, grilling, task. | skill: wayfinder | See BU-1001. |
| BU-1008 | Ticket dependencies use the tracker's native dependency relationship (so the frontier renders visually in the tracker's own UI) unless the tracker lacks native blocking, in which case a body convention is the fallback; a ticket is unblocked once every ticket blocking it is closed, and the frontier is the set of open, unblocked, unclaimed children. | skill: wayfinder | See BU-1001. |
| BU-1010 | Every ticket is either HITL — resolvable only through a live exchange with the human, who the agent never stands in for — or AFK, driven by the agent alone; an agent answering its own HITL questions has broken this. | skill: wayfinder | See BU-1001. |
| BU-1015 | The map does not chart what can't yet be seen (the fog of war); resolving a ticket clears the fog ahead of it, graduating whatever becomes specifiable into fresh tickets one at a time. | skill: wayfinder | See BU-1001. |
| BU-1033 | When discussing or designing module boundaries, use the codebase-design glossary terms (module, interface, implementation, depth, seam, adapter, leverage, locality) exactly, rather than substituting generic terms like component, service, API, or boundary. | skill: codebase-design | Interface/seam/deepening/testability vocabulary and rules belong to this skill (and its `DEEPENING.md`), not to always-on repo policy. |
| BU-1034 | When designing an interface, ask whether the number of methods can be reduced, whether the parameters can be simplified, and whether more complexity can be hidden inside. | skill: codebase-design | See BU-1033. |
| BU-1035 | To judge whether a module earns its keep, apply the deletion test: imagine deleting the module — if the complexity it held simply vanishes, it was a pass-through; if that complexity reappears spread across its N callers, the module was earning its keep. | skill: codebase-design | See BU-1033. |
| BU-1036 | Do not introduce a seam unless something actually varies across it: one adapter means the seam is only hypothetical, two adapters means it is real. | skill: codebase-design | See BU-1033. |
| BU-1037 | For testability, a module should accept its dependencies as parameters rather than constructing them internally. | skill: codebase-design | See BU-1033. |
| BU-1038 | For testability, a module should return results rather than produce side effects. | skill: codebase-design | See BU-1033. |
| BU-1039 | Interfaces should be kept to a small surface area, because fewer methods require fewer tests and fewer parameters require simpler test setup. | skill: codebase-design | See BU-1033. |
| BU-1040 | A candidate module whose dependencies are in-process (pure computation, in-memory state, no I/O) is always deepenable: merge the modules and test through the new interface directly, with no adapter needed. | skill: codebase-design | See BU-1033 (`DEEPENING.md`). |
| BU-1041 | A candidate module whose dependency has a local test stand-in (e.g. PGLite for Postgres, an in-memory filesystem) is deepenable if that stand-in exists: the deepened module is tested with the stand-in in the test suite, and the seam stays internal with no port at the module's external interface. | skill: codebase-design | See BU-1040. |
| BU-1042 | A candidate module whose dependency is a remote but owned service (e.g. an internal microservice) is deepened by defining a port at the seam so the deep module owns the logic while the transport is injected as an adapter; tests use an in-memory adapter and production uses an HTTP/gRPC/queue adapter. | skill: codebase-design | See BU-1040. |
| BU-1043 | A candidate module whose dependency is a true external, third-party service (e.g. Stripe, Twilio) is deepened by taking the dependency as an injected port, with tests supplying a mock adapter. | skill: codebase-design | See BU-1040. |
| BU-1044 | Do not introduce a port at a seam unless at least two adapters are justified (typically production and test); a single-adapter seam is just indirection. | skill: codebase-design | See BU-1040. |
| BU-1045 | Internal seams — private to a module's own implementation and used only by its own tests — should not be exposed through the module's external interface just because tests happen to use them. | skill: codebase-design | See BU-1040. |
| BU-1046 | Once tests exist at a deepened module's interface, the old unit tests on the shallow modules it replaced become waste and should be deleted. | skill: codebase-design | See BU-1040. |
| BU-1047 | New tests written for a deepened module are written at its interface and assert on observable outcomes through that interface, not on internal state. | skill: codebase-design | See BU-1040. |
| BU-1048 | A test that must change when a module's implementation changes without any corresponding interface change is a signal that the test is testing past the interface rather than describing behavior. | skill: codebase-design | See BU-1040. |
| BU-1064 | CONTEXT.md is restricted to glossary content: it must not be treated as a spec, a scratch pad, or a repository for implementation decisions, and must stay devoid of implementation details. | skill: domain-modeling | That skill's own file-role rule; distinct from — and not colliding with — the unrelated per-workflow-stage `CONTEXT.md` convention in `docs/icm/convention.md` (different files, same name, different owners, noted to avoid confusion). |
| BU-1080 | Prototype code is located close to where it will actually be used, but named so a casual reader can tell it is a prototype, not production, and throwaway UI routes follow the project's existing routing convention rather than inventing a new top-level structure. | skill: prototype | Prototype-shape rules belong to that workflow (published WORKFLOW per retriage). |
| BU-1081 | A prototype must be runnable with one command, using whatever task runner the project already supports, so the user can start it without having to think about how. | skill: prototype | See BU-1080. |
| BU-1082 | A prototype has no persistence by default (state lives in memory); if the question explicitly involves a database, the prototype hits a scratch DB or local file with a clear "PROTOTYPE — wipe me" name rather than a real data store. | skill: prototype | See BU-1080. |
| BU-1083 | A prototype skips polish: no tests, no error handling beyond what's needed to make it runnable, and no abstractions, because the point is to learn something fast. | skill: prototype | See BU-1080. |
| BU-1084 | A prototype surfaces its full relevant state after every action (logic branch) or on every variant switch (UI branch), so the user can see what changed. | skill: prototype | See BU-1080. |
| BU-1126 | This skill's sections on what a good test is, where tests go, the anti-patterns, and the rules of the loop are consulted before and during every TDD cycle, not only afterward. | skill: tdd | Good/bad-test criteria, seams, and mocking rules (incl. `mocking.md`) belong to that workflow (published WORKFLOW per retriage). |
| BU-1128 | A good test verifies behavior through public interfaces rather than implementation details, reads like a specification of a capability, and survives refactors because it does not depend on internal structure. | skill: tdd | See BU-1126. |
| BU-1129 | A test lives at a seam — the public boundary where behavior is observed without reaching inside — and never against internals. | skill: tdd | See BU-1126. |
| BU-1131 | An implementation-coupled test — one that mocks internal collaborators, tests private methods, or verifies through a side channel like querying the database instead of the interface — is an anti-pattern, tellingly breaking on refactors that don't change behavior. | skill: tdd | See BU-1126. |
| BU-1132 | A tautological test — one whose expected value is recomputed the same way the code computes it, so it passes by construction — is an anti-pattern; expected values must instead come from an independent source of truth such as a known-good literal, a worked example, or the spec. | skill: tdd | See BU-1126. |
| BU-1137 | Mocking is used only at system boundaries: external APIs, databases (sometimes, a test DB is preferred), time/randomness, and the filesystem (sometimes). | skill: tdd | See BU-1126 (`mocking.md`). |
| BU-1138 | Your own classes/modules, internal collaborators, and anything you control are never mocked. | skill: tdd | See BU-1137. |
| BU-1139 | For mockability, external dependencies at system boundaries are passed into a function/module (dependency injection) rather than being constructed internally by it. | skill: tdd | See BU-1137. |
| BU-1140 | For mockability, SDK-style interfaces (a specific function per external operation) are preferred over one generic fetcher with conditional logic, because each mock then returns one specific shape, test setup needs no conditional logic, it's easier to see which endpoints a test exercises, and type safety is per-endpoint. | skill: tdd | See BU-1137. |
| BU-1141 | A good test exhibits five characteristics together: it tests behavior users/callers care about, uses only the public API, survives internal refactors, describes WHAT rather than HOW, and makes one logical assertion. | skill: tdd | See BU-1126. |
| BU-1142 | A bad, implementation-detail test is recognized by any of six red flags: mocking internal collaborators, testing private methods, asserting on call counts/order, breaking on refactors without a behavior change, a name that describes HOW rather than WHAT, or verifying through external means instead of the interface (e.g. querying the database directly rather than using the createUser/getUser interface). | skill: tdd | See BU-1126. |
| BU-1143 | When the subject repository treats external pull requests as a request surface, triage handles a PR through the same category/state roles and the same state machine as an issue, with only a small set of PR-specific deltas. | skill: triage | PR-as-issue deltas, AI-disclaimer prefix, and one-category/one-state-role rules belong to that workflow (published WORKFLOW per retriage). |
| BU-1145 | Every comment or issue the triage skill posts to the issue tracker must begin with a disclaimer stating it was generated by AI during triage. | skill: triage | See BU-1143. |
| BU-1146 | Every triaged issue carries exactly one category role and exactly one state role. | skill: triage | See BU-1143. |
| BU-1196 | Workers and remediation loops must never invoke the validation pipeline; the Sergeant coordinator alone owns every validation pipeline gate. | docs/DEVELOPMENT.md | "Session conduct" — consolidated with BU-0041/BU-0122. |
| BU-1260 | The setup skill orchestrates only supported Sergeant and bootstrap commands and must not substitute undocumented workarounds for a capability that is missing. | AGENTS.md | Consolidated into the BU-0049 family (guardrails: no undocumented workarounds for a missing capability). |
| BU-1261 | Missing capabilities encountered during setup are surfaced as separate task tracker issues rather than worked around. | AGENTS.md | Consolidated into the BU-0049 family, translated: sergeant-rs has no task-tracker step in this loop, so the gap is named via `sgt doctor`'s remedy instead of a filed ticket. |
| BU-1262 | This skill must not be loaded when the user wants documentation only or is asking about a specific command; `sergeant-help` is used instead in both cases. | AGENTS.md | Routing table — doc/help questions route to `sergeant-help`; setup/repair routes to `sgt init`/`sgt doctor`, not a skill. |
| BU-1263 | This skill writes only to Sergeant-owned paths: `~/.config/sergeant/config.yaml` (global config) and `~/.config/sergeant/<project>.yaml` (project YAML files). | AGENTS.md | Guardrails — `sgt init`/`repo add`/`group add` write only within the estate they scaffold. |
| BU-1264 | This skill must never write to opencode's config, Claude's config or `CLAUDE.md` or any `.claude/` directory, Codex's config, Goose's config, any repository's `AGENTS.md`/`.github/`/other agent configuration paths, or any path outside `~/.config/sergeant/` the user has not explicitly named. | AGENTS.md | Guardrails — explicit: `sgt` never writes `AGENTS.md`/`CLAUDE.md` or another harness's own config. Directly relevant to the `CLAUDE.md`-symlink ruling this same lane executes. |
| BU-1265 | The skill does not automatically initialize the task tracker, Graphify, or Treehouse; each requires an explicit confirmation prompt before any command runs, and if consent is declined the skill leaves state unchanged and reports what was skipped. | not-adopted | No optional subsystems analogous to Graphify/Treehouse exist in sergeant-rs; nothing to consent-gate. |
| BU-1295 | Re-running this skill after a successful setup must produce the same final state: each phase skips steps that already pass verification, and no phase destroys existing working configuration to reach the same end state. | AGENTS.md | Guardrails — `sgt init` is idempotent on a second run, matching MVP-1's atomic-write/pin-at-bind estate manifest design. |
| BU-1297 | Prefer vertical slices that produce independently verifiable behavior when drafting tickets. | skill: to-tickets | Ticket-sizing, ownership, cross-repo counterpart, and epic rules belong to that workflow (published WORKFLOW per retriage). |
| BU-1298 | Keep each ticket small enough for one fresh agent context. | skill: to-tickets | See BU-1297. |
| BU-1299 | Assign exactly one owning repository to each implementation ticket. | skill: to-tickets | See BU-1297. |
| BU-1300 | Represent cross-repository delivery with counterpart tickets and explicit merge order, not one ambiguous shared ticket. | skill: to-tickets | See BU-1297. |
| BU-1301 | Use expand-migrate-contract for mechanical changes that cannot remain green as a vertical slice. | skill: to-tickets | See BU-1297. |
| BU-1302 | Create epics for coherent programs of work, not as substitutes for executable tickets. | skill: to-tickets | See BU-1297. |
| BU-1303 | Never duplicate an existing task tracker task or GitHub issue. | skill: to-tickets | See BU-1297. |
| BU-1304 | Preserve stable finding IDs such as `RBAC-P1-004` or `DATA-P0-002` in ticket titles. | skill: to-tickets | See BU-1297. |
| BU-1305 | A ticket is not ready unless its acceptance criteria are observable and its blockers are accurate. | skill: to-tickets | See BU-1297. |
| BU-1311 | Do not automatically add the task tracker instructions to repository guidance files. | AGENTS.md | `AGENTS.md` preamble — this file is hand-authored; no workflow/skill auto-appends to it. Directly informed this lane's own execution. |

## What this pass did not do

This lane (MVP-5 Lane F1: `AGENTS.md` rewrite, `CLAUDE.md → AGENTS.md`
symlink, README recenter) did not edit any `.sergeant/workflows/` package —
the 55 `skill:`-dispositioned rows above are a placement judgment, not a
content change to those workflows. Library re-homing execution (moving/
editing the actual workflow packages per `docs/icm/retriage-2026-08-11.md`'s
verdicts, including R-NS-6's `grilling`/`grill-with-docs` self-declaration
fix) is separate MVP-5 content-lane work. The two `docs/icm/convention.md`
gaps this pass surfaced but did not close (BU-0108, BU-0118 — documentation
example hygiene and skill-directive quality bar) are left as named
follow-ups rather than silently absorbed.
