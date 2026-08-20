Estate Root, Repository Ownership, and Git Work-Surface Contract

Status: Proposed — owner interview completed 2026-08-19
Date: 2026-08-19
Audit basis: miztertea/sergeant-rs main @ d39b025
Scope: Estate identity, daemon binding, repository topology, Work admission,
Git base selection, linked-worktree ownership, integrity reporting, branch
lifecycle, diagnostics, and always-on Captain doctrine
Product behavior: Changed — the current upward-discovery / zero-config
workspace model is replaced by an exact-root estate contract; repository mounts
become estate-owned base checkouts; every durable Work receives an explicit,
pinned Git surface

────────

Relationship to existing decisions and defects

This proposal is a correction to the core execution model, not a new
orchestration layer. It keeps the journal, Work state machine, workflow engine,
backend boundary, and one-surface-per-Work design. It replaces the ambient Git
and directory assumptions beneath them.

|Existing decision or defect                                                                |Disposition here                                                                                                                                                                                                                                                                    |
|-------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|`NORTH-STAR.md`: Sergeant runs intents in “isolated worktrees”                             |**Clarified and made mechanically honest.** A Work owns linked worktrees and output refs for the repositories explicitly selected for it. This is a mutation and output-attribution boundary, not a promise that every backend hermetically prevents reads elsewhere on the machine.|
|`NORTH-STAR.md`: `repos/` is a mount, never a development root                             |**Made enforceable.** Every mounted checkout is an estate-owned base checkout at `repos/<name>`. Workers execute only in Work-owned linked worktrees.                                                                                                                               |
|R-MVP1-12 / `Workspace::discover_scoped`: upward estate discovery across Git boundaries    |**Superseded.** Estate-scoped commands operate only when the current directory itself contains a valid `sergeant.toml`. No ancestor walk occurs.                                                                                                                                    |
|Zero-configuration single-repository workspace fallback                                    |**Removed.** A one-repository installation is an estate with one declared repository.                                                                                                                                                                                               |
|`--workspace` and `Work.workspace`                                                         |**Removed/replaced.** The daemon is bound to one exact estate root; Work records carry that estate context and an explicit repository scope rather than a client-provided workspace label.                                                                                          |
|Existing `[group.<name>]`                                                                  |**Preserved as the minimal composition primitive.** A group remains repository membership plus an explanatory brief. It gains no workflow, model, profile, branch, or execution behavior.                                                                                           |
|Issue #180: worktree isolation is only a starting directory                                |**Primary defect addressed.** Core defines a Work mutation surface, adapters receive the full binding contract, and retirement reports violations honestly. Universal OS sandboxing remains a non-goal.                                                                             |
|Issue #173: Works commit to self-named or caller integration branches                      |**Addressed.** Actual branch/HEAD state is reconciled against the bound Work branch; mismatch becomes terminal dirty evidence instead of a silently false output pointer.                                                                                                           |
|Issue #172: `sergeant/<ULID>` refs with no matching Work                                   |**Investigative tooling enabled, not pre-judged.** Repository ownership, common-directory identity, journal/ref reconciliation, and exact branch provenance make the source of unknown refs measurable.                                                                             |
|Issue #159: durable branches and stale worktree registrations accumulate without visibility|**Partially absorbed.** Durable branches remain intentional; `sgt doctor` gains a cheap count/size summary. Expensive ancestry classification and explicit deletion remain a separate maintenance action.                                                                           |
|Issue #94: Work may complete although the declared commit never happened                   |**Made visible, not redefined by core.** Core reports branch advancement and dirty state. A workflow that requires a commit must explicitly require/check it; core does not invent workflow completion semantics.                                                                   |
|Issue #112: GitHub identity cannot be inferred from local-path remotes                     |**Narrowed.** Estate-owned clones preserve declared origins and repository identity. GitHub-specific convenience remains environment/adapter behavior, not core repository identity.                                                                                                |
|Issue #120: an external gate cached the wrong default branch                               |**Not claimed closed.** Sergeant pins an explicit base ref/SHA and exposes it to workflows, removing the need for downstream tools to infer the Work base from ambient checkout state. External tools must still honor it.                                                          |
|ADR 0006 harness passthrough                                                               |**Tightened.** `sgt claude`, `sgt codex`, `sgt opencode`, and `sgt goose` launch only from the exact estate root and bind that root into the inherited environment before `exec`.                                                                                                   |
|ADR 0009 no-spawn rule                                                                     |**Preserved.** Exact-root validation happens before descriptor lookup or auto-spawn. Observation still never materializes a daemon.                                                                                                                                                 |

────────

1. Executive summary

The current linked-worktree implementation is careful. It records a
materialization plan before mutating Git, pins an exact base SHA, serializes
some registry mutations, avoids detached HEAD during checkout, rolls back
partial multi-repository materialization, captures dirty patches, and retains
Work branches as durable output.

The missing piece is not another Git command. It is authority.

Today, a client sends its current directory, the daemon searches upward for an
estate or falls back to the nearest Git repository, repository selection may
silently expand to every repository, the mounted checkout’s ambient HEAD
becomes the base, and the backend receives one starting directory. Sergeant
then inspects the branch it expected at teardown, even if the actor switched
branches or committed elsewhere.

That is a provenance record for the setup Sergeant attempted. It is not a
complete contract for the Git state a Work was authorized to modify or the
output it actually produced.

This proposal establishes that contract:

