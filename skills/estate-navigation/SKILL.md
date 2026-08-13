---
name: estate-navigation
description: Resolve an estate's declared repositories, groups, and health before acting in it, and bring its working set up to date — sergeant-rs's equivalent of upstream's sgt-context/sgt-sync.
---

Content ported from `reference/sergeant-upstream`'s core function map
(`docs/gauntlet/notes/upstream-core-function-map-2026-08-11.md`): `sgt-context`
("emit a project's layered agent-instructions block... for session start")
and `sgt-sync` ("clone missing / pull existing repos") were each ruled
**SKILL** by owner pre-ruling — "pure config read, no worktree or Work state
touched" / "git-reading harness assistance, no Work state" — never a
dispatched Work item. Their upstream mechanism (a `~/.config/sergeant/*.yaml`
project registry, `yq`-parsed, with `defaults → group → repo` instruction
layering the harness composed itself) does not exist in sergeant-rs and is
not re-created here; this skill teaches the equivalent judgment against
sergeant-rs's actual estate model (`sergeant.toml`'s `[estate]`/`[[repo]]`/
`[group.<name>]`, per `docs/gauntlet/contracts/MVP-1.md`).

## When to use

Before acting on an estate whose repositories, groups, or health you haven't
already confirmed this session — never infer which repo or estate you're in
from the current directory (`AGENTS.md`'s standard workflow loop step 1
already states this as always-on policy; load this skill when a task needs
more estate-navigation detail than that one line covers, e.g. "what's
registered", "sync my repos", "is repo X declared").

## Resolving estate context (the `sgt-context` equivalent)

1. `sgt doctor` — install/estate health with named remedies for anything
   fixable (git, the `claude` CLI and its version gate, Docker, the data
   directory, the journal, the analytics projection, the daemon, each
   profile's effective permission mode, and — inside an estate — the
   manifest's own health and disk pressure).
2. `sgt repo list` — every declared `[[repo]]`: name, origin, local path.
3. `sgt group list` — every declared `[group.<name>]` and its members, if the
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

## Guardrails this inherits from `AGENTS.md`

`sgt init`/`sgt repo add`/`sgt group add` write only within the estate they
scaffold (`sergeant.toml`, `repos/`) — never to another harness's own
configuration, and never to `AGENTS.md` or `CLAUDE.md` in any repo. A
missing tool or capability surfaces as `sgt doctor`'s named remedy, never a
silent skip or an invented workaround — the "no pull verb yet" gap above is
exactly that kind of honestly-named gap, not license to script something new
against the daemon's state.
