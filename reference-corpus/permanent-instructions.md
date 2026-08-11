# Permanent Instructions — Small-Constitution Candidate

Part of the N1 reference corpus (`docs/gauntlet/contracts/N1.md`, §8.1's
`permanent-instructions.md`). Built from `synthesis.md` §2 ("Permanent-instruction
set") by expanding each of its ten articles into individually cited lines, per
this milestone's binding rule that an uncited line is invention. Every line below
cites the `behavior-units/P*.ndjson` id(s) it is evidenced by; the appendix
resolves each id to its source path and locator (the id's own record additionally
carries the `quote_hash` — not repeated here to avoid a second copy of data that
already lives in one place, per Article VII's own single-owner rule).

**Method note.** These are *article candidates for* the repository's `AGENTS.md`
small constitution (`docs/icm/convention.md` §3), not a drop-in replacement for
it — convention.md's sample text is a different, already-minimal illustration.
This document is the traceable intermediate artifact: 103 `agents-invariant`
units → 10 articles → an editor can now compress each article to one or two
`AGENTS.md` sentences (`synthesis.md`'s sizing note: 103 units → roughly 45
sentences), confident every compressed sentence still has its evidence.

103 units classified `agents-invariant`. Two — BU-P1-062, BU-P1-063 — are
**not** represented below; they are product-positioning/attribution statements
that fail Article VII's own observability test (no trigger, no required/prohibited
action, no compliance evidence) and are recorded in `synthesis.md` §7 as
unassigned rather than silently folded in here.

---

## Article I — Resolve before acting

- Establish repository ownership, roles, and inherited instructions from
  resolved project context — never from the current working directory or from
  inference. `[BU-P1-001, BU-P8-052]`
- The layered instruction set governs before the first mutation. `[BU-P1-001]`

## Article II — Roles and execution mode

- The primary session coordinates multi-repository work by default; it may
  implement directly only when the user explicitly asks to work in-session, and
  one repository owns the complete outcome. `[BU-P1-002]`
- Never use the coordinator role as a reason to stop at a plan, status report,
  or dispatch suggestion when the user asked for an implemented outcome.
  `[BU-P1-015]`
- A plan, task, finding, or worker launch is not the requested outcome unless
  the user asked only for planning or dispatch. `[BU-P1-044]`

## Article III — Authority boundaries and ownership

- The shipping gate (`no-mistakes`) is coordinator-owned: workers and
  remediation loops never invoke it directly; it is vendored to workers only so
  they can understand the contract a brief references. `[BU-P1-040, BU-P1-131]`
- `no-mistakes` validates one explicit final shipping boundary, not an
  implementation loop — implementation, native tests, lint, and independent
  review must be complete before the coordinator starts a single run on a
  committed branch. `[BU-P1-068, BU-P1-073, BU-P1-111]`
- The coordinator launches the gate interactively and never passes `--yes`; any
  narrowing of its default validation scope is an explicit, fully-replacing
  skip list, never a silent reduction. `[BU-P8-083]`
- Validation agents never modify source while reporting findings; every
  actionable finding is routed to separate owning-repository remediation work,
  and an auto-fix is never authorized inside the validation-only workflow.
  `[BU-P1-110, BU-P8-091, BU-P8-100]`
- Never modify repositories under `~/.config/sergeant/` — that is
  configuration, not code. `[BU-P1-054]`
- Standing authorization removes repetitive confirmation only; it never
  authorizes risk acceptance, gate skipping, force operations, secret
  exposure, or destruction of preserved state. `[BU-P1-050]`

**Contested (X4):** whether the worker-invocation prohibition above is
runtime-enforceable or actor-honored-only is argued three ways across the
corpus — as an absolute (`BU-P1-040`, `BU-P1-131`, `BU-P8-082`), as an explicit
user override that must still be rendered verbatim into a worker's brief
(`BU-P7-112`), and as a rejected engine-gap claim (`G8`, `BU-P2-056`). See
`synthesis.md` §5 G8 and §6 X4; the ruling stands as stated above, with the
authority-model observation preserved for a future permission-capability
design.

## Article IV — Evidence over optimism; fail closed, never fail silent

- Do not infer progress from liveness: a live process or pane is not proof of
  work, and a worker is not active merely because its process exists — require
  recent, meaningful progress evidence with a defined fallback chain.
  `[BU-P1-037, BU-P1-047, BU-P8-072]`
- Completion requires its substantiating artifact: a terminal `done` status
  needs a non-empty result; an open PR or a single successful turn is not, by
  itself, terminal completion. `[BU-P7-011]`
- Do not rewrite an expected blocked exit as orphaned, and do not clean a
  waiting or blocked worktree merely because its process ended.
  `[BU-P1-037, BU-P8-096]`
- Tool absence produces an actionable fallback or an explicit blocker — never a
  silent skip, a false success, or an indefinite wait. Use supported commands
  before any manual process/tmux/Git/fleet-file operation, and preserve exact
  errors and state before attempting recovery. `[BU-P1-049, BU-P8-094]`
- A durable fact is settled only from the owning party's own proof, never from
  inference: a lock is reclaimed only on proof its owner is dead, not on
  suspicion; an action lease is marked complete only from the agent's own
  durable proof, and any mismatch — identity, nonce, target, missing proof —
  fails closed as pending with its reason. `[BU-P6-053, BU-P6-055]`
- An undiagnosable failure is not actionable: every lock-timeout report names
  the specific owner and the exact recovery action. `[BU-P6-061]`
- A read-only observation surface never mutates state as a side effect of
  being queried. `[BU-P6-102]`
- A worker counts as drained only when its exit is provable; an identity that
  was never durably recorded blocks the wait rather than being counted as
  drained. `[BU-P8-078]`
- A live, unreleased owner is never displaced by a claim; a claimant must prove
  it truly runs where it claims to, by walking its own process ancestry —
  never by asserting an environment variable. `[BU-P8-088]`
- Rollback undoes only what this invocation created and can still prove it
  owns; preexisting, reused, or concurrently replaced state is preserved
  untouched. `[BU-P8-090]`
- A pending response is never overwritten, and recovery tooling is never used
  against an active response generation; if a response was already applied,
  converge by rerunning the same acknowledgement command rather than
  intervening manually. `[BU-P8-097]`
- Cleanup is never forced by manually deleting fleet files, and deliberately
  refuses while a handshake could still be completed. `[BU-P8-104]`
- Cleanup can only retire — never truly acknowledge — an unfinished handshake,
  and only when the owning task is closed and the worker is provably dead by
  every independent proof together. `[BU-P8-105]`
- Retirement records the exact partial state before mutating anything, never
  writes an acknowledgement, and is permanently and structurally distinguished
  from a real acknowledgement. `[BU-P8-106]`
- Cleanup refuses to trust a retirement archive that no longer describes the
  state it originally preserved. `[BU-P8-107]`

## Article V — Measured, not assumed

- Capability is discovered by probing the installed thing's own surface — a
  command's own `--help` output, not a version number or a name; an
  unrecognized executable sharing a trusted name is rejected rather than
  silently wrapped. `[BU-P6-046, BU-P8-046]`
- A harness that is not installed is recorded as unmeasured — never as
  unsupported. `[BU-P1-059]`
- A pinned model/provider/variant tuple the resuming harness cannot honor is a
  terminal failure, checked before any durable state is created — never a
  silent fallback to an ambient default model. `[BU-P6-108, BU-P8-061]`
- One declaration drives the admission gate, the readiness probe, and the
  invocation together — never three independently maintained definitions.
  `[BU-P6-066]`

## Article VI — Secrets, privacy, and transport

- Never commit secrets; project configuration may contain paths but never
  credentials, and documentation examples never contain real credentials,
  private repository names, prompt or response bodies, or secret-bearing
  environment values. `[BU-P1-055, BU-P8-006]`
- No delivered content — briefs, responses, intent bodies, prompts — appears
  in process arguments; the transport decision (private file path vs. argv) is
  made once by the coordinator and honored exactly by the worker, never
  re-optimized at run time. `[BU-P8-065, BU-P6-045]`
- The argv transport is reachable only through explicit, per-invocation
  operator consent — never an environment variable that could be silently
  exported once and reapplied to later runs. `[BU-P6-047, BU-P8-086]`
- Recording that a response was delivered never writes the response text
  itself into tracked work — only an opaque identifier and a note that the
  text moved through a separate atomic transport. `[BU-P6-038]`
- A registered callback origin record persists only a correlation id, a
  profile name, and a version tag — never request text, platform IDs, tokens,
  secrets, message content, callback commands, or logs; a callback payload
  carries only concise status, a decision question, or completion evidence.
  `[BU-P8-011, BU-P8-018]`
- Callback executables come only from a fixed, pre-installed profile directory
  — never from fleet state, a request, or project configuration.
  `[BU-P8-009, BU-P8-031]`

**Contested (X18):** `BU-P8-065`/`BU-P8-085` state the argv prohibition as
absolute ("no delivered content *ever* appears in process arguments");
`BU-P6-047`/`BU-P8-086` state a documented, consent-gated exception to that
same absolute. Both are written as invariants in the same document family.
Recorded as an open contradiction, not silently reconciled — see
`synthesis.md` §6 X18.

## Article VII — Instruction and documentation authority

- Every directive names a trigger, a required or prohibited action, or the
  evidence that proves compliance; vague quality directives ("be thorough",
  "best practices") are prohibited and must be replaced by named commands,
  failure behavior, acceptance criteria, ownership, or review evidence —
  remove any sentence that cannot change a decision or be checked afterward.
  `[BU-P1-017, BU-P1-018, BU-P7-018]`
- Authority is single-owner: `AGENTS.md` alone owns always-on execution and
  safety policy (including that direct-mode executors never edit the default
  branch and always open a PR); skill files alone own trigger-specific
  procedure; `docs/schema.md` alone owns project configuration fields and path
  resolution — documentation never forks or restates any of them.
  `[BU-P8-002, BU-P7-014, BU-P8-003, BU-P8-004, BU-P5-104]`
- A command's own `--help`, its emitted contract, and its tests outrank prose;
  a prose/behavior disagreement is filed as tracked work, never silently
  resolved either way. `[BU-P8-005]`
- State explicitly when a behavior is undocumented or contradictory rather
  than inventing a command, flag, state transition, or safety guarantee.
  `[BU-P5-123]`

## Article VIII — Procedure discovery and loading

- Load a procedure only when its trigger applies; the repository-local
  procedure file is canonical and outranks any same-named registry entry, and
  a registry's omission never makes a procedure unavailable.
  `[BU-P1-021, BU-P1-022, BU-P1-023]`
- When a procedure file is absent or unreadable, stop and report only its
  exact expected path — never reconstruct a protocol from memory.
  `[BU-P1-024]`
- Procedures are executable instructions: review the source before installing
  or updating a skill, and never infer its provenance from a folder name —
  check the lock file, plugin metadata, or the source repository instead.
  `[BU-P1-112, BU-P1-114]`
- No install step writes to a user's global agent configuration; editable
  skill installs and managed read-only plugin bundles are distinct routes, and
  files owned by the managed route are never hand-edited.
  `[BU-P1-128, BU-P1-116]`
- Skills split into user-invoked orchestrators, explicitly selected by the
  user, and model-invoked disciplines, which load automatically whenever their
  trigger matches. `[BU-P1-118]`
- If a toolbelt command covers an operation, use it instead of reproducing the
  operation with ad hoc shell commands; fall back to manual operations only
  when no command covers it or the command returns an explicit
  unsupported-case error, and report that fallback while preserving the
  original error evidence. `[BU-P1-019, BU-P1-020]`
- Use a bare `sgt-*` command when it resolves on `PATH`; otherwise run the
  matching script from this repository's own `bin/` directory. `[BU-P1-056]`
- A generated worker brief does not require a global skill installation for
  its core workflow — the required bundle is vendored directly in the
  repository. `[BU-P1-115]`

## Article IX — Scope, deployment model, and delivery discipline

- Sergeant is a single-user, local-first orchestrator: one installation per
  developer, with its own local configuration, credentials, workers,
  worktrees, and fleet state — never a shared team service, central tenancy,
  organization RBAC, shared credentials, cross-machine leases, or team-wide
  fleet database. `[BU-P8-001, BU-P1-098, BU-P1-100, BU-P1-064]`
- Sergeant is not a centralized team-orchestration service, not a replacement
  for the forge, Git, CI, or the tracker, and never permission to push
  directly to a default branch; the distribution contains no remote-execution
  contract anywhere. `[BU-P1-109, BU-P7-019]`
- Direct-mode implementation always uses a feature branch and always opens a
  PR — never the default branch. `[BU-P8-057]`
- No harness-specific conversation-injection plugin is installed; worker
  updates surface only through durable fleet state. `[BU-P8-043]`
- Do not duplicate tasks, findings, PRs, workers, or review passes when a
  canonical owner already exists; do not repeatedly report an already-approved
  blocker — execute the next safe step instead; do not leave completed,
  merged, blocked, or abandoned work recorded as in-progress.
  `[BU-P1-046, BU-P1-045, BU-P1-048]`

## Article X — Deferred work, recovery, and installation integrity

- Deferred work is a durable waiting state with a recorded condition, never an
  in-process sleep. `[BU-P8-074]`
- A wake condition that can no longer be met converts the work to needs-input
  with the remedy stated, rather than retrying blindly until a deadline.
  `[BU-P8-076]`
- Recovery is one-shot per attempt: it escalates to needs-input rather than
  retrying indefinitely. `[BU-P8-081]`
- Interactive per-action permission gates are appropriate only for
  human-supervised sessions; an automated dispatched worker must never depend
  on a live prompt it cannot answer, so it runs with permissions bypassed by
  deliberate policy — scoped to the intent and brief reviewed and approved at
  dispatch time, not itself a capability grant.
  `[BU-P8-067, BU-P6-070, BU-P8-068]`
- The canonical intent record is the single source for implementation, review,
  PR text, and validation; only an audited human decision creates a new
  revision, and successor or recovery work inherits the exact same revision.
  `[BU-P7-006]`
- The durably recorded event is the source of truth for a worker update; any
  live delivery transport is an optional layer on top, and its failure to
  reach a coordinator is never treated as a failure to record the update.
  `[BU-P6-028]`
- A notified worker applies a decision exactly once and restores truthful
  status; rerunning the same acknowledgement converges existing state rather
  than reapplying the decision. `[BU-P8-080]`
- Shared credentials are never switched globally while unrelated runs are
  active; use an approved repo-scoped method, wait, or an explicit manual
  override instead. `[BU-P8-101]`
- Portability and test-isolation guarantees are proven by running under the
  target environment, not by parsing; every suite touching drain or fleet
  state must declare its own isolation, and a suite that aborts before its
  mutating lines is not silently treated as passing.
  `[BU-P7-033, BU-P7-071, BU-P1-065, BU-P7-020, BU-P7-024, BU-P7-077, BU-P7-080]`
- Sergeant refuses a cross-filesystem layout between fleet state and
  worktrees rather than silently falling back from an atomic rename to a
  non-atomic copy. `[BU-P8-108]`

**Contested (X5):** whether the Bash-3.2 portability target above is a live
operating invariant (`BU-P7-033`, `BU-P7-071`, `BU-P1-065`) or an obsolete,
distribution-specific target a compiled binary makes moot (`BU-P8-102`,
`obsolete-mechanisms.md` M9) is unresolved in the source corpus. sergeant-rs
itself is the compiled-binary side of that argument; recorded here as evidence
this milestone did not silently pick a side without citing both.

**Contested (X1):** whether Sergeant refuses a cross-filesystem layout
(`BU-P8-108`, stated above) or must support a copy-based fallback with a
CRITICAL rollback diagnostic (`BU-P7-079`, a rejected engine-gap claim, `G9`)
is unresolved between a doc and a test; the durable rule adopted above follows
the doc because the test's own engine-gap claim was rejected (`synthesis.md`
§5 G9, §6 X1).

