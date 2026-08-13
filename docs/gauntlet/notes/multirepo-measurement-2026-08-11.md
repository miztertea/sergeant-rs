# R-MVP1-5(a) — what `--repo` multi-repo bind DOES today

Governing: `docs/gauntlet/contracts/MVP-1.md` R-MVP1-5(a) ("measurement first —
the contract's first deliverable, before any group reasoning applies"),
R-MVP1-1 (`surfaces_root` split, not yet built), R-MVP1-4 (instruction-projection
contract, not yet built). LESSONS L1 ("measure the Claude CLI, never trust its
docs or its exit codes" — same discipline turned on our own engine, per the
contract's own framing) and L15 (a transmitted claim carries its evidence or is
a labeled hypothesis). Measured 2026-08-12 on Cerberus, against the working
tree at commit `31db58d` (R-MVP1-8 landed, R-MVP1-1/3/4/12 not yet built — this
note measures the **pre-R-MVP1-3 vocabulary**, `[workspace]`/`[[repository]]`,
because that is what is live).

**Verdict: the bind is not broken.** Both worktrees materialize, both land on
`sergeant/<work-id>`, `work show` names both, teardown retains both branches
and cleanly removes both worktrees whether the work runs to completion or is
canceled mid-flight. R-MVP1-5(b)'s falsifier — "an actor that cannot reach a
bound repo, a teardown that loses one, a `work show` that hides one" — does
not fire. Nothing here blocks MVP-3's CLI-side `--group` expansion plan.

## What was measured, and how

A real `sgt` daemon (`target/debug/sgt`, debug build at the commit above),
started two ways: once via the CLI's auto-spawn (default fake backend, no
script — every step completes immediately) and once with
`SGT_FAKE_SCRIPT="needs_input:parked for filesystem inspection"` so a run
could be caught mid-flight, both worktrees on disk at once, before teardown.
No token spend: the fake backend was used throughout, not a real Claude turn
— out of this note's narrowed scope (the task that produced it asked for "real
daemon, two scratch repos, one work, `--repo` twice", not a real-Claude turn,
which in any case needs R-MVP1-7's envelope, not yet built). This is recorded
as an open item below, not glossed over.

Scratch layout, disposable, outside the repo (`/tmp/.../scratchpad/mvp1-mr/`,
not committed):

```
estate/            git repo, holds sergeant.toml
  sergeant.toml
repo-a/             separate git repo, one commit
repo-b/              separate git repo, one commit
data-dir/           daemon 1 (completes the whole workflow)
data-dir2/          daemon 2 (parks on stage 1, for live filesystem inspection)
```

`estate/sergeant.toml` (today's vocabulary — `[workspace]` / `[[repository]]`,
`deny_unknown_fields`, per `src/domain/workspace.rs:172-181`):

```toml
[workspace]
name = "measurement-estate"

[[repository]]
name = "repo-a"
path = "../repo-a"

[[repository]]
name = "repo-b"
path = "../repo-b"
```

Commands (from `estate/`, the directory `Workspace::discover` resolves to via
`git rev-parse --show-toplevel` — R-MVP1-12's cross-boundary walk is not yet
built, so this only works because `estate/` is itself a git repo holding
`sergeant.toml`, not a directory nested under either member repo):

```sh
sgt --data-dir "$DATA" --json run --repo repo-a --repo repo-b "measure multi-repo bind"
sgt --data-dir "$DATA" --json work show "$WORK_ID"
curl -H "Authorization: Bearer $TOKEN" "$ENDPOINT/v1/events?work_id=$WORK_ID"   # raw journal
```

## Finding 1 — surface layout

One worktree per bound repository, named after the repository, directly under
the work's surface root — exactly `surface.rs`'s module doc
(`src/runtime/surface.rs:1-10`):

```
<data-dir>/surfaces/<work-id>/repo-a/     git worktree on sergeant/<work-id>
<data-dir>/surfaces/<work-id>/repo-b/     git worktree on sergeant/<work-id>
```

Confirmed live, mid-flight (daemon 2, parked run), by walking the actual
filesystem and asking git directly, not by reading `work show`'s claim about
itself:

```
$ find data-dir2/surfaces
data-dir2/surfaces/01KZSMXR3SMYK65H19DB232S7A
data-dir2/surfaces/01KZSMXR3SMYK65H19DB232S7A/repo-a
data-dir2/surfaces/01KZSMXR3SMYK65H19DB232S7A/repo-a/.git
data-dir2/surfaces/01KZSMXR3SMYK65H19DB232S7A/repo-a/README.md
data-dir2/surfaces/01KZSMXR3SMYK65H19DB232S7A/repo-b
data-dir2/surfaces/01KZSMXR3SMYK65H19DB232S7A/repo-b/.git
data-dir2/surfaces/01KZSMXR3SMYK65H19DB232S7A/repo-b/README.md

$ git -C data-dir2/surfaces/01KZSMXR3SMYK65H19DB232S7A/repo-a rev-parse --abbrev-ref HEAD
sergeant/01KZSMXR3SMYK65H19DB232S7A
$ git -C data-dir2/surfaces/01KZSMXR3SMYK65H19DB232S7A/repo-b rev-parse --abbrev-ref HEAD
sergeant/01KZSMXR3SMYK65H19DB232S7A

$ git -C repo-a worktree list
repo-a                                                     db01cd5 [main]
data-dir2/surfaces/01KZSMXR3SMYK65H19DB232S7A/repo-a       db01cd5 [sergeant/01KZSMXR3SMYK65H19DB232S7A]
```

