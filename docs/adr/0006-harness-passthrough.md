# ADR 0006: The harness passthrough

**Status:** Accepted, 2026-08-14. Implemented, 2026-08-15 (`src/harness.rs`,
`sgt claude`/`codex`/`opencode`/`goose` in `src/cli.rs`, the `environment`
row in `sgt doctor`).

## Context

Two open issues frame this decision. Issue #60 (an actor's environment
contract) is live because a daemon-launched actor today inherits whatever
environment the daemon happened to be started with — there is no contract
fixing what that environment must contain. On 2026-08-14, in the same
sprint this ADR's interview was part of, the orchestrating session made
that environment correct *by hand*: spawning the daemon itself and
verifying `/proc/<pid>/environ` to confirm the actor beneath it inherited
what it needed. That is operator discipline standing in for a contract the
product does not state. Issue #80 (`src/cli.rs:399`, "an owner ruling
tracked separately in #80 and is not decided here") is the companion
problem on the storage side: estate binding is rediscovered per command by
walking up from `cwd` rather than being fixed once at launch.

## Decision

**`sgt <harness> -- <args>` (D2).** `sgt claude -- <args>`, `sgt codex`,
`sgt opencode`, `sgt goose`. `sgt` does not own the harness process: it
composes the environment, binds the estate, and then **exec**s — replacing
its own process image with the harness's, not forking and supervising it.
The owner's own framing for this decision: it is "more of an
always-should-have-been-like-that than a feature." Launch through
`sgt claude` and the harness process, the daemon it in turn spawns, and
every actor beneath that daemon all inherit one deliberately-composed
environment, rather than whatever the daemon happened to be started with.
This also gives the other harnesses — `codex`, `opencode`, `goose` — a home
without `sgt` taking on ownership of them the way it owns the `claude`
backend today.

**Estate binding becomes explicit at launch (part of D2).** The same
passthrough that fixes the environment also fixes #80's binding half:
which estate a session operates against is decided once, explicitly, at
`sgt claude` launch time, rather than rediscovered implicitly per command
by walking up from the current working directory.

**Exec, not fork-and-supervise — load-bearing (part of D2).**
`NORTH-STAR.md`'s "Never" list is explicit: "reconstructed tmux-era
supervision" is never something this product builds. A passthrough that
grew a process table, a pid file, or a restart policy for the harness it
launches would be exactly that, wearing a different name. Exec'ing into the
harness means there is no lifecycle for `sgt` to own once the harness
starts — the boundary is the whole point of the decision, not an
implementation detail of it. This is a human-facing surface, joining
`sgt init` and `sgt doctor` as commands a person runs directly rather than
ones a workflow drives.

**Explicitly out of scope: #94's execution-model half.** This decision
does not address whether an actor gets a background-completion callback
mid-turn. That is a property of how the backend invokes the harness on
each turn — a per-turn contract — not a property of how the human's
session was launched. Solving the environment and binding problem at
launch time says nothing about what an actor already mid-turn can expect
from the runtime around it; see ADR 0007 for that question.

## Alternatives considered

**`sgt` owning and supervising the harness process** — spawning it,
tracking its pid, restarting it on failure, the shape every other command
in this repo's daemon-adjacent surface uses — was the implicit alternative
this decision rejects. It is ruled out directly by `NORTH-STAR.md`'s
"Never" list entry on reconstructed tmux-era supervision; the interview
did not treat this as a close call.

**Leaving the environment problem as operator discipline** (continue
verifying `/proc/<pid>/environ` by hand per session, as the orchestrating
session did on 2026-08-14) was the status quo this decision replaces. It
was not treated as a real alternative so much as the problem statement:
operator discipline is not a contract, and it does not survive a session
that forgets to apply it.

## Consequences

This solves #60's environment half structurally: an actor's environment is
composed once, deliberately, by the passthrough, instead of depending on
whichever operator remembered to check it. It solves #80's binding half the
same way: estate binding is explicit at launch instead of rediscovered per
command.

