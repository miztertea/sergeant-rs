---
name: sergeant-help
description: Answer a Sergeant usage/setup/troubleshooting question from repository-owned documentation with a fixed precedence order, read-only, never inventing behavior.
edition: 0.2.1
---

# Sergeant Help

Ported from `.sergeant/workflows/sergeant-help` (N1 reference-corpus candidate
W4), which retires: the execution-surface test's OPERATOR-SKILL verdict
(`docs/icm/retriage-2026-08-11.md`) is that a doc lookup needs no worktree
and no durable Work state — it belongs at this skills/`AGENTS.md` layer the
harness loads directly, never as a dispatched `sgt run`. Content re-verified
against the real, built CLI (`sgt --help`, 2026-08-12) rather than assumed.
The root-gate and Git-preflight rows below were added 2026-08-20 against the
refusal text the binary actually prints (`EstateRootError`,
`src/domain/estate.rs`; the preflight remedies in `src/runtime/preflight.rs`),
not against a paraphrase of it.

## When to use

The user asks what Sergeant is, how to install/configure/use it, where
workflows or skills come from, how a specific `sgt` command or flag works, or
how to diagnose a Sergeant error — read-only, doc/help-shaped questions.

Do not use this in place of actually doing the thing: once the user has
asked for the estate to be set up, a repo registered, or work submitted, load
`estate-navigation` or route through `sgt run` (see `AGENTS.md`'s standard
workflow loop) instead of continuing to answer questions about it.

## Documentation map

| Question | Primary document |
|---|---|
| What Sergeant is, the product/ownership model | `README.md`, `NORTH-STAR.md` |
| Install and first estate | `README.md` ("Get it") |
| How a harness should route, the standard workflow loop, guardrails | `AGENTS.md` |
| Which directory an estate-scoped `sgt` command must run from, and why one refused with "no estate found in ..." or "this command must be run from the estate root" | `AGENTS.md` ("Session start"), `README.md` ("Using sgt day-to-day") |
| Why a submission was refused for a dirty or detached `repos/<name>` mount, and what `--override-git-preflight` does and does not waive | `AGENTS.md` (ESTATE, and the enforced list under "CAN — enforceable authority"), `sgt run --help` |
| Waiting for a Work's next attention/result without polling, avoiding a `sgt work show` loop, subscribing to Work transitions | `sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/WATCH.md` (the `sgt watch` command), `AGENTS.md` step 6 |
| Workflow authoring rules, the `.sergeant/` filesystem convention | `docs/icm/convention.md` |
| What workflows exist | `.sergeant/index.md` (the catalog) |
| A specific workflow's stages/inputs/outputs | `.sergeant/workflows/<name>/index.md` and `CONTEXT.md` |
| Per-host environment facts (uid, Docker, toolchain, proxy posture) | `docs/environments/<host>.md` |
| Building/testing/gating sergeant-rs's own code | `docs/DEVELOPMENT.md` |
| Estate manifest (`sergeant.toml`) shape and fields | `sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/MVP-1.md`, `sergeant-rs-workspace/knowledge/evidence/gauntlet/notes/estate-manifest-design-2026-08-11.md` |
| Why a design decision was made, deviations from the proposals | `GAUNTLET.md` (deviation register, backlog) |
| A binding lesson about how this project has been burned before | `LESSONS.md` |
| What a milestone actually promised/delivered | `sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/<milestone>.md` |

There is no `docs/schema.md`/`docs/troubleshooting.md`/`docs/getting-started.md`
in sergeant-rs — those upstream primary documents don't exist here; the rows
above are sergeant-rs's real equivalents, not a copy of upstream's map.

## Query procedure

1. Classify the question against the documentation map above.
2. Read the primary document before searching broadly.
3. For terms not resolved there, search repository documentation:

   ```sh
   rg -n -i --glob '*.md' -- '<term>' README.md AGENTS.md docs GAUNTLET.md LESSONS.md .sergeant
   ```

4. For flag or argument questions, run `sgt <command> --help` (every `sgt`
   command supports it — verified 2026-08-13 against the built binary) rather
   than assuming syntax. Top-level: `sgt --help` lists `daemon`, `status`,
   `run`, `work`, `respond`, `retry`, `extend`, `cancel`, `watch`,
   `analytics`, `tui`, `doctor`, `init`, `repo`, `group`. Bare `sgt` (no
   subcommand) is a homepage, not a listed subcommand (ADR 0010); the
   embedded dashboard and its `web` verb are gone (ADR 0011). Every
   subcommand but `init` and `doctor` is estate-scoped: run it from the
   exact estate root, or name that root with the global `-C <estate-root>`
   instead of `cd`-ing there — reading `--help` itself never needs either.