1. An estate is exactly the current directory containing
./sergeant.toml. Every estate-scoped command fails before daemon
discovery when invoked anywhere else. No upward search and no Git fallback
remain.
2. A daemon belongs to one estate root. The runtime descriptor records and
verifies that root. Request cwd is evidence, never authority.
3. Every repository is owned by one estate and mounted at
repos/<name>. Arbitrary repository paths disappear from the manifest.
4. Captain explicitly scopes multi-repository Work. Omission never means
“all repositories.” A one-repository estate may infer its only repository;
a larger estate requires --repo, --group, an explicit --all, or a
named run template defined by the companion proposal.
5. Core performs a complete Git preflight. The normal base is each
selected mount’s clean, attached current branch at its exact HEAD SHA.
Sergeant does not fetch, pull, switch branches, or infer a remote default.
6. One Work owns one surface for its entire workflow. Every selected
repository receives a linked worktree on sergeant/<work-id>; every stage
is a fresh execution over those same Work-owned branches.
7. Repository selection is a mutation/output boundary, not a security
sandbox. The actor runs as the user and may read what that user can read.
Sergeant does not broker credentials, device auth, network policy, or tool
access.
8. Integrity is reported honestly at retirement. Core never injects an
implicit workflow checkpoint. A terminal Work is independently clean or
dirty; branch mismatch, assigned-surface dirt, and attributable escaped Git
mutations become durable findings.
9. Work branches remain durable. sergeant/<work-id> is never deleted
automatically. sgt doctor cheaply reports worktree, retained-artifact,
branch, and disk totals; explicit cleanup remains separate.

Once these invariants hold, the core execution loop has a stable boundary:
Captain shapes intent and selects scope, Sergeant constructs and guards the
surface, workflow stages execute, adapters translate harness behavior, and the
TUI renders journaled truth.

────────

2. Diagnosis

2.1 The client working directory currently chooses too much

The current CLI places its process cwd in request origin metadata. The engine
uses that path to discover a sergeant.toml, searching ancestors across Git
boundaries, and falls back to git rev-parse --show-toplevel when it finds no
estate.

That means a directory-navigation mistake can change all of these at once:

• which estate is selected;
• which data directory and daemon are selected;
• which repositories exist;
• which group/profile/default definitions exist;
• and which Git checkout becomes the Work source.

A Work launched from a linked worktree can therefore collapse from a
multi-repository estate into a zero-config single-repository workspace. A
Captain launched in one estate can cd repos/<name> and accidentally submit
against another interpretation of the same filesystem.

The tool should not compensate for that mistake. It should name it.

2.2 repos/<name> is an ordinary clone with ambient authority

The mounted checkout currently acts simultaneously as:

• repository identity;
• current integration view;
• source of the Work base;
• owner of local refs and the linked-worktree registry;
• source of remote/upstream inference;
• a path Captain and actors can navigate into;
• and a mutable checkout whose current branch may change independently.

An ordinary checkout is not inherently wrong. The missing rule is that its
role must be narrow and explicit:

> `repos/<name>` is the estate-owned **base checkout**. Captain prepares its
> branch and `HEAD`; Sergeant snapshots that committed state; workers never use
> the mount as their work surface.

2.3 Work bases are ambient rather than admitted

surface::materialize_one reads rev-parse HEAD and the current symbolic
branch from the mounted checkout. It carefully passes the exact SHA to
git worktree add, so the recorded and materialized commits agree.

What it does not decide is whether that checkout state is suitable:

• dirty working tree or index;
• detached HEAD;
• unintended integration branch;
• linked worktree admitted as a source repository;
• a Git common directory managed by a different process/estate;
• or a repository path that does not match the declared mount.

Sergeant currently records whatever it finds. The corrected model admits only
a state it can prove coherent, with one narrowly bounded override for dirty or
detached state where an exact commit still exists.

2.4 Repository selection is not an authority boundary

Today an empty selection means all repositories. In a large estate that turns
an underspecified intent into writable branches everywhere.

The opposite workaround is equally bad: select one repository, then let the
actor wander into repos/ to inspect or modify another. The Work record can no
longer explain the result.

The corrected rule is explicit:

> The selected repositories are the complete set Sergeant authorizes this
> Work to modify, materializes branches for, validates, tears down, and reports
> as output.

The actor may still read outside them because the process runs as the user.
That is an environmental security choice, not a universal Sergeant contract.

2.5 A worktree is presently a starting directory, not a guarded result

For a single repository, the backend receives that linked worktree as cwd.
For several repositories it receives the surface root. The native Claude
adapter then applies .current_dir(&cwd).

An actor may still:

• switch or create a branch inside the assigned linked worktree;
• use an absolute path into repos/;
• modify an unselected repository;
• work in another surface;
• or use an unrelated checkout elsewhere on the machine.

Universal prevention would require per-harness sandbox behavior, tool/network
allowlists, credential mediation, and operating-system policy. That would
constrain user workflows and adapter support while duplicating controls users
already establish through OS accounts, containers, device-auth flows, Doppler,
and other environment choices.

Core instead owns the invariant it can honestly keep: it constructs and
records the mutation surface, validates the assigned Git state, consumes
adapter causality where available, and never certifies a contradictory output
as clean.

2.6 Teardown reads the expected branch, not necessarily the actual branch

The existing teardown resolves refs/heads/<binding.work_branch> and checks
whether the recorded worktree is dirty. It does not first prove that the
worktree still has that branch checked out.

A clean actor-created branch can therefore contain the implementation while
sergeant/<work-id> remains at the base SHA. Teardown may remove the worktree
and point the Work output at the unchanged expected ref. Issue #173 measured
this exact class of behavior.

The corrected retirement path records both sides:

```text
expected work branch and SHA
actual symbolic branch (or detached HEAD) and SHA
worktree status
attributable outside-surface operations
unattributed estate drift
```

Mismatch makes the terminal integrity dirty. It does not retroactively change
the workflow’s stage result or inject a new checkpoint.

2.7 Repository locking uses checkout path rather than Git common identity

The current per-repository mutex is a process-local table keyed by the
canonical source checkout path. Different linked worktree paths may share one
Git common directory and therefore one ref/worktree registry while receiving
different locks. Separate processes receive no shared lock at all.

The corrected lock key is the canonical result of:

