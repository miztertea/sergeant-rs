---
name: estate-navigation
description: Resolve an estate's declared repositories, groups, and health before acting in it, bring its working set up to date, register new repos/groups interactively, and file tracked work for gaps `sgt doctor` can't remedy — sergeant-rs's equivalent of upstream's sgt-context/sgt-sync/sergeant-setup.
edition: 0.2.1
---

Provenance for this skill's citation record is kept in this project's
private development record, not shipped here.

Content ported from the upstream core function map: `sgt-context`
("emit a project's layered agent-instructions block... for session start")
and `sgt-sync` ("clone missing / pull existing repos") were each ruled
**SKILL** by owner pre-ruling — "pure config read, no worktree or Work state
touched" / "git-reading harness assistance, no Work state" — never a
dispatched Work item. Their upstream mechanism (a `~/.config/sergeant/*.yaml`
project registry, `yq`-parsed, with `defaults → group → repo` instruction
layering the harness composed itself) does not exist in sergeant-rs and is
not re-created here; this skill teaches the equivalent judgment against
sergeant-rs's actual estate model (`sergeant.toml`'s `[estate]`/`[[repo]]`/
`[group.<name>]`, per the MVP-1 estate-manifest ruling).

**Extended at ICM-R2** (sergeant-setup adjudication, dev-corpus record) to
absorb `sergeant-setup`'s two remaining live
behaviors — interactive repo/group registration and capability-gap tracking
— now that package retired. Both new sections below are live, Captain-
session judgment (PL-2): they decide whether Work should exist (a registered
repo/group, a filed td issue), which is why they belong here rather than in
an admitted background workflow.

## When to use

Before acting on an estate whose repositories, groups, or health you haven't
already confirmed this session — never infer which repo or estate you're in
from the current directory (`AGENTS.md`'s "Session start" section already
states this as always-on policy; load this skill when a task needs
more estate-navigation detail than that one line covers, e.g. "what's
registered", "sync my repos", "is repo X declared", "register a new repo/
group", "set up this estate", "file a ticket for a missing prerequisite").

## Resolving estate context (the `sgt-context` equivalent)

1. Confirm the exact estate root first: the one directory holding
   `./sergeant.toml` with an `[estate]` table. Check that directory itself —
   `ls ./sergeant.toml` — and never walk upward looking for one, because
   `sgt` doesn't either: an estate-scoped command run anywhere else (a
   parent, a `repos/<name>` mount, a Work surface) refuses before it
   contacts the daemon, naming `cd <estate-root>`, `sgt -C <estate-root>
   <command>`, or `sgt init`. `sgt -C <estate-root>` addresses an estate
   without moving the session's working directory; it names an exact root
   too, with no search of its own.
2. `sgt doctor` — install/estate health with named remedies for anything
   fixable (git, the `claude` CLI and its version gate, Docker, the data
   directory, the journal, the analytics projection, the daemon, whether the
   current directory is an exact estate root, each profile's effective
   permission mode, and — inside an estate — the manifest's own health, the
   declared workflow catalog, a cheap Git-surface summary of active and
   retained worktrees/patches and terminal-dirty Works, and disk pressure).
   It is one of the surfaces that work outside an estate at all
   (`sgt --help`, `--version`, `sgt init`, `sgt doctor`, plus the
   host-scoped bucket — `sgt status`, `sgt tui`, `sgt work show`/`list`/
   `transcript`, `sgt watch`, every `sgt daemon` verb — H1 §5), so it can
   also answer "am I anywhere near an estate?" before anything else is
   tried.
3. `sgt repo list` — every declared `[[repo]]`: name, local path, instruction
   policy, origin, and upstream.
4. `sgt group list` — every declared `[group.<name>]` and its members, if the
   task spans more than one repo.

There is no harness-side instruction-layering step to perform yourself:
per-repo instruction policy (`[[repo]] instructions = "local" | "suppress"`)
is declared in the manifest and **resolved and pinned by `sgt` itself at
Work bind time** (MVP-1 ruling R-MVP1-4), not composed by the harness the
way upstream's `sgt-context` did. `suppress` (the default) is
byte-identical to today's launch behavior; `local` parses but is refused at
submit until its adapter translation is measured (MVP-2) — if a submission
is refused for this reason, that refusal *is* the correct, honest behavior,
not a bug to work around.

## Bringing the working set up to date (the `sgt-sync` equivalent)

- **A repo not yet cloned, or not yet declared:**
  `sgt repo add <name> --origin <url>` — clones `--origin` into
  `repos/<name>` if the directory doesn't exist yet, or verifies it's
  already a git repository if it does, then declares `[[repo]]`. This is
  the entire "clone missing" half of `sgt-sync`, natively.