Both worktrees exist simultaneously, both on the **same** `sergeant/<work-id>`
branch name in their own repository — the shared-branch-name, per-repository
worktree shape R-MVP1-5's falsifier is checking for. Neither is missing,
neither is on the wrong branch.

## Finding 2 — execution cwd

**Source-confirmed, then live-consistent.** `WorkSurface::execution_cwd()`
(`src/runtime/surface.rs:152-161`):

```rust
pub fn execution_cwd(&self) -> PathBuf {
    match self.bindings.as_slice() {
        [only] => only.worktree_path.clone(),
        _ => self.root.clone(),
    }
}
```

For two-or-more bound repositories the actor's `cwd` is the **surface root**,
not either repository's worktree — `<data-dir>/surfaces/<work-id>/`, the
parent directory holding `repo-a/` and `repo-b/` as siblings. This is the one
value the fake backend's `StartRequest.cwd` receives (`src/runtime/
engine.rs:2216`, the live launch path this run actually took, and
`engine.rs:2049`, the resume path); it is not journaled anywhere (no event
records a `StartRequest`, only the backend receives it), so this note cites
the source the daemon that produced the run above was built from, rather than
inventing an inspection point that does not exist. The live filesystem check
in Finding 1 is the corroborating fact available without one: both `repo-a/`
and `repo-b/` sit one level below `surface.root`, so a process whose cwd is
`surface.root` can reach either by a one-hop relative path (`repo-a/...`,
`repo-b/...`), which is what "the actor reached both" can honestly mean for a
harness given a `cwd` and a relative path convention, absent a live actor that
actually walks in and writes (the fake backend performs no filesystem I/O at
all — the open item below).

## Finding 3 — `workflow.bound` content

Full payload, one two-repo run, embedded `software-change` workflow (4
stages), fake backend:

```json
{
  "backend": "fake",
  "profile": null,
  "route_source": "global_default",
  "stage_bindings": [ /* one entry per stage: harness, index, kind, route_source, stage_id */ ],
  "workflow": { "content_hash": "...", "name": "software-change", "source": "embedded", "stages": [...], "version": "1" },
  "workspace": "measurement-estate"
}
```

**The repository set is not in this event.** `workflow.bound` carries
`"workspace": "measurement-estate"` — the workspace *name*, a string — and
nothing that says which repositories, at which paths, were bound. This is
exactly the gap R-MVP1-4 names and rules to close: *"`workflow.bound` widens
from `workspace: <name>` (`engine.rs:1044-1051`) to the resolved
`Vec<RepositorySpec>` + per-repo policy + resolved instruction file
identities."* Measured here as the "before" state that ruling's pin will
change. The per-repository detail that *does* exist today lives one event
earlier, in `surface.materialized` (Finding 1's payload) and on the `Work`
record itself (`work.repositories: ["repo-a", "repo-b"]`, the raw `--repo`
strings as submitted, journaled in `work.submitted` and echoed by `work
show` — see Finding 5) — neither of which is `workflow.bound`, and neither of
which pins instruction-file identity at bind time (R-MVP1-4's "resolved
instruction file identities" is wholly unbuilt; there is no field for it to
go in yet).

## Finding 4 — per-repo policy reachability: none exists

Two independent checks, both negative:

1. **The manifest schema.** `RepositorySpec` (`src/domain/workspace.rs:33-39`)
   is `{ name, path }` only. No `instructions` field, no per-repo policy of
   any kind — `[workspace]`/`[[repository]]` (today's vocabulary) has nothing
   to declare `local`/`suppress` with. R-MVP1-4 adds exactly this field; it
   does not exist yet.
2. **The launch grammar.** The Claude adapter hardcodes one `--setting-sources
   user` for every turn (`src/backend/claude.rs:874-881`), unconditionally —
   there is no branch on which repository is bound, and multi-repo execution
   runs one process at the shared surface root (Finding 2), so there is
   structurally only one `Command` per turn to carry a policy on even if the
   manifest declared one. This is the exact shape R-MVP1-4 rules on
   ("one process, one policy — so no composition happens, and that is the
   ruling, not an omission").

So today: a two-repo bind has one shared launch configuration, no field to
disagree in, and nothing to refuse at submit — R-MVP1-4's mixed-policy refusal
has no way to trigger yet because there is no policy to be mixed.

## Finding 5 — `work show` and teardown

`work show <id> --json` after the completed run named both repositories, in
the `surface.bindings` array (name, source path, worktree path, branch,
base/head SHA per repo) and in `work.repositories` (`["repo-a", "repo-b"]`).
Neither repo was hidden or dropped.

Teardown, checked two ways:

- **Run to completion** (daemon 1, unscripted fake — every stage
  `StageCompleted` on the very next OBSERVE): all 4 stages ran, the work
  completed, and teardown fired in the same request — `surface.torn_down`
  reports `"clean": true`, both bindings `"disposition": "removed"`. The
  per-work surface directory itself is gone (`remove_dir`, only once empty,
  per the module doc); both source repos retain `sergeant/<work-id>`
  (confirmed with `git branch -a` in each).
- **Canceled mid-flight** (daemon 2, parked on `needs_input` at stage 0):
  `sgt cancel <id>` also tore down cleanly — same `"clean": true`, both
  worktrees removed, both `sergeant/<work-id>` branches retained. Teardown's
  fail-closed retention path (dirty/missing/error dispositions) was not
  exercised — the fake backend never writes to the worktree, so there was
  never anything to make a worktree dirty. That path is already covered by
  `tests/m4_backends.rs`'s crash-injection matrix (§22.5) and is not this
  note's job to re-measure.

## Answering R-MVP1-5(a)'s four questions directly

| question | answer |
|---|---|
| `execution_cwd` the actor got | surface root (`<data-dir>/surfaces/<work-id>/`), source-confirmed + live-consistent — Finding 2 |
| both worktrees exist on `sergeant/<work-id>`, actor reached both | both exist, both on that branch name, confirmed live (Finding 1); "reached" is structural (one-hop relative path from a shared cwd), not demonstrated by a live actor — no real harness ran in this pass |
| what teardown retained | both `sergeant/<work-id>` branches, both source repos, on every terminal path measured (complete, cancel); worktrees removed cleanly both times |
| what `work show` said | both repositories named, full per-repo binding detail, matching the journal |

## Falsifier check (R-MVP1-5(b))

None of the three break-conditions fired: no repo was unreachable, no
teardown lost one, no `work show` hid one. **The group-expansion ruling's
precondition holds** — R-MVP1-5(b)'s CLI-side `[group.<name>].repos`
expansion over repeatable `--repo` has a working bind underneath it to expand
into. This note is the citation R-MVP1-5(b)'s pin asks for.

## Open items (honest bounds, not measured here)

1. **No real actor.** The fake backend performs no filesystem I/O, so "the
   actor reached both worktrees" is a structural claim (cwd + relative path),
   not a demonstrated one — the contract's own text asks for "one bounded
   real-Claude turn inside R-MVP1-7's envelope", and R-MVP1-7 (the turn
   envelope) is not built yet. Re-measure with a real Claude turn once
   R-MVP1-7 lands and bounds the spend.
2. **`workflow.bound` has no repository list** (Finding 3) — the state
   R-MVP1-4 is scoped to change, not this note's job to fix.
3. **R-MVP1-12 (discovery past inner `.git`) is not built.** This measurement
   worked only because the estate directory holding `sergeant.toml` was
   itself the git toplevel `Workspace::discover` found — the harder shape
   (estate above a member repo, discovered from inside the member) is
   R-MVP1-12's own fixture set, not reproduced here.
4. **Dirty/missing/error teardown dispositions** were not exercised (Finding
   5) — already covered elsewhere (`tests/m4_backends.rs` §22.5), cited, not
   re-measured.

## Evidence

Scratch tree, disposable, not committed:
`/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/mvp1-mr/`
— `run-submit.json` / `run-submit2.json` (full `--json run` responses),
`work-show.json`, `events.json` (full raw journal for the completed run).
Both scratch daemons were stopped (`SIGTERM`) and `pgrep -af
"sergeant-rs/target/debug/sgt [-]-data-dir"` was empty after — no leaked
process from this measurement.

## Addendum, MVP-1 fixer pass (2026-08-12) — I2

Open Item 1's stated blocker ("R-MVP1-7 ... is not built yet") is gone:
R-MVP1-7's turn envelope landed (commit `9b24742`, before this fixer pass),
and this pass itself added its exit door (R-MVP1-10's `extend_turn_envelope`)
and its production config surface (`SGT_TURN_CAP`). The re-measure Open Item
1 asks for is genuinely unblocked now.

**Not run in this pass.** "One bounded real-Claude turn inside R-MVP1-7's
envelope" spends real tokens (`SERGEANT_CLAUDE_TESTS=1`'s own opt-in
condition, CLAUDE.md), and a fixer pass closing review findings is not
authorization to spend them unilaterally — that call belongs to whoever
scopes the token budget for this milestone. This addendum exists so the
unblocking is on record and the remaining gap is a deliberate, named
decision (L15: a transmitted claim carries its evidence or is a labeled
hypothesis) rather than a silently stale "not built yet."

**Recommended next step**, when someone allocates the spend: repeat this
note's own scenario (two-repo `sgt run --repo a --repo b`) with a real
`claude` turn substituted for the fake backend's scripted completion, inside
`DEFAULT_TURN_CAP`'s envelope, recording whether the actor's own
`execution_cwd` actually reached both worktrees (Open Item 1's "structural,
not demonstrated" gap) — the same six columns this note's table already
tracks, this time with a live column to fill in rather than "structural".
