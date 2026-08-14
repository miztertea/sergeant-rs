# Sergeant-rs Foundation Rationalization
## Captain and Sgt: correcting six boundaries the build outgrew

Status: proposed, 2026-08-14. Written by the orchestrating session (Captain)
after an owner grilling interview the same day. Submitted to a gauntlet for
validation before enactment; acceptance is the owner's, not the panel's.

Companion records: `docs/adr/0005`–`0011` (the decisions and their
rationale), `docs/gauntlet/runs/cross-platform-2026-08-14/` (the sprint that
surfaced most of this, its plan, and its lessons).

---

# 1. Executive Summary

This round corrects six boundaries that were drawn correctly for a system
that no longer exists, plus one surface that was never ruled on at all.

Sergeant was built from zero. Claude Code workflows built `sgt`; once `sgt`
could run work — days before this round — the owner began using `sgt` to
build `sgt`. Documents written during the first era still describe it, and
several are now load-bearing in ways their authors could not have intended.
`docs/DEVELOPMENT.md`'s shipping-gate section describes a procedure that
`validate-and-ship` has since encoded; §30 pins bare `sgt` to a TUI that
predates having a homepage; the actor's runtime was never described to the
actor because, originally, there was no actor.

Nothing here is a new feature. Every item is a correction, and every one
traces to something that measurably cost time or work on 2026-08-13/14 —
cited in §3 with evidence rather than asserted.

The seven changes:

| § | Change | ADR |
|---|---|---|
| 5.1 | Gating becomes a dispatched Work | 0005 |
| 5.2 | Harness passthrough — `sgt claude`, `sgt codex`, … | 0006 |
| 5.3 | The product states the actor's runtime contract | 0007 |
| 5.4 | The manifest gains authority over its own state location | 0008 |
| 5.5 | Observation never materializes the daemon | 0009 |
| 5.6 | Bare `sgt` becomes a homepage; the TUI becomes a verb | 0010 |
| 5.7 | The dashboard is deleted | 0011 |

# 2. Basis and Method

## 2.1 What produced these decisions

A live grilling interview between the owner and the orchestrating session on
2026-08-14, held in-session per R-NS-6 (execution ≠ dialogue; a dispatched
Work has no mid-turn hold for a human answer). Seven decisions were made by
the owner and confirmed at an explicit gate. Three of them **corrected the
orchestrating session's own proposal** — recorded as such in the ADRs and
summarized in §5, because a record that launders the orchestrator into
having been right is worse than no record.

## 2.2 What surfaced the problems