- **A repo whose upstream should resolve inside every surface:**
  `sgt repo add <name> --upstream <url>` — records `[[repo]] upstream` and
  configures the mount's `upstream` remote, so `gh`, `glab` and plain `git`
  all resolve it from a Work's worktree. The URL is opaque and forge-neutral:
  sergeant infers no host, forge or CLI from it. The manifest is the
  authority — if a mount's remote later stops matching what it declares,
  `sgt doctor`'s estate row names the drift and the exact `git remote
  add|set-url` command that fixes it, and never rewrites the mount itself.
- **An already-declared repo whose local clone is behind its remote:**
  **no `sgt` verb covers this today** (the "pull existing" half of
  `sgt-sync` has no engine-side equivalent — a genuine, named gap, not
  something to route around). Fall back to a manual per-repo pull and say so:

  ```sh
  git -C repos/<name> pull --ff-only
  ```

  Report a failed or diverged pull rather than silently skipping it — the
  same honesty upstream's own `sgt-sync` applied (it warned and skipped on
  a diverged branch instead of forcing).
- **Undeclaring a repo:** `sgt repo remove <name>` — refuses (naming the
  group) while any group still lists it as a member; never deletes
  `repos/<name>` from disk.

## Registering repos and groups interactively

When a user wants to set up or extend an estate rather than just navigate an
existing one (former `sergeant-setup` `30-project-interview`'s
transplantable fragment): ask, one at a time, waiting for each
answer:

1. For each repository to add: its name, its clone origin URL (or confirm
   it's already cloned at `repos/<name>`), and which group(s) it belongs to
   if the estate uses groups.
2. Run `sgt repo add <name> --origin <url>` per repository — this call is
   already idempotent (verifies rather than re-clones if the directory
   exists) and already scoped to `sergeant.toml`/`repos/` only, so no
   separate preview-and-confirm ceremony is needed before it: the command
   itself is the confirmable, individually-reversible unit (`sgt repo
   remove` undoes one entry), not a single monolithic file write.
3. For group membership, use `sgt group add` (or the manifest's
   `[group.<name>]` table directly) to record it.

**What this does not cover, on purpose.** The retired interview also asked
for per-repo role, a free-text `agent_instructions` block (default and per-
group), and a project-level GitHub identity. None of these have a
`sergeant.toml` field today (the MVP-1 estate-manifest ruling — the schema
has `[[repo]] instructions = "local" | "suppress"`, not free text, and no
Graphify-path field at the estate level). The nearest thing to the third is
`[[repo]] upstream`, and it is deliberately narrower than what was asked
for: one opaque URL per repository, no identity, credential, account or
forge attached — record a URL there and nothing else. Don't invent values
for fields that don't exist; if a user asks for one of these, say plainly
that sergeant-rs's estate model doesn't have that field yet rather than
fabricating a place to put the answer.

## Filing tracked work for a gap `sgt doctor` can't remedy

When `sgt doctor` reports a failing check it names no remedy for, or a
required/optional prerequisite is otherwise confirmed unsupported (former
`sergeant-setup` `05-file-capability-gaps`): draft a `td` issue
— title, description, acceptance criteria — and show it in full for explicit
`y`/`yes` approval before creating it. On decline, do not create it; report
the gap plainly (in the session or in the estate-health summary you're
already giving the user) instead of silently dropping it or silently filing
it without consent.

## Guardrails this inherits from `AGENTS.md`

`sgt init`/`sgt repo add`/`sgt group add` write only within the estate they
scaffold (`sergeant.toml`, `repos/`) — never to another harness's own
configuration, and never to `AGENTS.md` or `CLAUDE.md` in any repo. A
missing tool or capability surfaces as `sgt doctor`'s named remedy, never a
silent skip or an invented workaround — the "no pull verb yet" gap above is
exactly that kind of honestly-named gap, not license to script something new
against the daemon's state. `td`, Graphify, and Treehouse are never
auto-initialized without an explicit per-tool confirmation prompt — if
consent is declined, leave the state unchanged and report the skip.

## Bounded judgment

*(Added ICM-R2, closing a pre-existing gap this skill predated —
owner ruling ICM-R0 decision 4; `.sergeant/common/contexts/icm-policy.md`
— flagged by the pilot's independent reviewer as a gap
the sergeant-setup fold made worse by adding new judgment content without
it.)*

### This skill may decide
- Whether a fact is already resolvable from `sgt doctor`/`sgt repo list`/
  `sgt group list` output rather than a genuine registration decision.
- How to phrase a drafted `td` issue (title, description, acceptance
  criteria) for the user's approval.

### This skill must ask the user
- Every repository/group registration field one at a time, waiting for
  each answer (name, origin, group membership).
- Explicit `y`/`yes` approval before filing a `td` issue for a capability
  gap — never filed on decline.
- Explicit per-tool confirmation before `td`, Graphify, or Treehouse is
  auto-initialized.

### This skill must not do
- Invent a value for a `sergeant.toml` field that doesn't exist (per-repo
  role, free-text `agent_instructions`, a project-level GitHub identity) —
  say plainly the estate model doesn't have that field yet.
- Write outside `sergeant.toml`/`repos/`, or to `AGENTS.md`/`CLAUDE.md` in
  any repo.
- Silently drop a capability gap or silently file a `td` issue without
  consent.

### Durable handoff
A registered repo/group (`sergeant.toml`) or a filed `td` issue, only on
explicit consent. No other promotable artifact.