---

## Sizing note

103 units → 10 articles → roughly 45 distinct sentences once each article
above is compressed for `AGENTS.md` itself (per `synthesis.md` §2's own sizing
observation). The compression is concentrated in Articles IV, VI, and VIII,
where independent extractors correctly recorded the same underlying rule
separately because it recurs verbatim across `AGENTS.md`, `README.md`,
`docs/`, and a test file.

---

## Appendix — citation resolution

Every `BU-*` id cited above, resolved to its source path and locator in
`reference/sergeant-upstream` (pinned at the SHA in `reference/UPSTREAM.md`).
The full record — including `quote_hash`, `scope`, `trigger`, `outcome`, and
extraction `notes` — is the canonical copy, in
`reference-corpus/behavior-units/P{1,5,6,7,8}.ndjson`; this table exists only
to make the citations above independently checkable without opening eight
files.

| Unit | Source path | Locator |
|---|---|---|
| BU-P1-001 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L3-5, opening paragraph |
| BU-P1-002 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L9-13, 'Your role' section |
| BU-P1-015 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L38-39 |
| BU-P1-017 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L45-49, Instruction quality |
| BU-P1-018 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L51-54, Instruction quality (vague directives) |
| BU-P1-019 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L60-61, Toolbelt preface |
| BU-P1-020 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L99-103, Toolbelt fallback rule |
| BU-P1-021 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L109, Procedural skills preface |
| BU-P1-022 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L122-124, skill precedence |
| BU-P1-023 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L125-126 |
| BU-P1-024 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L127-128 |
| BU-P1-037 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L148, waiting anti-patterns |
| BU-P1-040 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L152 |
| BU-P1-044 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L161-162, avoid no-op (a) |
| BU-P1-045 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L163-164, avoid no-op (b) |
| BU-P1-046 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L165-166, avoid no-op (c) |
| BU-P1-047 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L167-168, avoid no-op (d) |
| BU-P1-048 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L169-170, avoid no-op (e) |
| BU-P1-049 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L171-172, avoid no-op (f) |
| BU-P1-050 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L173-175, avoid no-op (g) |
| BU-P1-054 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L182 |
| BU-P1-055 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L183 |
| BU-P1-056 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L184-185 |
| BU-P1-059 | `reference/sergeant-upstream/AGENTS.md` | AGENTS.md L187, measured transports |
| BU-P1-062 | `reference/sergeant-upstream/README.md` | README.md L7-9, Genesis (unassigned — see §7 note above) |
| BU-P1-063 | `reference/sergeant-upstream/README.md` | README.md L9, narrowed focus (unassigned — see §7 note above) |
| BU-P1-064 | `reference/sergeant-upstream/README.md` | README.md L17-19, What it is |
| BU-P1-065 | `reference/sergeant-upstream/README.md` | README.md L21 |
| BU-P1-068 | `reference/sergeant-upstream/README.md` | README.md L264, no-mistakes framing |
| BU-P1-073 | `reference/sergeant-upstream/README.md` | README.md L277 |
| BU-P1-098 | `reference/sergeant-upstream/docs/what-is-sergeant.md` | docs/what-is-sergeant.md L8-13, audience |
| BU-P1-100 | `reference/sergeant-upstream/docs/what-is-sergeant.md` | docs/what-is-sergeant.md L23-24 |
| BU-P1-109 | `reference/sergeant-upstream/docs/what-is-sergeant.md` | docs/what-is-sergeant.md L76-82, non-goals |
| BU-P1-110 | `reference/sergeant-upstream/docs/what-is-sergeant.md` | docs/what-is-sergeant.md L81-82 |
| BU-P1-111 | `reference/sergeant-upstream/docs/what-is-sergeant.md` | docs/what-is-sergeant.md L90 |
| BU-P1-112 | `reference/sergeant-upstream/docs/skills.md` | docs/skills.md L3-4 |
| BU-P1-114 | `reference/sergeant-upstream/docs/skills.md` | docs/skills.md L19-20 |
| BU-P1-115 | `reference/sergeant-upstream/docs/skills.md` | docs/skills.md L24-27 |
| BU-P1-116 | `reference/sergeant-upstream/docs/skills.md` | docs/skills.md L55-57 |
| BU-P1-118 | `reference/sergeant-upstream/docs/skills.md` | docs/skills.md L88-100, orchestrator vs discipline distinction |
| BU-P1-128 | `reference/sergeant-upstream/docs/repo-scoped-skills.md` | docs/repo-scoped-skills.md L3-6 |
| BU-P1-131 | `reference/sergeant-upstream/docs/repo-scoped-skills.md` | docs/repo-scoped-skills.md L38-40 |
| BU-P5-104 | `reference/sergeant-upstream/skills/load-project/SKILL.md` | lines 51-52 |
| BU-P5-123 | `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` | lines 51-52 |
| BU-P6-028 | `reference/sergeant-upstream/bin/sgt-notify` | L38-39, L51-60 |
| BU-P6-038 | `reference/sergeant-upstream/bin/sgt-td-memory` | L116-127 |
| BU-P6-045 | `reference/sergeant-upstream/bin/sgt-validation-worker` | L44-53 |
| BU-P6-046 | `reference/sergeant-upstream/bin/_sgt-intent.sh` | L19-21 |
| BU-P6-047 | `reference/sergeant-upstream/bin/_sgt-intent.sh` | L31-37 |
| BU-P6-053 | `reference/sergeant-upstream/bin/_sgt-drain.sh` | L148-152 (rationale shared with `bin/_sgt-response-lock.sh`'s identical lock design) |
| BU-P6-055 | `reference/sergeant-upstream/bin/_sgt-response-lock.sh` | L251-256 |
| BU-P6-061 | `reference/sergeant-upstream/bin/_sgt-drain.sh` | L366-369 |
| BU-P6-066 | `reference/sergeant-upstream/bin/_sgt-harness.sh` | L9-11 |
| BU-P6-070 | `reference/sergeant-upstream/bin/_sgt-harness.sh` | L63-72 |
| BU-P6-102 | `reference/sergeant-upstream/bin/sgt-watch` | L42-45 |
| BU-P6-108 | `reference/sergeant-upstream/bin/sgt-interactive-worker` | L44-49 |
| BU-P7-006 | `reference/sergeant-upstream/templates/worker-brief.md` | section '### 1. Pin scope and source of truth' |
| BU-P7-011 | `reference/sergeant-upstream/templates/worker-brief.md` | section '### 4. Escalate and resume' |
| BU-P7-014 | `reference/sergeant-upstream/tests/instruction-policy-test.sh` | lines 48-49 |
| BU-P7-018 | `reference/sergeant-upstream/tests/instruction-policy-test.sh` | lines 102-113 |
| BU-P7-019 | `reference/sergeant-upstream/tests/no-remote-test.sh` | lines 5-12 |
| BU-P7-020 | `reference/sergeant-upstream/tests/global-state-isolation-test.sh` | lines 5-13 |
| BU-P7-024 | `reference/sergeant-upstream/tests/global-state-isolation-test.sh` | lines 178-185 |
| BU-P7-033 | `reference/sergeant-upstream/tests/runtime-bash-test.sh` | lines 20-36 |
| BU-P7-071 | `reference/sergeant-upstream/tests/sgt-dispatch-bash32-test.sh` | lines 1-9 |
| BU-P7-077 | `reference/sergeant-upstream/tests/sgt-dispatch-worker-test.sh` | lines 333-334 |
| BU-P7-080 | `reference/sergeant-upstream/tests/sgt-cleanup-test.sh` | lines 138-145 |
| BU-P8-001 | `reference/sergeant-upstream/docs/README.md` | L3-6 |
| BU-P8-002 | `reference/sergeant-upstream/docs/README.md` | L30 (Documentation authority) |
| BU-P8-003 | `reference/sergeant-upstream/docs/README.md` | L31 (Documentation authority) |
| BU-P8-004 | `reference/sergeant-upstream/docs/README.md` | L32 (Documentation authority) |
| BU-P8-005 | `reference/sergeant-upstream/docs/README.md` | L34-36 (Documentation authority) |
| BU-P8-006 | `reference/sergeant-upstream/docs/README.md` | L38-39 |
| BU-P8-009 | `reference/sergeant-upstream/docs/callbacks.md` | L15 |
| BU-P8-011 | `reference/sergeant-upstream/docs/callbacks.md` | L44-49 |
| BU-P8-018 | `reference/sergeant-upstream/docs/callbacks.md` | L87-90 |
| BU-P8-031 | `reference/sergeant-upstream/docs/schema.md` | L21-24 |
| BU-P8-043 | `reference/sergeant-upstream/docs/getting-started.md` | L82-83 |
| BU-P8-046 | `reference/sergeant-upstream/docs/getting-started.md` | L141-142 |
| BU-P8-052 | `reference/sergeant-upstream/docs/using-sergeant.md` | L3-12 (Start with project context) |
| BU-P8-057 | `reference/sergeant-upstream/docs/using-sergeant.md` | L24 (Direct mode, step 3) |
| BU-P8-061 | `reference/sergeant-upstream/docs/using-sergeant.md` | L65-66 |
| BU-P8-065 | `reference/sergeant-upstream/docs/using-sergeant.md` | L83-86 |
| BU-P8-067 | `reference/sergeant-upstream/docs/using-sergeant.md` | L99-102 (Security posture) |
| BU-P8-068 | `reference/sergeant-upstream/docs/using-sergeant.md` | L103-106 |
| BU-P8-072 | `reference/sergeant-upstream/docs/using-sergeant.md` | L161-172 (Worker states) |
| BU-P8-074 | `reference/sergeant-upstream/docs/using-sergeant.md` | L188-190 (Resume deferred work) |
| BU-P8-076 | `reference/sergeant-upstream/docs/using-sergeant.md` | L216-222 |
| BU-P8-078 | `reference/sergeant-upstream/docs/using-sergeant.md` | L241-243 |
| BU-P8-080 | `reference/sergeant-upstream/docs/using-sergeant.md` | L272-281 (Respond to a worker) |
| BU-P8-081 | `reference/sergeant-upstream/docs/using-sergeant.md` | L283-296 (Recover one stalled worker) |
| BU-P8-083 | `reference/sergeant-upstream/docs/using-sergeant.md` | L318-327 |
| BU-P8-086 | `reference/sergeant-upstream/docs/using-sergeant.md` | L340-346 |
| BU-P8-088 | `reference/sergeant-upstream/docs/using-sergeant.md` | L359-374 (Coordinator ownership and handover) |
| BU-P8-090 | `reference/sergeant-upstream/docs/using-sergeant.md` | L384-387 |
| BU-P8-091 | `reference/sergeant-upstream/docs/using-sergeant.md` | L392-395 |
| BU-P8-094 | `reference/sergeant-upstream/docs/troubleshooting.md` | L3-4 |
| BU-P8-096 | `reference/sergeant-upstream/docs/troubleshooting.md` | L72-74 (Worker became orphaned after blocking) |
| BU-P8-097 | `reference/sergeant-upstream/docs/troubleshooting.md` | L77-86 (Response already pending) |
| BU-P8-100 | `reference/sergeant-upstream/docs/troubleshooting.md` | L102-113 (no-mistakes is parked) |
| BU-P8-101 | `reference/sergeant-upstream/docs/troubleshooting.md` | L114-116 |
| BU-P8-104 | `reference/sergeant-upstream/docs/troubleshooting.md` | L154-164 (Cleanup refuses or state is partial) |
| BU-P8-105 | `reference/sergeant-upstream/docs/troubleshooting.md` | L166-186 (Cleanup refuses an unfinished response handshake) |
| BU-P8-106 | `reference/sergeant-upstream/docs/troubleshooting.md` | L188-197 |
| BU-P8-107 | `reference/sergeant-upstream/docs/troubleshooting.md` | L199-201 |
| BU-P8-108 | `reference/sergeant-upstream/docs/troubleshooting.md` | L203-209 (fleet state and worktree must be on the same filesystem) |