The 2026-08-14 cross-platform sprint, run through the engine. It closed six
issues (#16, #70, #83, #86, #87, #91) and surfaced seven findings nobody was
hunting: #90, #91, #94, #95, #96, #100, plus corrections filed against #90
and #91 where the original filing was wrong. Three of those degraded the
engine every subsequent wave ran through.

This matters methodologically: the defects in §3 were not found by audit.
They were found by using the system hard enough that they cost something.

## 2.3 What this proposal is not

It is not a redesign, not a feature round, and not a re-litigation of the
seven decisions — those are owner-ruled. It is the plan for enacting them,
submitted for validation that the plan is sound, its assumptions are true,
its principles are intact, and its sections are executable.

# 3. What the round corrects, with evidence

## 3.1 The gate is Captain-serial and hand-driven

`scripts/gate.sh` wraps no-mistakes, which takes ownership of the branch for
the duration of a run. Because ownership is repo-wide, `docs/DEVELOPMENT.md`
rules that an actor inside a worktree never invokes the gate — only the
top-level orchestrating session does.

The consequence, measured on 2026-08-14: every gate blocked Captain for
roughly eight minutes, three ran in sequence, one was lost entirely to a
harness kill and had to be redone. Worse, Captain spent the day driving
pipeline mechanics — `axi respond`, `axi sync --recover`, custody
reconciliation — while the judgment those calls existed to serve took
minutes.

The compounding defect: `validate-and-ship` already encodes that procedure
as seven stages, including `40-drive-gates` (which classifies findings as
`auto-fix` / `no-op` / `ask-user`) and `50-reconcile-custody` (a complete
decision table over `branch_sync` states, including a `--keep-local`
remediation for the dirty-worktree refusal). Captain hand-rolled all of it,
badly, because `docs/DEVELOPMENT.md` restates the procedure in prose and
never names the workflow. **An owning document that summarizes a workflow
without citing it guarantees readers stop there.**

## 3.2 The actor's runtime is undeclared

Two distinct facts an actor cannot discover and was never told:

**Its environment.** A daemon-launched actor inherits whatever environment
the daemon was started with (#60). On this host the toolchain is not on a
non-interactive shell's PATH, so an actor gets no `cargo` unless someone
started the daemon from a shell that had it. On 2026-08-14 the orchestrating
session made that true by hand — spawning the daemon itself and verifying
`/proc/<pid>/environ` — which is operator discipline standing in for a
contract. #60's own history records an actor misdiagnosing the resulting
failure as a permissions fault.

**Its execution model.** An actor in a headless turn gets one turn and no
callbacks. It cannot background a command and be woken when it finishes.
Actors guessed otherwise **twice on 2026-08-14 and lost their work both
times** (#94). The second transcript shows correct reasoning from wrong
premises: the actor considered a wakeup mechanism, correctly ruled it
inapplicable, then concluded it would "wait for the background task
notification instead."

The second occurrence was **provoked by a dispatch brief written by
Captain** demanding empirical proof, which required long background runs. A
well-formed acceptance criterion steered the actor into losing its work.

## 3.3 The manifest is not authority for its own state

`resolve_data_dir` (`src/cli.rs:400-421`) walks up from cwd to *find*
`sergeant.toml`, then ignores its contents and hardcodes
`estate_root.join(DEFAULT_ESTATE_DATA_DIR)`. Meanwhile `[estate]
surfaces_dir` exists and is honored (`src/domain/workspace.rs:199-204`,
overridable by `SGT_SURFACES_DIR`).

So the manifest — which the MVP plan calls the keystone — is authority for
repos, groups, profiles and `surfaces_dir`, but not for where its own
durable state lives. Two sibling path decisions, two ownership models.

Adjacent and unresolved: the precedence itself. Estate discovery outranks
`XDG_DATA_HOME`, so an explicitly-set `XDG_DATA_HOME` is silently inert
inside any estate. This was undocumented until 2026-08-14 and cost a
base-commit bisect to attribute (#73, since closed; the contract question
re-homed to #80).

## 3.4 Observation materializes what it observes

`sgt status` on a cold estate starts a daemon and then reports it healthy.
So do `work show|list|transcript`, `analytics`, `web`, and bare `sgt`
(`ensure_daemon`, ten call sites in `src/cli.rs`). `sgt doctor`,
`sgt watch` and `sgt daemon stop` are already exempt, the last by explicit
ruling (R-WATCH-3: "observation must not materialize the thing observed").

The rule exists and is applied to one verb. Everywhere else, a surface
changes what it reports.

## 3.5 The CLI's entry points are inverted

Bare `sgt` is the TUI (§30, cited at `src/cli.rs:426`). A stranger who
installs sergeant and types `sgt` is dropped into a cockpit — and silently
starts a daemon to populate it. There is no homepage, and the TUI has no
verb of its own.

Separately, there is no way to launch a harness *bound to an estate*. The
North Star's loop is "clone → `sgt` on PATH → `sgt init` → open your harness
→ say let's work on the api bug." The fourth step is unassisted: the harness
is opened by hand, in whatever environment the terminal happens to carry,
and binds to an estate only implicitly by cwd.

## 3.6 The dashboard is an unowned surface

`src/web.rs` is 779 lines plus `web/`, reached by the `sgt web` verb. It
carries two open issues (#21, no test ever executes `dashboard.js`; #15,
token-in-URL must become a cookie handoff before any non-loopback binding)
that have been deferred repeatedly because nobody could say whether it has a
future.

`north-star-arbitration-2026-08-11.md:196` argued for deleting it. That file
is the **argument record, not the ruling**:
`north-star-dispositions-2026-08-11.md` never mentions web, dashboard or
freeze; `NORTH-STAR.md` still lists the dashboard as a surface and its Wave 3
plans `#11/#16`; and #11 was fixed on 2026-08-13, which the proposed freeze
would have foreclosed. The disposition has never actually been ruled.

# 4. Invariants this round must preserve

Every change below is checked against these. A change that requires
weakening one is a finding, not a trade.

## 4.1 The journal is the only durable truth
Nothing here adds a second source of truth or a state that outlives a
journal rebuild.

## 4.2 One owner
The daemon exclusively owns the data dir. §5.1 does not weaken this — it
replaces one ownership mechanism (no-mistakes' repo-wide branch lock) with
one sergeant already enforces (a Work owns its surface).

## 4.3 Ambiguity fails closed
§5.5's sweep makes more surfaces fail closed, not fewer. §5.3's safety net
converts a silent success into an honest non-terminal state.

## 4.4 A surface adds usability, never functionality
§5.2's passthrough **execs** and owns no lifecycle. §5.6's homepage reads a
manifest and contacts nothing.

## 4.5 Measured, not assumed
No change here claims a platform, capability or behavior that has not been
measured on a host we have. Where something is unmeasured it is marked, and
the issue stays open.

## 4.6 Procedure is data
§5.1 moves gating into a workflow package rather than a script, which
strengthens this rather than bending it.

## 4.7 The Ponytail minimality ladder
Each change should sit on its lowest viable rung. §5.7 is a deletion — the
lowest rung available. §5.2 is an `exec`, not a supervisor. §5.1 reuses
`validate-and-ship` rather than authoring a new procedure.

# 5. The changes

## 5.1 Gating becomes a dispatched Work (ADR 0005)

The gate becomes a Work with its own surface. Captain reads findings and
decides; Sgt executes the pipeline. `docs/DEVELOPMENT.md`'s "never gate from
a worktree" rule **dissolves** rather than being amended — it was a
workaround for no-mistakes' ownership model, and if the gate *is* the Work
that owns the surface, there is exactly one owner and nothing to break.

`validate-and-ship`'s existing authority split is preserved unchanged:
`auto-fix` findings are the actor's to authorize, `ask-user` findings are
relayed verbatim and never resolved autonomously. In a dispatched gate,
`ask-user` surfaces as `needs_input` and Captain answers via `sgt respond` —
the engine's existing hold mechanism, not new machinery.

**no-mistakes stays inside the gate Work initially.** Its review is the
asset: on 2026-08-14 it found four real defects Captain's own review had
passed, including a regression its own fix had introduced. A rebuilt ICM
review is a new brief with no track record. Stages get rebuilt only where we
can show we have matched them.

**Consequence accepted:** gating stops being Captain-serial and becomes
durable and resumable. **Consequence to watch:** the gate Work reviewing a
branch is a Work grading another Work's output; the independence that makes
the review valuable must not quietly become self-review.

## 5.2 Harness passthrough (ADR 0006)

`sgt claude -- <args>`, `sgt codex`, `sgt opencode`, `sgt goose`. sgt
composes the environment, binds the estate, and **execs** — replacing itself
with the harness.

This makes §3.2's environment half a guarantee instead of a ritual: the
harness, the daemon it spawns, and every actor beneath inherit one
deliberately-composed environment. It makes estate binding explicit at
launch instead of rediscovered per command. And it gives the other harnesses
a home without sgt owning them.

**The boundary is load-bearing: exec, not fork-and-supervise.**
`NORTH-STAR.md`'s "Never" list includes reconstructed tmux-era supervision.
A passthrough that grows a process table, a pid file or a restart policy is
exactly that; exec'ing means there is no lifecycle to own.

**Residual hole, stated:** this improves the common path, it does not close
it. `sgt run` from a terminal that never went through `sgt claude` returns
to §3.2's problem. The complement is `sgt doctor` checking its own
environment against the contract and naming the remedy.

## 5.3 The actor runtime contract (ADR 0007)

Two parts.

**(a)** Whatever composes an actor's context states what it can rely on —
its environment guarantee (§5.2) and its execution model: one turn, no
callbacks, long commands run in the foreground. The actor in §3.2 reasoned
correctly from premises nobody gave it; give it the premises.

**(b)** Independently, a closing stage that declares a commit as its durable
outcome must not land in plain `completed` when the branch never advanced
and the worktree is dirty. This is the safety net for when an actor guesses
wrong anyway, and it is separable from (a) — worth having even if (a) were
perfect.

## 5.4 Manifest authority over storage paths (ADR 0008)

Three parts, one ruling.

**Uphold estate-first precedence.** `XDG_DATA_HOME` is one global path;
estates are per-directory. If XDG won, every estate on a machine would
collapse into one journal, one blob store and one exclusive daemon lock,
with one estate's Works referencing another's repos. That is a collision,
not a preference. §5.2's explicit launch binding removes most of the
surprise that made it feel wrong.

**Add `[estate] data_dir`**, for symmetry with `surfaces_dir`. The manifest
is authority for both or neither.

**Re-rule #64 rather than implement it.** The promised flip moved surfaces
outside every checkout via `XDG_STATE_HOME`; with the estate as a
deliberate, explicitly-bound directory, splitting state across
`~/.local/state` and the estate is *less* coherent —`NORTH-STAR.md`'s own
phrasing is "machine-local truth is in-estate and gitignored." **The cost,
stated rather than softened:** surfaces under `.sergeant/data/surfaces/` do
sit inside the sergeant-rs checkout when sergeant operates on itself,
invisible only because that path is gitignored and the engine's refusal
checks the target repo rather than the outer one. Re-ruling accepts that
permanently.

## 5.5 Observation never materializes the daemon (ADR 0009)

`status`, `work show|list|transcript`, `analytics` and the TUI join
`doctor`, `watch` and `daemon stop` in the no-spawn set. Auto-spawn survives
only on mutating verbs: `run`, `respond`, `retry`, `extend`, `cancel`.

**No exceptions**, including the TUI. The owner's reasoning decided it:
reconnect (#16) presupposes a daemon existed and the tail died — retry with
backoff. Bare `sgt` with no daemon has nothing to reconnect *to*. Different
state, different answer: fail closed, name the remedy. Captain proposed
carving out the TUI as a human surface that should "just work"; that was
rejected, and rightly — materializing a daemon so the cockpit has something
to show is precisely the lie the rule exists to prevent.

**Blast radius, known:** `AGENTS.md`'s standard-loop step 2 relies on
auto-spawn on a fresh boot; pinned contract tests change (m2 t7,
`tests/m6_surfaces.rs:414`, `tests/m8_estate_cli.rs:1080`); and `sgt
doctor`'s own message — "no daemon running; the next client command starts
one" — becomes **false** the moment this lands. The diagnostic surface would
begin lying about the rule this establishes, so it changes in the same
change.

## 5.6 Bare `sgt` is a homepage; the TUI is a verb (ADR 0010)

`sgt tui` becomes explicit. Bare `sgt` becomes a daemon-free homepage: logo
and condensed quickstart.

This dissolves §5.5's carve-out question rather than answering it — bare
`sgt` touches no daemon, so auto-spawn does not arise. It also fixes a
strange first contact and serves `NORTH-STAR.md`'s acceptance directly ("a
stranger reaches that last step in under five minutes of setup").

**Deviation:** §30 specifies bare `sgt` *is* the TUI. Per
`docs/DEVELOPMENT.md`, departures from the proposal live in `GAUNTLET.md`'s
deviation register and are settled there — this needs a register entry, not
a quiet edit.

**Open, not decided:** whether the homepage is estate-aware (reading
`sergeant.toml` is not observing the daemon) or a static banner.

## 5.7 The dashboard is deleted (ADR 0011)

Remove `src/web.rs`, `web/`, and the `sgt web` verb. #21 and #15 close as
won't-do, and `web` leaves §5.5's sweep list.

The human surfaces are `sgt init`, `sgt doctor`, `sgt tui` and the new
homepage. The dashboard is not among them, and three hand-maintained
renderings of one API for one human is R1 failure three ways. Deletion is a
lower rung than a disabled stub that still owns two issues we are not
honoring.

**This is the first actual ruling on the dashboard**, not a restatement —
see §3.6.

# 6. Sequencing

Dependency-ordered, not priority-ordered.

1. **§5.1 (gate as a Work)** first, gated the old way one final time, so
   everything after lands through the corrected process.
2. **§5.3 (runtime contract)** next: it is the cheapest and it stops active
   bleeding — actors have lost work twice.
3. **§5.2 (passthrough)** then, since §5.3(a)'s environment guarantee is
   what the passthrough composes.
4. **§5.4 (manifest authority)** — independent, slots anywhere after 1.
5. **§5.5, §5.6, §5.7 together.** All three are CLI surface with shared test
   blast radius; §5.7 removes a verb §5.5 would otherwise sweep, and §5.6
   dissolves §5.5's hardest case. Splitting them means three passes over the
   same pinned tests.

Carried from the current sprint and unaffected: #85, #12, #10, #8→#4, plus
#90 and #94's engine-side fixes.

# 7. Explicit non-goals

- **Not a TUI redesign.** That is deliberately not started; §5.6 changes how
  the TUI is *invoked*, nothing about what it renders.
- **Not new harness adapters.** §5.2 gives codex/opencode/goose an entry
  point; measuring and adapting them is separate work, still gated on an
  environment where each can be measured (#25's precedent).
- **Not closing the macOS portability issues.** #18, #81, #82 and #95 close
  when measured on a real macOS host, not here.
- **Not re-litigating the seven decisions.** Owner-ruled; the panel's
  question is whether this proposal expresses them faithfully and
  executably.
- **Not a migration story.** There are no users and the integration branch
  is unmerged. This is the moment such changes are cheapest, and that is
  part of why they are being made now.

# 8. Unknowns

Named rather than resolved, per the contract convention.

## 8.1 What exactly the environment contract guarantees
§5.2 composes "the environment," and §5.3 states it. The precise list —
toolchain, estate binding, what else — is not settled. Under-specifying it
recreates #60 with extra steps; over-specifying it makes sergeant
responsible for a host it does not own.

## 8.2 Whether a gate Work reviewing a Work stays independent
§5.1 trades no-mistakes' repo-wide lock for sergeant's surface isolation.
Whether the review stays as adversarial when it is another Work rather than
an external pipeline is unmeasured, and it is the assumption most likely to
be wrong.

## 8.3 The homepage's estate-awareness
§5.6, unruled.

## 8.4 Whether `validate-and-ship` needs re-homing
`grilling` and `grill-with-docs` were retired from workflows to `skills/`
because they need live dialogue. If gating becomes a dispatched Work,
`validate-and-ship` moves the other way — but its directly-invoked
`/no-mistakes` entry variant, written when there was no engine to dispatch
to, may now be dead weight. Not decided here.

## 8.5 What replaces the dashboard's one genuine use
§5.7 deletes a surface that could be opened on a phone or a second machine.
Nothing in the remaining set does that. Whether that use was real is
unmeasured — there are no users to ask.