The residual hole is explicit and must not be papered over: this improves
the common path, it does not close the problem. Running `sgt run` from a
terminal that never went through `sgt claude` (or one of the other harness
verbs) still inherits whatever environment that terminal happened to have,
and the old #60 problem returns for that session. The complement this
decision names for that gap is `sgt doctor` checking its own environment
against the contract this ADR establishes and naming the remedy when it
does not hold — tracked as **issue #100**, not built by this ADR.

Because `sgt claude` and its siblings are human-facing surfaces rather than
something a workflow drives, they add a new category of command alongside
`sgt init` and `sgt doctor` — commands whose audience is a person sitting
at a terminal, not an actor executing a Work.

## Implementation, 2026-08-15

§8.1 left "the environment" undecided rather than unresolved-by-omission —
the Work that closed #100/#60 had to choose a list, not invent a mechanism
to defer the choice again. Recorded here as the choice actually shipped,
not as a retroactive ruling: it is reviewable and reversible the way any
other implementation decision is, unlike the D2/D3 owner rulings above it.

**What `sgt <harness>` composes (`src/harness.rs`):**

- **PATH enrichment**, prepending `~/.cargo/bin` and `~/.local/bin` when
  each exists on disk. Both are measured, not guessed
  (`docs/environments/cerberus.md`): `~/.cargo/bin` houses cargo/rustc and
  is the literal #60 failure; `~/.local/bin` is where the `claude` CLI
  itself and `no-mistakes` are installed on that host, so a harness a user
  cannot even find on PATH is a sharper version of the same problem than a
  missing build tool.
- **`SGT_DATA_DIR` bound explicitly**, set to whatever `cli::resolve_data_dir`
  already resolved for the invocation (flag, env, estate walk, or the
  pre-estate platform fallback, in that order) — reusing the existing ladder
  rather than re-deriving estate discovery, so the harness, the daemon it
  spawns, and every actor beneath it agree on one data dir.

**The argument for this list:** it is the smallest change that closes the
measured failure without sergeant taking on a host it does not own. It
states two concrete, named directories and stops, rather than sourcing
`.bashrc`/`.profile` (which would make sergeant responsible for parsing and
running arbitrary shell config) or attempting to enumerate every possible
toolchain manager (`nvm`, `sdkman`, Homebrew's `/opt/homebrew/bin`, ...).

**The strongest argument against it:** it is measured on exactly one host
(Cerberus). A colleague whose toolchain lives somewhere this list does not
name (a non-standard `CARGO_HOME`, an `asdf`-managed toolchain, Homebrew on
macOS) gets no PATH enrichment at all and is back to #60's failure shape —
under-specified for their host even though it is not under-specified for
the one host this was measured against. Widening the list is cheap later
(`toolchain_path_dirs` in `src/harness.rs` is the one place both `sgt
<harness>` and `sgt doctor`'s environment check read it from, so a future
Work extends one function, not two call sites that could drift); narrowing
a list that guessed wrong would be more disruptive. This asymmetry is why
the narrower list was chosen over a broader guess.

`sgt doctor`'s environment check (issue #100) reads the exact same list via
`harness::dirs_missing_from_path`, so the check and the thing it is
checking cannot silently disagree about what "the environment" means.

## Open questions

The design of `sgt doctor`'s environment check — resolved above: it checks
the same `toolchain_path_dirs` list the passthrough composes, `Warn`s (not
`Fail`s) when one exists on disk but is missing from `PATH`, and names `sgt
claude` (or the sibling verbs) as the remedy.

The exact set of environment variables and estate-binding facts the
passthrough must compose — resolved above, as an implementation judgment
call rather than an owner ruling. Whether the chosen list is the *right*
one for hosts beyond Cerberus remains open in the sense any measured-not-
assumed fact is open: it closes further only as it is measured elsewhere
(ADR 0001's posture).
