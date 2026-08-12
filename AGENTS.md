<!--
  This is sergeant-rs's small constitution (docs/icm/convention.md §3): a
  stable operating invariant document, not a procedural encyclopedia. It
  changes rarely by design — frequent churn here is a classification
  defect, not a documentation improvement (§3 rule 3). It does not restate
  `.sergeant/index.md`'s catalog or any workflow's own `index.md` (§3 rule
  2); it routes to them by name.

  Provenance: rewritten 2026-08-12 (Cerberus, MVP-5 CONTENT, Lane F1) to
  consume the 126 `agents-invariant`-classified units from N2 run 4
  (`docs/gauntlet/runs/n2-run4/.../40-classify/output/classifications.ndjson`,
  representation=agents-invariant). Every one of the 126 is dispositioned —
  landed here (cited by `BU-xxxx` id in an HTML comment beside the line it
  justifies), landed in a named skill/workflow, or recorded not-adopted with
  a reason — in `docs/icm/agents-invariant-dispositions.md`. This file is
  hand-authored; no workflow or skill auto-appends to it (BU-1311).
-->

# sergeant-rs

Sergeant is an AgentOS distro: a cloned directory of instructions, skills,
and conventions that turns a general-purpose coding harness into an
operator of your estate. It is carried by `sgt`, a durable
intent-execution engine that runs submitted intents to completion in
isolated git worktrees whether or not anyone is watching. The full
destination and the rulings behind it are `NORTH-STAR.md` — read it before
changing anything here.

Documentation is layered by ownership: this file owns always-on operating
policy for any harness acting in an estate; `.sergeant/workflows/*/index.md`
and `SKILL.md` files own trigger-specific procedure; `docs/DEVELOPMENT.md`
owns the rules for changing sergeant-rs's own code; `README.md` owns
install/quickstart. When two sources disagree about a behavior, the one
that owns that topic wins. <!-- BU-0106 -->

## Trigger → skill/workflow routing table

| Trigger | Load | Owns |
|---|---|---|
| A task names or implies work in a registered repo, and the outcome should be a durable, resumable Work item | `sgt run "<intent>"` (below) | Intent shaping, workflow selection, execution |
| The user asks how to install, configure, use, or diagnose Sergeant itself (read-only, doc/help-shaped) | `sergeant-help` (`.sergeant/workflows/sergeant-help/`) | Documentation lookup, command verification, prerequisites |
| The estate isn't set up yet, or `sgt doctor` reports a fixable install/config fault | `sgt init` / `sgt doctor` (not a skill — CLI verbs; see "When NOT to use `sgt`") | Estate scaffolding, install repair |
| Substantive procedural work has a matching published workflow | the workflow's own `index.md` under `.sergeant/workflows/<name>/`, discovered via `.sergeant/index.md` | That workflow's stages, inputs, and outputs |
| A `@@name` reference appears in an active stage's `CONTEXT.md` | `.sergeant/common/contexts/<name>.md` | Shared context text, resolved by this exact rule and no other |

`.sergeant/index.md` is the full catalog (35 published workflows at last
count) — this table is not a copy of it and will not be kept in sync with
every addition; consult the catalog directly when the intent doesn't
obviously match a row above. One entry worth knowing before you route to
it: `grilling`/`grill-with-docs` are published workflows whose design calls
for a live back-and-forth checkpoint, but this host's harness has been
measured completing both stages autonomously with zero pauses for input —
routing an intent that actually needs a mid-run human answer there gets you
the workflow's best guess, not a conversation. <!-- honesty-vision:F2 / R-NS-6 -->

## Standard workflow loop

