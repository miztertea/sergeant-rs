# MVP-1 — CORE — **DRAFT, not adjudicated**

**Owner adjudication gate: no build until adjudicated.** Every ruling here is a
hypothesis with provenance — argue it wrong with evidence, do not ratify it. Two
(R-MVP1-2, R-MVP1-11) contradict already-approved text and say so in place.

Governing, cited never restated: `NORTH-STAR.md`; `notes/mvp-bucketing-2026-08-11.md`
v3 (MVP-1 table, three ruled escalations, A1–A9);
`notes/estate-manifest-design-2026-08-11.md` v2; `reference/review-northstar-outside-codex.md`
(findings 1–4, §3 self-hosting); `goal-prompt-mvp.md` §3–4;
`reference/notes/gauntlet-pattern.md`; `docs/icm/convention.md` §1a; `CLAUDE.md`,
`GAUNTLET.md`, `LESSONS.md` (L1, L6, L7, L16, L18 bind hardest). Proposal sections
by number as usual.

## Outcome

The substrate the delegation loop stands on: a manifest parsing fail-closed in the
new vocabulary and discoverable from inside a member repo; a work surface able to
live outside the checkout it modifies — what makes sergeant runnable against
sergeant; a bound Work pinning its repository set and instruction policy; bounded
execution (turn count, per-turn wall clock) at every verb that spawns a turn; a Work
whose branch and outputs are findable without decoding the journal and whose declared
dispositions execute; a test instrument that can stand inside a turn. Plus two
standalone closures: terminal-work eviction (#4) and the blocked exit-door invariant.

**Environment facts relied on** (`docs/environments/cerberus.md`, 2026-08-11): DAC
permission bits enforced (fault fixtures work here, unlike the root container); Claude
CLI 2.1.227 with **`post_turn_summary` absent — the ask affordance measured withdrawn
on this host**; cargo/rustc and `claude` off the non-interactive PATH.

## RULINGS

### R-MVP1-1 — `data_dir` / `surfaces_root` split (A1) — R2

**Ruled.** `surfaces_root` becomes a path distinct from `data_dir`, defaulting to
`<data_dir>/surfaces` (today's layout — nothing moves), overridable by
`[estate] surfaces_dir` / `SGT_SURFACES_DIR`; `SurfacePlan::new` and `materialize`
take it, `Engine` carries it beside `data_dir` (`engine.rs:635,643,913,919`). When
MVP-3 flips the data dir estate-local (U-R2), `surfaces_root` defaults **outside every
checkout** (`${XDG_STATE_HOME:-$HOME/.local/state}/sergeant/surfaces`).

**Why.** Our own tests pin the contradiction: `materialize_one` refuses a worktree
under its source repo (`surface.rs:330-338`; tests `:680`, `:854`), so an in-estate
data dir makes self-hosting illegal by construction. Rejected — option 2 (estate root
untargetable) kills the recursion proof, option 3 (external data dir) defers rather
than decides. R2: `surface_root()` already localizes the join.

**Pin.** Existing refusal tests unchanged; data dir *inside* the checkout with
`surfaces_root` outside materializes; `surfaces_root` inside still refuses (the guard
moved, not weakened); `surface.planned`'s `root` reflects it. The MVP-3 default flip
must not precede this.

### R-MVP1-2 — promote/finalize: the four-meanings fork (A6 / Finding 4) — R2

**Ruled: meaning (3) — the workflow's closing actor stage invokes a deterministic
shared finalize helper; the engine learns no output vocabulary.** Declaration input:
each stage's `output/README.md` disposition line (§1a), `promote` | `evidence`,
silence promotes nothing. Owner: workflow content. Timing: inside the closing stage,
before terminal state and therefore before teardown, while the worktree exists.
Failure: a non-zero helper exit fails that stage down the ordinary stage-failure path
— `failed` with the helper's diagnostic, branch still retained.

**Rejected.** (1) *Engine understands output declarations* — moves §1a policy into the
engine against §12's "procedure is data", inventing an event kind and failure semantics
for a convention that has run on one workflow. (2) *A `kind = "execute"` finalize
stage* — `StageKind::Execute` is a doc-stub (`workflow.rs:90-98`) landing in MVP-2/N4,
and a CORE row depending on ADAPTERS is the inversion the outside review flagged; it is
the designated migration target instead (same helper, new invoker, no engine change).
(4) *Report-only* — that is the output-pointer sibling below; alone it leaves "silence
promotes nothing" unexecuted, the E6 defect itself. R2 because a working helper exists
(`.sergeant/workflows/repo-to-icm/scripts/finalize.py`, with its own evidence-guard
test) but workflow-local: the work is generalizing it plus one closing-stage
instruction per workflow that declares outputs.

**Owner ratification needed.** `NORTH-STAR.md` gives Core "promote/finalize", reading
as meaning (1). This narrows it to *core owns the declaration record and the pointer;
workflow content executes the disposition*. If the owner holds that line, this ruling
is void and the row moves to MVP-2 behind `kind = "execute"`.

**Pin.** A fake run of a two-stage workflow with one `evidence` and one `promote`
output leaves only the promoted file on the retained branch, the removal in branch
history; reverting the closing-stage instruction leaves the evidence file (L7).

**Sibling in scope, not a ruling — the output pointer (E6's other half):** terminal
state and `work show` name, per repository, the source repo, retained branch, worktree
path, finalize commit. Retention already works (`surface.rs`); the pointer is missing.

### R-MVP1-3 — schema rename-with-refusal mechanics — R3

**Ruled.** `WorkspaceFile` (`workspace.rs:172-198`) becomes `estate`/`repo`/`profile`,
still `deny_unknown_fields`; `[[profile]]` keeps its array-of-tables shape;
`[group.<name>]` is new. Legacy vocabulary is not merely unknown: `[workspace]` or
`[[repository]]` raises a **named migration refusal**
(`LegacyVocabulary { file, found, expected, remedy }`), and mixing hits it on the
first legacy key. Every fixture, example and in-repo `sergeant.toml` migrates **in the
same commit**; frozen `reference/` evidence does not. R3 — `deny_unknown_fields` gives
refusal free; the named arm is one probe before parse, not a second parser.

**Pin.** Legacy fixture → named error with remedy, non-zero exit; new fixture parses;
a same-commit grep finds zero `[workspace]`/`[[repository]]` outside `reference/`.

### R-MVP1-4 — instruction-projection contract (A3) — R2, one named R7

**Ruled: manifest declares, core resolves and pins at bind, adapter translates.**
*Declares:* `[[repo]] instructions = "local" | "suppress"`, default `suppress` —
byte-identical to today's hardcoded `--setting-sources user` (`claude.rs:874-881`), so
the default changes no behavior (L18/R1). *Pins:* `workflow.bound` widens from
`workspace: <name>` (`engine.rs:1044-1051`) to the resolved `Vec<RepositorySpec>` +
per-repo policy + **resolved instruction file identities** (path + content hash at
bind; absent recorded as absent), so a mid-flight edit cannot reach a running Work —
additive fields in an immutable event, additive-only after. *Composition and conflict:*
**one process, one policy — so no composition happens, and that is the ruling, not an
omission**: multi-repo execution runs at the shared surface root (`surface.rs:156-161`)
and one `--setting-sources` rides one `Command` per turn, so all bound repos must
agree and disagreement **fails closed at submit** naming each repo, its value, and the
remedy. *Translates:* the adapter maps the pinned policy to its native mechanism and
never redefines it; what `local` translates to is **unmeasured** (L1), so `local`
parses and pins but is **refused at submit** — "measurement pending (MVP-2)". R7 taken
once, for the content hash: no lower rung answers "the file the actor will read is the
one we recorded."

**Pin.** Mixed-policy multi-repo submit refuses naming both repos; single-repo
`suppress` yields byte-identical launch args to today; `workflow.bound` carries specs +
policy + identities; editing an AGENTS.md after bind does not move the pinned identity.

### R-MVP1-5 — multi-repo measured, then group expansion — R1 engine / R2 CLI

**(a) Measurement first — the contract's first deliverable, before any group reasoning
applies.** Bind and run a two-repo Work on this host via `sgt run --repo a --repo b`
(fake backend, plus one bounded real-Claude turn inside R-MVP1-7's envelope), recording
the `execution_cwd` the actor got, whether both worktrees exist on `sergeant/<work-id>`
and the actor reached both, what teardown retained, and what `work show` said —
commands and raw output into
`docs/gauntlet/notes/multi-repo-bind-measurement-<date>.md`. Fact-finding, not a
promise (L1 turned on our own engine).

**(b) Group expansion, pre-committed with its falsifier.** Group membership gets **no
new engine surface**: `[group.<name>].repos` is manifest data and expansion is the
caller's job — `--repo` is already repeatable (`cli.rs:75-77`), so MVP-3's `--group` is
CLI-side expansion over the same submit request. **Falsifier:** if (a) shows the bind
broken — an actor that cannot reach a bound repo, a teardown that loses one, a
`work show` that hides one — the fix lands in MVP-1 and this ruling is re-argued on
that evidence before MVP-3 builds any group verb.

**Pin.** The measurement note exists and is cited by the MVP-3 contract; a test proves
a two-repo submit binds two worktrees and a `work show` naming both.

### R-MVP1-6 — intent schema, minimal fields (Finding 2 / U-R6) — R2, dedup R1

**Ruled.** Optional fields: `objective`, `repos` (or `group`), `acceptance`,
`exclusions`, `workflow`. Free-text `intent` stays required and primary. Progressive:
any subset may be present; present fields are journaled inside the existing
`work.submitted` payload and shown by `work show`; a `workflow`/`repos` disagreeing
with the flags refuses at submit (one source of truth, fail closed).

**Additive-only evolution — already safe, not merely promised.** `Work` carries no
`deny_unknown_fields` (`work.rs:159`) and event payloads are `serde_json::Value`
round-tripped losslessly, so a later field deserializes as absent from old events and
an older binary re-serializing a newer event drops nothing. Added discipline: new
fields `Option`/`default`, none ever removed or retyped.

**Dedup identity and promotion provenance (U-R5): not reserved — R1.** Reserving
fields nobody writes is the overbuild Finding 2 warns of, and the evidence above makes
later addition free. The expensive part is choosing the identity, not adding the field;
seven upstream collision issues argue for deciding it *with data*. It becomes the
post-MVP backlog type's first task.

**Pin.** All five fields journal and display; a disagreeing `workflow` refuses naming
both sources; replay of a payload carrying an unknown key is byte-identical.

### R-MVP1-7 — turn envelope at every turn-producing verb + ceiling — R2

**Ruled, on measured placement.** `launch` spawns turn 1 and `send` spawns every later
turn (both `spawn_turn`, `claude.rs:1389-1454`); `resume` never starts a turn
(`backend/mod.rs:718-731`, documented and measured); `start` is prepare+launch. **The
envelope gates exactly `launch` and `send`, in the engine, before the effect**, and
counts **spawned turns, not verb calls**: `PendingSend::perform` may call
`backend.send` twice for one delivery (`engine.rs:258-266` — failed send → resume →
retry) and only the successful call spawns a turn; the count derives from the journal.
The (N+1)th turn is never spawned — the Work settles `blocked`, reason `turn envelope
exhausted (N turns)`, whose exit door R-MVP1-10 covers. The per-turn wall-clock ceiling
rides the completion driver already sweeping active runs every 200 ms (`api.rs:563`,
`drive_completions`/`due_observations`): a turn older than the ceiling is INTERRUPTed
and journaled, settling by the ordinary interrupt path. That is the soak's hang bound
(CUT 10), not a stall detector — that stays post-MVP.

**L16 arithmetic, stated.** A cap of N bounds spend at N × the *largest* single turn,
never N × the average: the largest measured single turn here is $3.21 (Run B2), so a
6-turn envelope is a ~$19 bound, not a $2.50 one. A ceiling of T bounds a stalled turn
at T + one driver interval (200 ms) + adapter interrupt latency, never at T. Dollars
are *reported* where the adapter reports usage, never promised. Default cap and ceiling
**values** come from measured N-series turn counts at build time, recorded in the
ledger — not invented here.

**Pin.** A scripted run exceeding the cap blocks at exactly N spawned turns with no
(N+1)th; the double-send path counts one turn (needs R-MVP1-8); a `hang()` turn is
interrupted within ceiling + one interval; each test dies when its guard is reverted
(L7; guard map to the independent prober).

### R-MVP1-8 — minimum fake deferred-finish fidelity (A5) — precondition, R2

**Measured, sharper than A5 states it: the fake has no in-flight turn at all.**
`launch` installs a step whose terminal signal is readable by the very next OBSERVE
(`fake.rs:669-698`); `send` swaps in the next scripted step synchronously before
returning (`fake.rs:705-717`). Only `hang()` keeps a turn running, and it also ignores
STOP. No test has ever stood between "a turn was spawned" and "the turn finished" for
a turn that finishes — the exact interval R-MVP1-7 legislates over, and why #46's
45-minute stall was structurally invisible.

**Ruled.** A `FakeStep` may declare a **settle delay**: report `Running` for the first
*k* OBSERVEs after the launch or send that spawned it, then its scripted signal.
`k = 0` is today's behavior and the default, so no existing test changes. No wall-clock
variant (R1) — `hang()` already models the never-finishing turn the ceiling test needs.
Full R-H0-7 stays MVP-2. Built first.

**Pin.** A `k=2` step reports Running twice then completes *via the completion driver*
rather than launch-settle; all existing suites pass unchanged.

### R-MVP1-9 — Rule A: terminal-work eviction, standalone (#4) — R2

**Ruled** (owner-confirmed, v3 escalation 1). In-memory projection eviction of terminal
Work with journal re-derivation on access; nothing needs Docker. N4's adjudication
drops its bracketed Rule A paragraph; landing it here makes that mechanical.

**Pin** — the vehicle exists: `scripts/perf/s2-churn.sh` (200 works, waves of 10,
RSS/fd/CPU per wave, settle, re-sample). Post-settle RSS returns within a stated band
of the pre-run mark and the per-wave slope is flat, against #4's measured ~25 kB/work
monotonic climb; an evicted Work's API view is byte-identical to a non-evicted one;
restart indifferent (rebuild-on-start stays the only population path).

### R-MVP1-10 — blocked exit-door fault-injection invariant — R2

**Ruled: every state a fault can land a Work in has a journaled, testable way out,
proven by injecting the fault — never by synthesizing the state.** In scope:
`pending → blocked` (start failure), `active → blocked`, `waiting|needs_input →
blocked`, and R-MVP1-7's envelope-exhausted landing. Each gets one fault-injection test
that then drives `blocked → active` (already legal, `work.rs:106-112`; missing is proof
the door opens after a *real* fault). Upstream's 15-issue scar is the reason. L6 applies
in review: every path this contract adds that appends two causally-linked events
tolerates the second going missing, or writes one compound event. Permission-bit
fixtures work on Cerberus and the GH runner but not the root container — probe-gate
with a loud `SKIPPED-ENV`.

### R-MVP1-11 — submit-time capability preflight — R2/R6

**Correction to the plan, evidence first.** The bucketing calls this "E3's surviving
two-line form". It is not: **no workflow declares an ask stage, and no declaration
exists to read.** `StageKind` is `Actor` plus the `Execute` doc-stub
(`workflow.rs:90-98`); zero admitted workflows carry a `[stage."<id>"]` table at all;
`grilling` — the ask-dependent workflow — is two plain actor stages.

**Ruled.** Give the preflight the smallest declaration that makes it real: an optional
`requires_ask = true` on the existing, already-additive `[stage."<id>"]` table
(`workflow.rs:116-117`), set on `grilling`'s interview stage. At submit, once routing
resolves the backend, a workflow with any such stage against a backend whose
`Capabilities::ask` is false is refused, naming workflow, stage, backend, remedy. No
new stage kind (R7 avoided). This does not resurrect the WORKFLOW-IF-E3 category
(R-NS-6): the declaration says what the stage *needs*, not that the engine converses.

**Honest bound, doubled.** (i) It bites on a *statically declared* low capability;
runtime withdrawal is detected only after a turn completes (`claude.rs:193` needs a
finished envelope), so a withdrawn one is caught only once MVP-2 persists the
measurement. (ii) On Cerberus `ask` is **measured withdrawn today** (a5 red,
`post_turn_summary` absent) while the adapter still declares it — until MVP-2 this
host's verdict is the declared value, not the measured one. Named in the ledger and
the doctor row.

**Pin.** `grilling` + a fake with `ask: false` refuses naming stage and remedy; with
`ask: true` it submits; an undeclared workflow is unaffected; revert-probe kills it.

### R-MVP1-12 — estate discovery past inner `.git` — R2

**Measured.** `Workspace::discover` shells `git rev-parse --show-toplevel` and reads
`sergeant.toml` only there (`workspace.rs:205-231`), and `--show-toplevel` stops at the
innermost `.git` — so from inside `repos/payments-api` discovery finds the member repo,
never the estate.

**Ruled.** Discovery walks **upward from cwd, filesystem-first, crossing git
boundaries**, for the nearest `sergeant.toml` containing an `[estate]` table; the
git-toplevel path stays the zero-config single-repo fallback when none is found.
Bounded at `$HOME` or the filesystem root, whichever comes first, and never above an
explicit `--data-dir`/`SGT_DATA_DIR` scope. First match wins; a `sergeant.toml`
*without* `[estate]` on the way up is a member's own config, not an estate, and does
not stop the walk. Ancestors canonicalized once before the walk (the symlink hazard
`surface.rs:286-289` already defends).

**Pin.** Fixtures under #22's umbrella ("nested repos, submodules, paths with spaces,
repo-inside-a-worktree"): estate above a member repo; member repo with its own
`sergeant.toml`; nested worktree; path with a space; no estate anywhere → fallback
unchanged; outside any repo → today's `NotARepository` unchanged. #22's remaining edges
stay MVP-4 — this lands the discovery fixtures only and does not close the issue.

## Build order (part of the contract)

R-MVP1-8 (the instrument, before the semantics depending on it — A5) → R-MVP1-5(a)
(measurement, before any group reasoning) → R-MVP1-3 + R-MVP1-12 (the manifest's two
preconditions) → R-MVP1-1 → R-MVP1-4 → R-MVP1-6 → R-MVP1-7, R-MVP1-11, R-MVP1-10 →
R-MVP1-2 plus the output pointer. R-MVP1-9 is independent and may run isolated in
parallel.

## Acceptance (whole contract)

- Three gates green plus repeated-run verification (single-run green is not a gate,
  L7); every ruling's pin above, each carrying a guard-map entry for the one
  independent prober (pattern doc, 2026-08-11). `scripts/demo.sh` exits 0;
  `pgrep -f "debug/sgt [-]-data-dir"` empty after suites.
- **Self-hosting checkpoint (A8 — acceptance, not branding): at least one fix in this
  bucket executes as a sergeant Work against this repo.** R-MVP1-1 is what makes it
  legal; before the split a self-hosted surface is refused by design. Documented setup
  on Cerberus: data dir in-estate with `surfaces_root` outside the checkout (exercising
  the split, not avoiding it), and the **E2 workaround** — the daemon inherits its
  spawner's environment and this host's non-interactive PATH carries neither
  `~/.cargo/bin` (cargo/rustc) nor `~/.local/bin` (`claude` 2.1.227), so start it as
  `env PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH" sgt daemon …`
  (`docs/environments/cerberus.md`). E2 proper — the product owning its env contract —
  is MVP-2/3; this is a documented workaround, not its fix, and the ledger entry says
  so. Evidence recorded: work id, journal excerpt, retained branch, resulting commit.
- Ledger entry with both scorecards and per-decision rungs; deviation rows for the
  schema break and for any ruling departing from `NORTH-STAR.md` (R-MVP1-2 at minimum).

## Non-goals

Every MVP-2..5 row of the bucketing, as written there — including the four this
contract touches the edge of and must not do: the `--setting-sources` de-leak and what
`local` means, capability provenance, `kind = "execute"` (with the Docker executor and
the first real execute stage), and MVP-3's whole CLI surface — `--group`, the
estate-resolved data-dir *default*, `work transcript`, doctor estate checks. Also out:
the backlog type and dedup-identity semantics; daemon-resident stall detection beyond
the ceiling; dollar-cost enforcement; hot manifest reload; #22's non-discovery edges;
and the design capture's deliberately-absent list (per-repo backends/models, path
overrides, per-group instruction files, a third `instructions` value).

## Unknowns

1. What `local` translates to for the Claude adapter — unmeasured, which is why
   R-MVP1-4 refuses it at submit rather than guessing.
2. Default turn cap and ceiling **values** — mechanism contracted, numbers measured at
   build time. Likewise R-MVP1-9's "flat" band, set from the S2 baseline.
3. Whether multi-repo bind works end to end; R-MVP1-5(a) can falsify (b).
4. Whether the owner reads `NORTH-STAR.md`'s "core owns promote/finalize" as meaning
   (1). If so, R-MVP1-2 is void and the row moves to MVP-2.
5. Whether `grilling` survives MVP-5's R-NS-6 re-homing; if it becomes a skill,
   `requires_ask` may briefly have no declaring workflow.