5. Answer with the exact command, required preconditions, expected evidence,
   and links to repository-relative documentation paths.
6. If sources disagree, use this precedence:
   - `sgt <cmd> --help` output and observed command behavior for released
     syntax (`docs/DEVELOPMENT.md`, "the Claude adapter's behavior is
     *measured*, never assumed from docs" — LESSONS L1);
   - `AGENTS.md` for always-on execution/safety policy;
   - the trigger-loaded skill or workflow's own `index.md`/`CONTEXT.md` for
     its procedure;
   - `sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/MVP-1.md` for estate-manifest fields;
   - `README.md`/`docs/DEVELOPMENT.md` for walkthroughs and dev commands.
7. State when a behavior is undocumented, unmeasured, or contradictory. Do
   not invent a command, flag, state transition, or safety guarantee.

## Answer format

```text
Answer: <direct answer>
Command: <exact command, when applicable>
Requires: <preconditions>
Verify: <observable success evidence>
Docs: <repository-relative links>
```

Omit fields that don't apply. Keep destructive operations (`git push
--force`, `sgt daemon stop` mid-work, anything under "Guardrails" in
`AGENTS.md`) out of examples unless the documentation itself requires
confirmation for them and the user explicitly requested them.

## Failure behavior

| Condition | Required action |
|---|---|
| Primary document missing | Report its expected path and stop before guessing. |
| Command behavior differs from documentation | Report the mismatch, trust the measured `--help`/observed behavior, and name the stale doc as a fix candidate — don't silently paper over it. |
| A command the user ran refused with the root gate ("no estate found in ...", or "this command must be run from the estate root") | Say plainly what the refusal means — sergeant does not search parent directories for an estate and does not fall back to a plain Git checkout — and repeat the remedy the refusal itself names: `cd <estate-root>`, `sgt -C <estate-root> <command>`, or `sgt init` if this directory should become an estate. Never suggest running the command from a parent, a `repos/<name>` mount, or a Work surface; only `sgt --help`, `--version`, `sgt init`, and `sgt doctor` work outside an estate at all. |
| A submission was refused by the Git preflight (a dirty or detached `repos/<name>` mount) | Report the mount's own named remedy verbatim — commit or stash it (`git -C <mount> status`), or check it out onto the intended branch (`git -C <mount> switch <branch>`). Name `--override-git-preflight` only as what it is: a per-submission waiver of a dirty or detached mount and nothing else, basing the Work on the committed HEAD; it is unavailable when the mount has no commit to pin, and it never waives any other preflight finding. |
| Question actually requires estate/repo state | Load `estate-navigation` (`sgt repo list`, `sgt doctor`) rather than answering from memory. |
| Question actually requires submitting or mutating work | Hand off to the standard workflow loop (`AGENTS.md`) / `sgt run`; this skill stays strictly read-only. |

## Bounded judgment

### This skill may decide
- Which primary document answers a classified question.
- How to phrase the `Answer/Command/Requires/Verify/Docs` response.
- How to respond to a missing primary document or an observed doc/behavior
  mismatch: report and stop, or report and flag, rather than guess.

### This skill must ask the user
- Nothing — this skill is read-only and does not itself reach J0; every
  unresolved condition it can encounter (missing doc, stale doc, a
  question needing estate state or mutation) routes to a stated report or
  a named hand-off, not a live question of its own.

### This skill must not do
- Answer from memory when a primary document is missing.
- Assume `sgt` command/flag syntax instead of running `--help`.
- Invent a command, flag, state transition, or safety guarantee.
- Include a destructive-operation example unless the documentation
  requires confirmation for it and the user explicitly requested it.
- Offer a way around a root-gate or Git-preflight refusal instead of its
  named remedy — no running an estate-scoped command from a parent
  directory or a Work surface, no hand-editing a `repos/<name>` mount to
  make preflight pass, and no `--override-git-preflight` for anything but
  the dirty-or-detached mount it actually waives.
- Continue answering once the question actually requires estate/repo state
  or work submission — `AGENTS.md`'s routing table settles ownership of
  that hand-off (J3: a settled authoritative record, not a J5 prohibition
  — corrected 2026-08-16, ICM-R2 pilot review) — hand off instead.

### Durable handoff
None. This skill produces no promotable artifact; the only "handoff" is
routing to a different skill (`estate-navigation`) or `sgt run` when the
user's need turns out not to be read-only, at which point this skill's own
procedure ends.