1. **Load estate context.** `sgt doctor` (install/estate health, named
   remedies) and `sgt repo list` (what's declared) before acting — never
   infer which repo or estate you're in from the current directory.
   <!-- BU-0001, BU-0002 -->
2. **Check running work.** `sgt status` (daemon health, counts by state)
   and `sgt work list` (the fleet: id, state, intent) — reuse or resume a
   matching Work item instead of creating a duplicate for the same intent.
   <!-- BU-0046, BU-0048 -->
3. **Shape the intent.** Free text is legal and sufficient at the CLI;
   structured fields (objective, repos-or-group, acceptance, exclusions,
   workflow) are progressive elaboration you add when they sharpen the
   ask, not a form you must fill.
4. **Choose a workflow.** Explicit (`--workflow <name>`) or let `sgt run`
   fall back to the workspace's own `software-change` workflow, then the
   built-in default. Use the routing table and `.sergeant/index.md` above.
5. **`sgt run "<intent>"` with envelope flags.** `--repo`/`--group` (which
   repositories), `--backend`/`--profile` (which adapter and launch
   profile), `--turns`/`--ceiling-secs` (override this Work's turn and
   wall-clock envelope). A workflow declaring an interactive-ask stage on a
   backend whose capability doesn't support it is refused at submit, not
   discovered mid-run.
6. **Monitor.** `sgt work show <id>` (stage, execution, surface, output
   pointer, recent events) and `sgt work transcript <id>` (the decoded
   conversation) — a Work item isn't progressing merely because a process
   for it exists; trust the journal-backed state these surfaces read, not
   liveness alone. `in_progress`, `needs_input`, and `blocked` are all
   nonterminal. <!-- BU-0036, BU-0038, BU-0047, BU-0111, BU-0115 -->
7. **Respond to `needs_input`.** `sgt respond <id> "<answer>"` — reserved
   for genuine human-judgment gates (product, security, destructive-action,
   ambiguity a mechanical check can't resolve), not relayed for findings a
   workflow could apply itself. `sgt retry`/`sgt extend` re-enter a stage
   only after reading its actual current state, never blind. <!-- BU-0039, BU-0112 -->
8. **Collect.** The output pointer in `sgt work show <id>` names the
   branch and every artifact's home; a plan, a dispatched Work item, or a
   status report is not the delivered outcome unless that's literally all
   that was asked for. <!-- BU-0017, BU-0044, BU-0045 -->

## ICM procedure discipline

While a workflow stage is active, follow only the stage context Sergeant
actually supplies (its `CONTEXT.md`) — not a recollection of the workflow's
other stages or a prior run. Treat `.sergeant/common/scripts/` and
workflow-local `scripts/` as helpers invoked while crossing a checkpoint,
never as independent procedure, unless the workflow explicitly declares
them a durable stage. Never treat anything under
`.sergeant/drafts/workflows/` as published procedure — it is
human-reviewable evidence for promotion, not runnable by name, regardless
of how complete it looks (`docs/icm/convention.md` §2). Use `sgt respond`,
`sgt retry`, `sgt cancel`, and the inspection surfaces above to reflect
workflow state — never fabricate a stage transition, an answer, or a
completion in prose that the journal doesn't actually contain.

## When NOT to use `sgt`

A single-turn ask — answer a question, read a file, make a small edit in
the current session with no need to survive a restart or run unattended —
does not need the Work apparatus. The harness (this session) owns that
routing judgment; sergeant's core makes no claim about it (North Star
ruling 4). Reach for `sgt run` when the work should be durable, resumable,
and reviewable independent of this conversation continuing — not by
default for everything.

## Working on sergeant-rs itself

Changing sergeant-rs's own code, tests, or CI is repo content, not a
separate product: the rules live in **`docs/DEVELOPMENT.md`** (build
commands, architecture invariants, testing rules, the shipping gate,
per-host environment facts). Load it before touching `src/`, `tests/`,
`scripts/`, or CI config. It still applies in full when working directly
in this session rather than through a dispatched Work item — no mode
waives tests, review, or the shipping gate. <!-- BU-0018, BU-0113, BU-0114 -->

## Guardrails

- `sgt init`/`sgt repo add`/`sgt group add` write only within the estate
  they scaffold (`sergeant.toml`, `repos/`) — never to another harness's
  own configuration, and never to `AGENTS.md` or `CLAUDE.md` in any repo.
  Re-running `sgt init` on an already-initialized estate is a no-op, not a
  reset. <!-- BU-1263, BU-1264, BU-1295 -->
- Prefer the `sgt` verbs above over ad hoc shell reconstructions of the
  same operation; fall back to manual steps only when no verb covers it,
  and say so plus the exact error when you do. <!-- BU-0019, BU-0021 -->
- A missing tool or capability surfaces as `sgt doctor`'s named remedy,
  never a silent skip or an invented workaround. <!-- BU-0049, BU-0265, BU-1260, BU-1261 -->
- Standing authorization to proceed without re-confirming every step never
  extends to skipping the shipping gate, forcing a git operation, exposing
  a secret, or destroying preserved state (a retained branch, a journal, a
  Work record). <!-- BU-0050 -->
- Secrets never enter a commit, a project/estate config file, or workflow
  output — paths and names are fine, credentials are not. <!-- BU-0055, BU-0259 -->

## Skills, procedures, and the corpus this file draws on

For every listed trigger above, load the named skill or workflow file
directly and treat it as canonical — a harness's own skill registry
omitting it does not make it unavailable, and its omission is never a
reason to stop or ask. Stop and report the exact path only when the file
itself is absent or unreadable; don't reconstruct the procedure from
memory in that case. <!-- BU-0022, BU-0023, BU-0024 -->

Most of the 126-unit corpus this rewrite consumed belongs to specific
workflows (`tdd`, `codebase-design`, `prototype`, `wayfinder`, `to-tickets`,
`triage`, `diagnose-bug`, `domain-modeling`) rather than to this always-on
file — the full per-unit disposition, including everything ruled
not-adopted and why, is `docs/icm/agents-invariant-dispositions.md`.
