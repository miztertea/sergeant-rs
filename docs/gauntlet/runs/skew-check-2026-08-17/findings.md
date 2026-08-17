# Doctrine-vs-binary skew check — 2026-08-17

Phase 0. Investigate-only: nothing in `src/` or doctrine was changed, no
branch other than `sergeant/01M0748HEH4SHSCFX0MCN05PY0` was touched, no PR
opened.

## Method

1. Built the CLI authority by running `sgt --help` and `--help` for every
   subcommand and nested subcommand it lists (`daemon`, `daemon stop`,
   `status`, `run`, `work`, `work list/show/transcript/retained/reap`,
   `respond`, `retry`, `extend`, `cancel`, `watch`, `analytics`, `tui`,
   `doctor`, `init`, `repo`, `repo add/remove/list`, `group`, `group
   add/remove/list`, `claude`, `codex`, `opencode`, `goose`). `sgt --help`
   lists no `dispatch` or `harness` verb, so there is nothing further to
   expand there.
2. Extracted every CLI-shaped claim (`sgt <verb>`, `--<flag>`, `SGT_*`,
   `sergeant.toml` fields) from `AGENTS.md`, `CLAUDE.md` (byte-identical to
   `AGENTS.md`), `README.md`, `docs/DEVELOPMENT.md`, `docs/adr/*.md`,
   `NORTH-STAR.md`, all of `skills/*/SKILL.md`, and every `CONTEXT.md`/
   `index.md` under `.sergeant/workflows/` (`output/README.md` stage stubs
   excluded — they're boilerplate ICM-convention text, not CLI claims).
   `docs/gauntlet/**` (dev log, run archives, promoted-provenance) was
   read for corroboration only, not scanned as doctrine — it's a record of
   past work, not currently binding instruction.
3. Diffed each claim against the authority from step 1, verifying every
   mismatch by actually running the command in a scratch estate
   (`/tmp/skew-init-test`) and quoting its real output.
4. Noted binary capabilities with no doctrine mention (reverse direction).

## Summary

| Classification | Count |
|---|---|
| FLAG_MISSING | 2 |
| VERB_MISSING | 1 |
| ARG_SHAPE | 1 |
| BEHAVIOR | 2 |
| STALE | 0 |
| **Total mismatches** | **6** |

Reverse-direction (binary has, doctrine doesn't mention): 4 items, listed
at the end — not defects.

---

## Mismatches

### 1. FLAG_MISSING — `--intent-file` mandated for the risk-classification stage but absent from `sgt run`

**File:** `.sergeant/workflows/dispatch/05-classify-risk/CONTEXT.md`, lines 21, 27, 36

**Claim (quoted):**
> L21: "**An objective whose text matches a fixed set of safety-sensitive or stateful keywords (auth, security, secrets, payments, databases, migrations, production, destructive, persistent state, state transitions) cannot proceed on the standard-isolated intent path and must instead be given an explicit --intent-file.**"
>
> L27: "**--intent-file is mandatory whenever the objective names auth/OAuth, security, secrets or credentials, payments, databases or migrations, stateful/production work, destructive work, persistent state, or state transitions; the intent file must contain the eight required sections, and malformed, missing, path-traversing, symlinked, or oversized input fails before any dispatch mutation, while every other objective uses the lighter standard-isolated path.**"
>
> L36: "**An objective matching the fixed safety-sensitive keyword set (...) cannot proceed on the standard-isolated path — it must be given an explicit `--intent-file`** (`BU-P6-048`, `BU-P8-069`). Not a delegated judgment call; the keyword match is fixed."

This stage's own `output/README.md` describes its durable outcome as "The
objective is routed to the standard-isolated path or forced onto an
explicit intent-file path" — i.e. the mandate is meant to be enforced by
whatever CLI call actually creates the Work (`sgt run`, since there is no
`sgt dispatch` verb — see finding 3).

**Command run:**
```
$ sgt run --intent-file /tmp/foo "test"
error: unexpected argument '--intent-file' found

  tip: to pass '--intent-file' as a value, use '-- --intent-file'

Usage: sgt run [OPTIONS] <INTENT>
```
`sgt run --help`'s full option list is `--data-dir`, `--workflow`,
`--backend`, `--json`, `--profile`, `--repo`, `--group`, `--workspace`,
`--turns`, `--ceiling-secs` — no intent-transport flag of any kind.
`grep -rn "intent_file\|intent-file" src/` returns nothing.

**Classification:** FLAG_MISSING — a stage contract states a hard,
non-negotiable requirement ("Not a delegated judgment call") that no flag
in the shipped binary can satisfy. (This is the confirmed KNOWN 1.)

---

### 2. FLAG_MISSING — `sgt-watch --sync-all` cited as the fleet-reconciliation trigger; `sgt watch` has no such flag

**File:** `.sergeant/workflows/dispatch/80-monitor/CONTEXT.md`, line 51

**Claim (quoted):**
> "(trigger: sgt-watch --sync-all runs, or dispatch runs it automatically before new work; outcome: fleet state converges toward truth using identity-verified evidence and a bounded grace period, never a bare liveness guess, and never silently sweeps a needs_input/blocked/orphaned record)"

Distinct from finding 1's severity: every other `sgt-watch`/`sgt-dispatch`/
`sgt-validate` mention in this stage's contract carries an explicit
`reference/sergeant-upstream/...` citation, i.e. it's cited as behavior
ported from the old bash tool being decomposed (BU-P# provenance), not
asserted as the current `sgt` binary's interface — see the workflow's own
`CONTEXT.md`, which already carries a 2026-08-16 (ICM-R3) correction
distinguishing shipped `sgt respond` from unbuilt delegation targets. This
one bullet, though, states the trigger condition operatively ("sgt-watch
--sync-all runs, **or** dispatch runs it automatically") without that
citation immediately attached, and reads as describing how reconciliation
is actually invoked today.

**Command run:**
```
$ sgt watch --sync-all
error: unexpected argument '--sync-all' found

  tip: to pass '--sync-all' as a value, use '-- --sync-all'

Usage: sgt watch [OPTIONS] [ID]
```
`sgt watch --help`'s full option list is `--data-dir`, `--follow`,
`--json` — no `--sync-all`. `grep -rn "sync_all\|sync-all" src/` only
matches unrelated `File::sync_all()` fsync calls.

**Classification:** FLAG_MISSING — lower confidence than finding 1 (this
bullet is more plausibly upstream-tool provenance bleeding into the
trigger clause than a live claim about `sgt watch`), flagged because nothing
in the shipped engine performs bulk fleet reconciliation from a watch
command at all — `sgt watch` is read-only (`docs/adr/0009`).

---

### 3. VERB_MISSING — no `sgt dispatch` subcommand exists; the `dispatch` workflow package assumes one

**Files:** pervasive across `.sergeant/workflows/dispatch/**/CONTEXT.md`
(e.g. `00-check-queue-and-plan/CONTEXT.md:35`, `15-check-admission/CONTEXT.md:23,57,68`,
`20-prepare-intent/CONTEXT.md:22`, `80-monitor/CONTEXT.md` throughout)

**Claim:** dozens of bullets phrase behavior as "sgt-dispatch does X" /
"sgt-dispatch must Y" (e.g. `80-monitor/CONTEXT.md:92`: "**sgt-dispatch
must resolve an OpenCode (`oc`) target session for routing coordinator
notifications...**"). Read charitably these are citations of the upstream
bash tool (`reference/sergeant-upstream/bin/sgt-dispatch`) being
decomposed into behavior units, and the workflow's own `CONTEXT.md`
already flags that the two things this package would need to delegate to
(`drain-fleet`, `respond-to-worker`) "neither... exists in this
repository — both are open, unbuilt engine gaps." That self-correction
means the doctrine authors know the mapping from `dispatch`-the-workflow
to a concrete `sgt` invocation is incomplete — but nowhere in the package
does it state the one fact that would resolve the ambiguity: **there is no
`sgt dispatch` verb at all**, hyphenated or otherwise. A reader
encountering `sgt-dispatch` repeatedly, without that explicit disclaimer
next to the verb itself, would reasonably try `sgt dispatch`.

**Command run:**
```
$ sgt dispatch --help
error: unrecognized subcommand 'dispatch'

  tip: a similar subcommand exists: 'watch'

Usage: sgt [OPTIONS] [COMMAND]
```
Full top-level verb list (`sgt --help`): `daemon`, `status`, `run`,
`work`, `respond`, `retry`, `extend`, `cancel`, `watch`, `analytics`,
`tui`, `doctor`, `init`, `repo`, `group`, `claude`, `codex`, `opencode`,
`goose`. No `dispatch`, no `harness`.

**Classification:** VERB_MISSING. The `dispatch` *workflow* is real and
correctly invoked as `sgt run --workflow dispatch` (mechanically — though
see finding 5: that 422s on a fresh estate). The confusion is that the
package's own prose, written from the upstream bash tool's perspective,
never states this mapping once, so `sgt-dispatch` reads as a present-tense
CLI verb throughout.

---

### 4. ARG_SHAPE — `sgt work retry` cited twice; the real command is top-level `sgt retry <id>`

**File:** `.sergeant/workflows/repo-to-icm/20-harvest/references/partition-checkpoint-protocol.md`, lines 14, 110

**Claim (quoted):**
> L14: "`output/` is Git-tracked on this run's Work branch and persists across a stage retry (`sgt work retry` re-enters a stage as a fresh execution — fresh actor turn, fresh context window — against the artifacts already on disk; ..."
>
> L110-111: "...someone (a human operator, or an orchestrating caller of this Work) needs to notice it and cause another attempt of this stage. `sgt work retry` is **not** that mechanism (fixes #53; ...): retry is only legal against a failed/blocked/waiting stage, and this stage is neither..."

**Command run:**
```
$ sgt work retry --help
error: unrecognized subcommand 'retry'

Usage: sgt work [OPTIONS] <COMMAND>
```
`sgt work --help`'s subcommands are `list`, `show`, `transcript`,
`retained`, `reap` — no `retry`. The real command is the top-level
`sgt retry <ID>` (confirmed via `sgt retry --help`: "Retry the current
stage of a failed, blocked or waiting work item").

**Classification:** ARG_SHAPE — the verb (`retry`) is real and the
described semantics (re-enters a stage as a fresh execution; only legal
against failed/blocked/waiting) match `sgt retry`'s actual behavior
exactly; only the argument path is wrong (nested under `work` instead of
top-level).

---

### 5. BEHAVIOR — `sgt init` does not create `.sergeant/`, so data-dir resolution silently falls through to `$XDG_DATA_HOME`/`~/.local/share/sergeant`, contradicting NORTH-STAR's "in-estate" default

**Files:** `NORTH-STAR.md:140` (claim) vs. `src/` behavior (observed)

**Claim (quoted):** `NORTH-STAR.md:140`: "**Wave 1 — the estate**: `repos/`
manifest, data-dir default flipped in-estate, per-repo instruction
contract, E5 discoverability, daemon lifecycle + admission verbs (drain =
one journaled event pair), live-turn stall detection." Corroborated by
`NORTH-STAR.md:86-88`: "machine-local truth is in-estate and gitignored"
and `README.md:90`'s precedence chain, which lists `this estate's own
.sergeant/data` ahead of the XDG fallbacks and calls it "the path `sgt
init`'s `.gitignore` entry covers."

**Command run (fresh directory, nothing else present):**
```
$ cd /tmp/skew-init-test && sgt init
initialized estate at /tmp/skew-init-test
  created sergeant.toml
  created repos/
  updated .gitignore

sergeant doctor — /home/miztertea/.local/share/sergeant
  [ok  ] data_dir     /home/miztertea/.local/share/sergeant is writable
  ...
$ ls -la /tmp/skew-init-test
-rw-r--r-- .gitignore
-rw-r--r-- .sergeant.toml.lock
drwxr-xr-x repos
-rw-r--r-- sergeant.toml
$ ls -la /tmp/skew-init-test/.sergeant
ls: cannot access '.sergeant': No such file or directory
```
`sgt init`'s own `.gitignore` output is `.sergeant/data`, `repos/`,
`sergeant.toml`, `.sergeant.toml.lock`, `sergeant.toml.validate-*` — it
writes a rule for a directory (`.sergeant/data`) it never creates, so
`resolve_data_dir`'s estate-discovery rung (walk up looking for
`.sergeant/data`) has nothing to find and falls through to
`$XDG_DATA_HOME`/`~/.local/share/sergeant`, exactly as README.md's own
precedence chain predicts for the "wrinkle" case — except README.md scopes
that wrinkle to "the very first `sgt init`... since the estate doesn't
exist yet at the instant that check runs," implying every command *after*
that first one resolves in-estate. In fact every subsequent command still
resolves outside the estate, because nothing ever creates `.sergeant/`.

**Classification:** BEHAVIOR — NORTH-STAR states the in-estate default as
a ruled, shipped Wave 1 item; the shipped `sgt init` does not establish
the directory that default depends on, so real estates land data outside
the clone by default. (This is the confirmed KNOWN 2.)

---

### 6. BEHAVIOR — a freshly initialized estate has no `.sergeant/workflows/`, so any explicitly named workflow 422s

**Files:** `README.md:210-213` (workflow-drop-in claim) vs. `sgt init --help` (scaffolding claim) vs. observed behavior

**Claim:** `sgt init --help`'s own summary: "Scaffold an estate at the
current directory (MVP-3): `[estate]` in `sergeant.toml`, `repos/`,
`.gitignore` entries for `.sergeant/data` and `repos/`." `README.md`
documents 23 published workflows living under `.sergeant/workflows/` in
*this* repo and says routing to a named one is just `--workflow <name>`;
nothing in `sgt init`'s scaffold list mentions workflows, but nothing
elsewhere warns that a fresh estate ships with zero of them either — the
workflow catalog reads as generally available once you have an estate.

**Command run (same scratch estate, repo declared so submission gets past
the repo check):**
```
$ sgt repo add testrepo --origin <local-path>
added repo testrepo at /tmp/skew-init-test/repos/testrepo
$ sgt run --repo testrepo --workflow software-change "test intent"
submitted 01M074FWXNK7P4PKNWT5W5KAYJ (completed) [30-close]
$ sgt run --repo testrepo --workflow dispatch "test intent 3"
sgt: 422: workflow "dispatch" not found (looked in /tmp/skew-init-test/.sergeant/workflows/dispatch)
```
`software-change` (and presumably the other built-in default) resolve
without a `.sergeant/workflows/` directory on disk at all — they're
compiled in. Every other named workflow, including the 22 others this
same repository's own `.sergeant/index.md` catalogs, 422s on any estate
`sgt init` scaffolded, because `sgt init` never populates
`.sergeant/workflows/` and nothing else does either.

**Classification:** BEHAVIOR — `sgt init`'s help text is accurate about
what it scaffolds (it never claims to seed workflows), but the doctrine
corpus's workflow catalog (README.md, `.sergeant/index.md`, every
skill/AGENTS.md routing table entry that names a specific workflow) reads
as unconditionally available once an estate exists, when in fact only the
compiled-in default workflow works out of the box. (This is the confirmed
KNOWN 3.)

---

## Reverse direction — binary capabilities not mentioned anywhere in doctrine

Not defects; useful gaps to fold into doctrine later.

1. **`sgt work retained`** and **`sgt work reap`** (#109's inspect/dispose
   verbs for teardown-retained dirty state) — zero hits for "retained" or
   "reap" anywhere in the scanned doctrine set.
2. **`sgt group add --brief <text>`** ("One orientation line (AI-facing)")
   — zero hits for `--brief`; `README.md`'s `group add` example
   (`sgt group add <name> <repo>...`) omits it entirely.
3. **`sgt init --name <name>`** (override the estate name) — zero hits;
   every doctrine example uses bare `sgt init`.
4. **`sgt daemon`** run with no subcommand (foreground daemon until
   SIGINT/SIGTERM) — doctrine only ever mentions `sgt daemon stop`; the
   foreground form is undocumented outside `sgt daemon --help` itself.