```bash
git rev-parse --path-format=absolute --git-common-dir
```

Registry-mutating operations use an interprocess lock associated with that
common directory. In-process locking may remain as a fast local layer, but it
is no longer the authority.

────────

3. Product model and fixed vocabulary

3.1 Estate

An Estate is a directory initialized by sgt init whose current directory
contains a valid ./sergeant.toml.

It may be a plain directory or a Git repository. If the estate root itself is
a Git repository, Sergeant still does not treat it as an implicit Work
repository. Only declared mounts under repos/ are targetable.

Users may organize estates however they choose:

```text
~/estates/payments/
~/estates/platform/
~/estates/personal/
```

or place every repository they use in one estate. Sergeant does not prescribe
team, product, project, or domain boundaries.

3.2 Repository mount

A Repository Mount is an estate-owned ordinary Git checkout at:

```text
<estate-root>/repos/<repository-name>
```

It is the prepared base view from which Sergeant snapshots Work bindings.
Workers never use it as their execution checkout.

3.3 Group

A Group remains a minimal composition:

```toml
[group.payments]
repos = [
  "payments-api",
  "auth",
  "ledger",
  "payments-knowledge",
]
brief = "Repositories that participate in payment authorization and settlement."
```

It answers only which mounted repositories commonly belong together and why.
A repository may belong to several groups. A knowledge repository is not a
special repository type; selecting it gives it the same Work branch and
mutation authority as any other selected repository.

3.4 Work scope

A Work Scope is the explicitly resolved repository set for one Work. It is
the complete set for which Sergeant creates branches/worktrees and accepts
mutation/output responsibility.

3.5 Repository binding

A Repository Binding is the immutable admission record for one selected
repository:

```text
repository name
mount path
canonical Git top level
canonical Git common directory
observed base branch, if attached
exact base SHA
assigned Work branch
assigned linked-worktree path
preflight result and any authorized override evidence
```

3.6 Work surface

A Work Surface is one directory per Work containing one linked worktree per
selected repository. It belongs to the Work, not to a stage.

3.7 Integrity disposition

A terminal Work has its ordinary WorkState and a separate Integrity
Disposition:

```text
clean
dirty
```

This is not a new Work state and does not move the workflow. It is the honest
assessment of whether the Git result agrees with the Work bindings.

────────

4. Exact-root estate admission

4.1 No ancestor search

Every estate-scoped command begins with one deterministic check:

```text
current working directory / "sergeant.toml"
```

The file must exist, parse, contain [estate], and satisfy the manifest
schema. Sergeant does not search parents and does not use Git to infer an
estate.

These two commands intentionally differ:

```bash
cd ~/estates/payments
sgt run "fix duplicate authorization"       # valid

cd ~/estates/payments/repos/payments-api
sgt run "fix duplicate authorization"       # refused
```

4.2 Commands requiring the estate root

The exact-root rule applies to all estate-scoped commands, including:

```text
sgt run
sgt status
sgt work ...
sgt respond / retry / extend / cancel
sgt watch
sgt analytics
sgt tui
sgt daemon / daemon stop
sgt repo ...
sgt group ...
sgt workflow ...
sgt template ...          (companion proposal)
sgt claude / codex / opencode / goose
```

Commands usable outside an estate are intentionally small:

```text
sgt                    static homepage
sgt --help
sgt --version
sgt init
sgt doctor
```

sgt doctor never searches upward. Outside an estate it reports installation
health and a failing estate-root row with the remedy to cd or initialize.
It does not start a daemon.

4.3 Failure happens before daemon interaction

Root validation precedes:

• data-directory resolution from the estate manifest;
• runtime-descriptor lookup;
• daemon auto-spawn;
• API calls;
• repository inspection;
• harness preparation or exec.

A directory mistake therefore cannot attach to or spawn the wrong daemon.

4.4 Loud corrective diagnostics

No manifest in the current directory:

```text
sgt: no estate found in the current directory

Expected:
  /current/directory/sergeant.toml

Sergeant does not search parent directories for an estate. This prevents a
Captain session or Work from silently attaching to the wrong environment.

Are you in the intended estate root?
  cd <estate-root>

If this directory should become a new estate:
  sgt init
```

A valid estate exists in a known bound environment but Captain has navigated
into a descendant:

```text
sgt: this command must be run from the estate root

Current directory:
  /estates/payments/repos/payments-api

Bound estate root:
  /estates/payments

Return to the root and retry:
  cd /estates/payments
```

An invalid manifest surfaces the exact parser diagnostic and never falls
through to another estate.

────────

5. Estate-bound daemon identity

5.1 One daemon, one estate

Daemon startup receives a canonical estate root. Its runtime descriptor gains:

```json
{
  "estate_root": "/absolute/canonical/estate",
  "manifest_path": "/absolute/canonical/estate/sergeant.toml"
}
```

A client validates that the descriptor’s root matches its exact current root
before using the endpoint. A live daemon bound to another estate is a named
refusal, never a reusable global service.

5.2 Request cwd loses authority

Origin metadata may retain cwd for evidence and debugging, but neither the
API nor engine uses it to discover topology. The daemon loads the estate it was
started against.

This removes the current recursion hazard where a child command launched from
a Work surface rediscovers that linked worktree as a new zero-config
workspace.

5.3 Harness binding

sgt <harness> validates the exact root, resolves the estate data directory,
and exports at least:

```text
SGT_ESTATE_ROOT=<canonical estate root>
SGT_DATA_DIR=<resolved data directory>
SGT_ORIGIN_CLIENT=<harness name>
```

It then execs the harness as today. The environment helps diagnostics and
later invocations name the correct root, but it does not waive the exact-current-
directory check. If Captain cds into a mount and calls sgt run, the tool
still refuses and tells Captain to return.

────────

6. Estate-owned repository topology

6.1 Fixed mount paths

Repository path is derived, not configured:

```text
<estate-root>/repos/<name>
```

The path field is removed from [[repo]].

```toml
[estate]
name = "payments"

[[repo]]
name = "payments-api"
origin = "git@github.com:company/payments-api.git"
instructions = "suppress"

[[repo]]
name = "auth"
origin = "git@github.com:company/auth.git"
```

sgt repo add remains the validating writer and clones/verifies exactly that
path. sgt repo remove undeclares without deleting the checkout.

6.2 One checkout belongs to one estate

Separate estates use separate clones, even for the same upstream repository.
The product does not support manifest aliases into another estate, ../repo
paths, symlinked shared mounts, or declaring a Work’s linked worktree as a
repository source.

This deliberately trades shared-object optimization for clear ownership of:

• local branches;
• the Git common directory;
• linked-worktree registry entries;
• sergeant/* refs;
• and the prepared base checkout.

Git alternates or future storage optimization may reduce disk use later if
measured need earns it; they do not weaken logical ownership.

6.3 The mount may advance while Works run

A Work pins its base SHA. Captain or another authorized process may later
advance or switch the mounted checkout. That does not alter an existing Work
and does not automatically make it dirty.

The binding remains:

```text
base branch observed at admission
base SHA used to materialize
```

Later mount movement is estate activity. It is attributed to a Work only when
direct execution evidence establishes causality.

6.4 No automatic Git synchronization

Sergeant does not fetch, pull, reset, rebase, switch to main, or infer a
remote default during Work admission.

Captain prepares the mount using ordinary Git and leaves it on the intended
committed base. If it is stale, that is the base Captain selected. If a workflow
needs remote synchronization, that procedure must be explicit and separately
authorized.

────────

7. Explicit Work scope

7.1 Multi-repository scope is mandatory

A one-repository estate may infer its sole declared repository when no scope is
provided.

An estate with two or more repositories requires one of:

```text
--repo <name>       repeatable
--group <name>      may be combined with explicit --repo additions
--all               explicit entire-estate selection
--template <name>   only when that explicitly selected run template supplies scope
```

Omission is a refusal. It never expands to all repositories.

```text
sgt: this estate contains 14 repositories, but no Work scope was selected

Sergeant will not expose the entire estate to a worker by default.

Select repositories:
  sgt run "..." --repo payments-api --repo auth

Select a declared group:
  sgt run "..." --group payments

Intentionally select the complete estate:
  sgt run "..." --all
```

7.2 Core resolves groups

Group expansion moves out of CLI-only behavior and into estate/core request
resolution. The API accepts the scope request and the daemon resolves it
against its bound manifest.

This preserves the North Star rule that a client surface adds usability,
never functionality, and allows the TUI or another client to submit the same
request without reimplementing manifest semantics.

7.3 Scope is journaled twice

The Work records:

1. the request form Captain chose (repos, group, all, or template); and
2. the exact resolved repository list used for materialization.

A later manifest edit therefore cannot rewrite the meaning of an existing
Work.

7.4 --workspace is removed

The daemon is already bound to an exact estate. A free-form workspace label no
longer has a role in submission or Work identity.

────────

8. Git admission contract

8.1 Normal preflight

Before work.submitted or any Git mutation, core validates every selected
repository:

1. The derived mount path exists.
2. The path is not a symlink or alias outside the estate-owned mount.
3. git rev-parse --show-toplevel resolves exactly to the canonical mount.
4. The mount is an ordinary primary checkout, not a linked worktree admitted
as a repository source.
5. The canonical Git common directory is resolvable and lockable.
6. HEAD is attached to a named local branch.
7. HEAD resolves to a full commit SHA.
8. git status --porcelain is empty for index and working tree.
9. sergeant/<work-id> does not already exist in that repository.
10. The planned linked-worktree path is absent and not registered.
11. All selected repositories can be planned before the first materialization
side effect occurs.

Any unresolved fact refuses the submission before a Work record exists.

8.2 Default base

The normal base is exactly:

```text
base_branch = mounted checkout's current local branch
base_sha    = mounted checkout's exact HEAD
```

Both are pinned in the binding. Sergeant does not judge an integration branch
as less valid than main.

8.3 One bounded override

sgt run gains one explicit flag:

```text
--override-git-preflight
```

The name is intentionally specific. It is not a general “make Sergeant do it
anyway” switch.

It may waive only these normal cautions when an exact commit can still be
pinned:

|Condition                   |Override behavior                                                                                                                  |
|----------------------------|-----------------------------------------------------------------------------------------------------------------------------------|
|Mounted checkout is dirty   |Pin committed `HEAD`; record full porcelain evidence and state explicitly that uncommitted changes are excluded from the Work base.|
|Mounted checkout is detached|Pin exact `HEAD`; record no named base branch and the operator-authorized override.                                                |

It may never bypass:

• missing or invalid exact-root estate;
• unresolved or undeclared scope;
• unknown repository;
• wrong/aliased repository path;
• unresolvable Git top level/common directory/commit;
• ownership or lock conflict;
• existing Work ref/path collision;
• partial or failed surface construction;
• backend/workflow/profile validation failures.

The principle is:

> Override may waive policy caution. It may not replace a missing fact or
> overcome mechanical impossibility.

The override and every waived finding are journaled with the Work. It is never
available from run defaults or a run template; the operator must type it for
that submission.

8.4 Preflight is core-owned

AGENTS.md teaches Captain what state to prepare and reconcile, but correctness
does not depend on Captain remembering a checklist. The API/daemon repeats and
authoritatively enforces the mechanical contract.

────────

9. Surface construction and locking

9.1 One branch per Work per selected repository

For Work 01ABC over three repositories:

```text
.sergeant/data/surfaces/01ABC/
├── payments-api/         branch sergeant/01ABC
├── auth/                 branch sergeant/01ABC
└── payments-knowledge/   branch sergeant/01ABC
```

The same branch name appears in separate Git repositories and therefore does
not collide.

9.2 Exact materialization

For each binding, Sergeant creates:

```bash
git worktree add --no-checkout \
  -b sergeant/<work-id> \
  <worktree-path> \
  <pinned-base-sha>

git -C <worktree-path> reset --hard HEAD
```

The existing journal-before-effect and partial-materialization rollback rules
remain.

9.3 Surface belongs to the Work

The surface is materialized once per Work and persists across the workflow.
Each stage is a fresh execution but receives the same Work-owned surface.
Stages never mint stage-level branches or worktrees.

Retry rematerializes the same Work branches as today.

9.4 Lock by Git common directory

Every operation mutating the linked-worktree registry or Work refs uses the
canonical Git common directory as its lock identity.

The lock is interprocess. At minimum it covers:

```text
git worktree add
git worktree remove
git worktree prune
Work-ref creation/deletion performed by Sergeant
```

The implementation may retain an in-process mutex for efficiency, but the
filesystem lock is the correctness boundary.

9.5 Mounted checkout remains available

Sergeant does not hold the repository lock for the lifetime of a Work. It holds
it only across the narrow Git registry/ref operations that require exclusive
mutation. Captain may inspect or advance the mount after Work admission; the
Work remains pinned to its base SHA.

────────

10. Worker execution contract

10.1 Selected repositories are the mutation surface

The worker is authorized to modify exactly the linked worktrees in its Work
surface.

Outside that mutation authority are:

```text
estate root
estate repos/ mounts
unselected repositories
other Work surfaces
Sergeant runtime state
unrelated checkouts elsewhere on the machine
```

The backend receives the complete binding summary, not only a cwd, so its
prompt/launch grammar can state exact repository paths, expected branches, and
base SHAs.

10.2 Not a read sandbox

The worker still runs as the invoking user. Sergeant does not attempt to define
or reproduce that user’s security environment.

Core does not own:

• filesystem read allowlists;
• network policy;
• package caches or registries;
• compiler/toolchain exposure;
• Docker or service sockets;
• device-auth flows;
• Doppler, password managers, or secret stores;
• commercial harness credentials.

A user wanting stronger separation uses OS users, containers, VMs, separate
machines, or separate estate folders/checkouts as appropriate.

An adapter may expose or enforce stronger write restrictions when its harness
supports them, but core correctness and adapter admission do not depend on a
universal sandbox capability.

10.3 Captain captains

The estate-root harness session is the Captain surface. Captain:

• reads the estate map;
• reconciles existing Work;
• shapes intent;
• selects repository scope, workflow, and launch options;
• dispatches;
• watches and adjudicates results.

Meaningful repository implementation is dispatched to Work. Captain does not
become a concurrent coding worker in repos/.

Because every estate-scoped sgt command requires the exact root, a worker in
a linked worktree cannot silently become a nested Captain by invoking
sgt run there. Workflow composition belongs to the workflow engine/content,
not to a stage actor rediscovering and dispatching an estate from its surface.

────────

11. Integrity observation and terminal dirtiness

11.1 No implicit workflow checkpoint

Core does not pause a workflow because a stage or surface becomes dirty. It
does not insert a hidden reconciliation stage before push, PR creation, or any
other consequential action.

A workflow may explicitly invoke a deterministic surface-integrity check and
define its own response (needs_input, remediation, stop, or continue). That
is workflow procedure using a core mechanic, not universal engine policy.

11.2 Observation points

Core inspects the binding at least:

• immediately after materialization;
• before launching or resuming an execution;
• at stage/execution settlement where inspection is already available without
blocking under the core lock;
• and during final Work retirement.

Nonterminal observations accumulate evidence. They do not move Work or stage
state on their own.

11.3 Directly attributable dirty findings

The following are attributable to the Work because they occur in its assigned
surface or are reported by its execution:

```text
assigned_worktree_uncommitted
assigned_worktree_missing
assigned_branch_mismatch
assigned_head_detached_or_unreferenced
assigned_common_dir_mismatch
adapter_observed_outside_surface_git_command
backend_reported_outside_surface_write
```

The exact vocabulary is a closed serialized enum, not free-form prose.
Each finding records expected and observed paths/refs/SHAs plus evidence.

11.4 Estate drift is separate

A mount or unselected repository changing during the Work window is not enough
to attribute that mutation to the Work. Captain, an editor, another Work, or
another process may have changed it.

Before/after differences without direct causality are reported as estate drift:

```text
repository: auth
before: abc123
observed later: def456
attribution: unknown
```

Estate drift does not make the Work dirty by itself.

Direct adapter tool evidence may promote the same observation into an
attributable dirty finding.

11.5 Terminal shape

Work state remains unchanged:

```text
completed
failed
canceled
```

Integrity is orthogonal:

```text
completed / clean
completed / dirty
failed / clean
failed / dirty
canceled / clean
canceled / dirty
```

The CLI and TUI may render completed_dirty as a compact label, but the domain
model does not add it as a state transition target.

11.6 Branch advancement is a fact, not universal acceptance

A Work branch remaining at its base SHA is reported. It is not inherently
dirty: a research, validation, or no-op workflow may correctly produce no
commit.

A workflow whose durable outcome requires a commit must explicitly declare or
check that requirement. This preserves the North Star boundary that workflow
content owns procedure and acceptance while core owns mechanics and pointers.

11.7 Teardown under mismatch

• Uncommitted/untracked assigned-surface changes use the existing durable patch
capture and fail-closed retention behavior.
• A clean worktree on the wrong named branch records that branch and final SHA
before linked-worktree removal; both named branches remain durable.
• A detached HEAD containing commits not proven reachable from a named ref
retains the worktree rather than risking loss.
• The expected sergeant/<work-id> branch is always retained even if it did
not advance.

────────

12. Durable branch lifecycle and diagnostics

12.1 Branches remain durable by default

Every terminal Work retains sergeant/<work-id> in every selected repository.
Sergeant never deletes it automatically because:

• the Work completed;
• a PR exists;
• a mounted branch advanced;
• commits appear merged;
• or the branch appears redundant.

Deletion is a separate explicit maintenance action with its own authorization
and dry-run evidence.

12.2 Cheap sgt doctor summary

sgt doctor gains one bounded estate Git/surface summary derived from the
journal, manifest, and retained-artifact filesystem metadata without walking
Git ancestry per branch:

```text
git surfaces
  active works:              3
  active linked worktrees:   7
  retained worktrees:        1
  retained patches:          2
  retained artifact size:    184 MiB
  journaled Work branches:   247
  terminal dirty Works:      4
```

The check names a separate inspection/cleanup remedy when nonzero residue
needs attention. It does not classify merged/squashed/redundant branches and
does not delete anything.

12.3 Expensive reconciliation remains separate

A deliberate sweep may later compare:

```text
journaled Work bindings/branch records
actual sergeant/* refs
Git worktree registry
surface paths
base/default/integration ancestry
```

That operation owns unknown-ref classification, stale-registration repair,
and explicit branch deletion. This proposal supplies the identities and
invariants it needs but does not make doctor perform the walk.

────────

13. Manifest and domain-model changes

13.1 Proposed manifest shape

```toml
[estate]
name = "payments"
data_dir = ".sergeant/data"
surfaces_dir = ".sergeant/data/surfaces"

[[repo]]
name = "payments-api"
origin = "git@github.com:company/payments-api.git"
instructions = "suppress"

[[repo]]
name = "auth"
origin = "git@github.com:company/auth.git"
instructions = "suppress"

[[repo]]
name = "payments-knowledge"
origin = "git@github.com:company/payments-knowledge.git"
instructions = "suppress"

[group.payments]
repos = ["payments-api", "auth", "payments-knowledge"]
brief = "Payment authorization, settlement, and governing team knowledge."

[[profile]]
name = "sonnet"
backend = "claude"
default_model = "sonnet"
```

Run defaults/templates are defined by the companion proposal.

13.2 Domain vocabulary

Workspace becomes Estate in the public/core domain model. Internal file
renames should follow where doing so removes semantic ambiguity; compatibility
aliases are unnecessary at 0.1.0.

Representative types:

```text
Estate
RepositoryMount
GroupSpec
WorkScopeRequest
ResolvedWorkScope
GitPreflight
GitPreflightOverride
RepositoryBinding
WorkSurface
IntegrityDisposition
IntegrityFinding
EstateDriftObservation
```

13.3 Work submission API

The submission request gains explicit scope and override fields and loses
workspace authority:

```json
{
  "command_id": "...",
  "intent": "...",
  "scope": {
    "repos": ["payments-api"],
    "group": "payments",
    "all": false
  },
  "workflow": "software-change",
  "backend": "claude",
  "profile": "sonnet",
  "override_git_preflight": false,
  "origin": {
    "client": "claude",
    "cwd": "/estate/root"
  }
}
```

origin.cwd is recorded evidence only. The daemon’s estate binding is
canonical.

13.4 Binding/event evidence

The journal must make these facts reconstructable without re-inspecting current
Git state:

```text
estate root used
scope request and resolved repositories
preflight observations
any authorized override
surface plan
canonical common-directory identities
base branches and SHAs
assigned branches and paths
integrity observations
terminal teardown and integrity disposition
estate drift observations
```

Existing surface.materializing, surface.materialized, and
surface.torn_down may be widened where additive history compatibility is
useful. At 0.1.0, a clearer event/type replacement is acceptable if it keeps
journal replay explicit and tested.

────────

14. AGENTS.md, skills, and operator doctrine

This proposal changes product behavior and must change the embedded distro in
the same release.

14.1 Session-start invariant

AGENTS.md should state, near the front door:

```text
A Captain session begins at the estate root.
Read ./sergeant.toml (or the sgt inspection surfaces that render it) before
acting. Never infer or search upward for another estate.
```

14.2 Estate/Git model

Always-on doctrine should explain:

```text
estate root                 Captain environment and AgentOS files
repos/<name>                estate-owned clean base checkout
surface/<work-id>/<name>    worker-owned linked worktree
sergeant/<work-id>          durable Work output branch
```

14.3 Captain’s pre-Work responsibility

Before meaningful repository Work, Captain:

• confirms it is at the exact estate root;
• reads sgt doctor, declared repositories, groups, and relevant templates;
• reconciles active/terminal dirty Work already touching the intended repos;
• selects the intended repository scope;
• ensures the mounted checkouts are on the intended committed base;
• dispatches rather than coding concurrently in the mounts.

Core repeats and enforces every mechanical Git check.

14.4 Worker responsibility

Workers are told the exact selected paths, bases, and assigned branches.
They do not:

• edit repos/ mounts;
• create replacement branches;
• navigate into another Work surface;
• expand repository scope themselves;
• invoke estate-scoped sgt commands from their surface.

Violation is reported dirty rather than silently treated as ordinary output.

14.5 Routing correction

The current in-session route that permits meaningful one-repository coding by
Captain should be removed or narrowed to dialogue, inspection, and genuinely
non-repository actions. The owner ruling is explicit: Captain captains.
Repository implementation belongs to dispatched Work, even when only one
repository is involved.

14.6 Skill updates

At minimum:

• estate-navigation stops upward inference and teaches exact-root checks;
• sergeant-help carries the new loud root/preflight remedies;
• to-spec and grilling remain Captain-session skills;
• workflow templates that instruct a stage actor to dispatch nested Work from
its linked worktree are revised to use admitted workflow composition or are
flagged as engine gaps rather than routing around the exact-root contract.

14.7 Glossary

Add definitions for Estate, Repository Mount, Work Scope, Repository Binding,
Work Surface, Integrity Disposition, and Estate Drift.

────────

15. Error behavior

Every refusal occurs at the earliest point that still has enough evidence to
name the remedy.

|Failure                                |Required response                                                                                          |
|---------------------------------------|-----------------------------------------------------------------------------------------------------------|
|No `./sergeant.toml`                   |Name current path, explain no ancestor search, suggest `cd <estate-root>` or `sgt init`.                   |
|Invalid manifest                       |Exact parser/schema diagnostic; do not search elsewhere.                                                   |
|Daemon descriptor bound to another root|Name both roots; do not connect or spawn a second daemon over the same data dir.                           |
|Missing multi-repo scope               |List declared repos/groups and show `--repo`, `--group`, `--all`.                                          |
|Unknown group/repository               |Name available values.                                                                                     |
|Mount path missing/wrong/aliased       |Name expected derived path and actual Git top level/common dir.                                            |
|Dirty/detached base                    |Show branch/HEAD/status and the normal remedy; mention the bounded `--override-git-preflight` escape hatch.|
|Unresolvable HEAD                      |Refuse; override is unavailable because no exact base can be pinned.                                       |
|Work ref/path collision                |Name ref/path and owning Work evidence if known; never delete or reuse automatically.                      |
|Surface materialization partial failure|Preserve/journal rollback report and refuse Work start.                                                    |
|Terminal branch mismatch               |Complete ordinary workflow retirement, mark integrity dirty, report expected and actual refs/SHAs.         |

Errors are stable structured API codes plus human remedies.

────────

16. Implementation plan

This is one architectural proposal but should land in reviewable slices.

Slice 0 — Contract pins and current-state reproductions

• Add failing tests for exact-root behavior, zero-config fallback removal,
multi-repo scope refusal, branch mismatch at teardown, and common-dir lock
aliasing.
• Preserve live evidence for #172/#173/#180 before changing the dogfood estate.
• Record current sgt doctor surface/branch totals as the baseline.

Slice 1 — Exact-root estate and daemon binding

• Introduce Estate resolution from cwd/sergeant.toml only.
• Gate every estate-scoped CLI command before data-dir/descriptor behavior.
• Add estate_root to daemon configuration and runtime descriptor.
• Remove engine workspace discovery from request cwd.
• Bind harness passthrough environment to exact root.
• Update root error messages and tests.

Slice 2 — Estate-owned mounts and manifest schema

• Remove arbitrary [[repo]].path.
• Derive and validate repos/<name>.
• Update sgt init, repo add/remove/list, doctor, fixtures, and dogfood estate.
• Reject symlink/linked-worktree/external-path mounts.
• Rename public Workspace vocabulary to Estate where appropriate.

Slice 3 — Explicit Work scope

• Add structured scope request to CLI/API.
• Move group expansion into daemon/core.
• Add --all.
• Permit implicit scope only for one-repository estates.
• Remove --workspace.
• Journal requested and resolved scope.

Slice 4 — Git preflight and bounded override

• Implement repository identity, common-dir, branch, HEAD, status, ref/path
collision checks.
• Add --override-git-preflight for dirty/detached only.
• Journal excluded dirty state and detached-base evidence.
• Ensure no preflight path fetches, pulls, switches, resets, or infers remotes.

Slice 5 — Common-dir locking and binding enrichment

• Replace checkout-path lock identity with canonical Git common directory.
• Add interprocess locking around registry/ref mutations.
• Widen RepositoryBinding and backend request context.
• Preserve existing materialization rollback and crash-window guarantees.

Slice 6 — Integrity reconciliation and retirement

• Record actual symbolic branch/HEAD/status against expected binding.
• Add integrity disposition/findings and estate-drift observations.
• Make branch mismatch visible without inserting workflow state changes.
• Protect detached/unreferenced output from teardown loss.
• Update work show/list, output pointer, watch snapshots, and TUI rendering.

Slice 7 — Diagnostics and doctrine

• Add cheap doctor surface/branch/disk summary.
• Rewrite AGENTS.md, estate-navigation, sergeant-help, glossary, help,
README examples, and affected workflow templates.
• Remove upward-discovery/zero-config claims from docs and tests.

Slice 8 — Dogfood gauntlet

Run from a clean estate root and demonstrate:

• one-repo and multi-repo Work;
• group selection;
• mounted branch advancement while Work runs;
• dirty/detached override evidence;
• worker branch switch producing completed/dirty without a hidden pause;
• attributable outside-mount command evidence where the Claude adapter can
supply it;
• unattributed concurrent mount movement reported as estate drift only;
• crash/retry/rematerialization;
• durable branch and doctor totals;
• exact-root refusal from every repository and surface descendant.

────────

17. Acceptance criteria

The proposal is complete only when all of the following are true.

Estate identity

• Every estate-scoped command refuses outside an exact valid root before
descriptor lookup or spawn.
• No code path searches ancestors for an estate during normal command or API
handling.
• No code path falls back to a Git repository as a workspace.
• A runtime descriptor cannot be used from a different estate root.

Repository topology

• Every declared repository resolves only to repos/<name>.
• The estate root is never implicitly a Work repository.
• A linked worktree, symlinked external checkout, or another estate’s clone is
refused as a mount.

Work scope

• A multi-repo estate cannot submit without explicit scope.
• --all is explicit and journaled.
• Group resolution is identical across CLI, TUI, and direct API clients.
• Existing Work meaning survives later manifest edits.

Git base and surface

• Normal admission requires clean attached HEAD and pins an exact SHA.
• Sergeant performs no automatic network or branch-changing Git command.
• Dirty/detached override records exactly what was waived and what was
excluded.
• Every selected repository receives exactly one Work branch/worktree.
• Every stage and retry uses the same Work surface/branches.
• Registry operations serialize by Git common directory across processes.

Integrity and retirement

• A Worktree ending on another branch cannot be reported clean or represented
as though the expected branch contains its output.
• Assigned uncommitted content is preserved under existing fail-closed rules.
• Detached/unreferenced output is not destroyed.
• Work/Stage state progression is unchanged by implicit integrity checks.
• Unattributed mount changes are not falsely charged to a Work.
• Workflow-required commit behavior remains workflow-owned.

Operations and doctrine

• Work branches remain after every terminal outcome.
• sgt doctor reports bounded active/retained/count/size facts without Git
ancestry walking or deletion.
• AGENTS.md makes Captain, estate root, mounts, surfaces, and branch ownership
unambiguous.
• Embedded distro and binary behavior ship together.

────────

18. Test strategy

Unit tests

• exact-root manifest resolution;
• command classification into estate-scoped/unscoped;
• derived repository paths and invalid-name protection;
• scope resolution and single-repo default;
• preflight result taxonomy and override matrix;
• common-directory canonicalization;
• integrity finding classification;
• terminal display mapping (completed + dirty).

Integration tests

• real CLI command refusal from root descendants;
• daemon descriptor estate mismatch;
• real two-repository group materialization;
• mount branch advances after admission while surface remains pinned;
• dirty and detached mount with/without override;
• wrong branch in assigned worktree;
• uncommitted and untracked patch retention;
• detached output retention;
• two source paths sharing a Git common dir contend on one interprocess lock;
• crash windows before/after plan, worktree add, materialization event, and
teardown event;
• no remote access or branch switch during admission (scripted Git binary);
• doctor summary against seeded journal/surface state.

Adapter contract tests

• StartRequest contains exact binding paths/refs/SHAs.
• Claude prompt names the mutation surface and forbidden estate mounts.
• Adapter tool events carrying cwd/path can produce attributable integrity
evidence.
• An adapter without that evidence still runs; core reports only what it can
prove.

Doctrine skew tests

• AGENTS.md command/root claims match --help and root-gate behavior.
• No embedded workflow tells a stage actor to run estate-scoped commands from
a Work surface.
• Manifest examples parse under the current schema.

────────

19. Explicit non-goals

This proposal does not:

• introduce a bare Git object store or replace ordinary estate clones;
• share one checkout across estates;
• fetch, pull, rebase, merge, or select remote default branches;
• prevent all filesystem reads outside the Work surface;
• become a credential, secret, network, package-cache, or device-auth broker;
• require every adapter to implement the same sandbox;
• add stage-level branches or surfaces;
• insert implicit workflow checkpoints;
• define whether a workflow must commit, push, or open a PR;
• automatically integrate Work output into a mount;
• automatically delete retained Work branches;
• classify squash-merged/redundant branches inside sgt doctor;
• add run templates (companion proposal);
• preserve compatibility with pre-0.1.0 workspace/manifest behavior.

There is one developer and one live dogfood estate. The implementation replaces
the old model directly; no migration subsystem, compatibility flag, or
deprecation window is warranted.

────────

20. Owner decisions captured by the grilling interview

All rows are J4: explicit owner decisions made 2026-08-19.

|# |Decision                                                                                                                               |
|-:|---------------------------------------------------------------------------------------------------------------------------------------|
|1 |Every durable Work requires an estate; zero-config Git fallback is removed.                                                            |
|2 |An estate is only the exact current directory containing `sergeant.toml`; no upward search.                                            |
|3 |Every harness passthrough and estate-scoped command uses the same exact-root rule and fails loudly before daemon interaction.          |
|4 |Only homepage/help/version/init/doctor work outside an estate.                                                                         |
|5 |Multi-repository Work requires explicit scope; omission never means all repos.                                                         |
|6 |Group remains membership plus brief only.                                                                                              |
|7 |Core owns mechanical Git preflight; Captain owns semantic reconciliation and scope/base intent.                                        |
|8 |The default base is each selected mount’s clean attached current branch and exact `HEAD`; no fetch/pull/switch/remote inference.       |
|9 |Selected repositories are the Work’s hard mutation/output boundary.                                                                    |
|10|Sergeant does not restrict reads or own the user’s security/credential environment.                                                    |
|11|Out-of-surface attributable mutation completes normally but makes terminal integrity dirty.                                            |
|12|Dirtiness is assessed at Work retirement by default; no automatic stage pause.                                                         |
|13|The one override may waive dirty/detached caution only; never missing identity, scope, provenance, collisions, or construction failure.|
|14|Every repository is estate-owned at `repos/<name>`; separate estates use separate clones.                                              |
|15|`repos/<name>` remains an ordinary prepared base checkout; workers never work there.                                                   |
|16|Mounted checkouts may advance while Works run; bindings remain pinned to admission SHA.                                                |
|17|One Work owns one surface/branch set for the whole workflow; stages are fresh executions over it.                                      |
|18|Work branches remain durable; deletion is explicit and separate.                                                                       |
|19|Doctor provides a cheap worktree/branch/disk summary.                                                                                  |
|20|Escaped mutation is charged to a Work only with direct surface or adapter causality; otherwise report estate drift.                    |
|21|No migration/compatibility machinery is needed at `0.1.0`.                                                                             |

────────

21. Final product statement

After this proposal lands, the ordinary loop is:

```text
cd <estate-root>
sgt claude
Captain reads ./sergeant.toml and reconciles the estate
Captain shapes intent and selects repositories/group/workflow
sgt run admits exact committed bases or refuses with a remedy
Sergeant creates one Work-owned linked worktree per selected repository
fresh workflow-stage executions operate over that one surface
Sergeant retires the Work with honest clean/dirty Git evidence
Work branches remain durable and visible through the journal/TUI/doctor
```

The core promise becomes concrete:

> **A Work never inherits estate or Git authority accidentally. Sergeant binds
> one exact estate, resolves one explicit repository scope, pins exact commits,
> constructs one owned surface, and reports what that surface actually left
> behind.**
