# ADR 0002: Platform boundary shape

**Status:** Accepted, 2026-08-14.

## Context

With ADR 0001 fixing the measured targets at Linux, macOS, and
Windows-via-WSL2, this repo needs a shape for the code that has to know
which of those it's on. The obvious existing precedent in this codebase is
the `Backend` trait (`src/backend/mod.rs:669`, the §15 boundary per
`docs/DEVELOPMENT.md`'s layout note) — `claude`, `fake`, and `docker`
implementations selected at runtime by `--backend` (`src/cli.rs:77`), a
genuine per-Work choice made at `sgt run`/submit time. The question this
ADR settles is whether platform should get the same treatment.

## Decision

**Standard API over `#[cfg]`-selected modules, not a trait (D2).** Platform
is a compile-time fact, not a runtime decision — nobody runs
`sgt --platform windows` on a Linux host the way they run
`sgt run --backend fake`. The `Backend` trait earns its runtime
polymorphism because backend choice genuinely varies per Work at submit
time; platform choice does not vary at all once the binary is built. A
`Platform` trait would buy dispatch nobody exercises and would permit
instantiating platform/backend combinations that cannot exist on the host
running them. This mirrors two existing precedents in this codebase rather
than inventing a new pattern: the M1 ruling that kept `Reducer` a plain fn
pointer instead of a trait, resolved at Ponytail rung R6 ("trait when a
second projection demonstrates need" — `GAUNTLET.md`'s M1 entry, "Event
Core (2026-08-08)", design-decisions paragraph), and `TtyWatch::watching`
in `src/tui.rs:1043`, which takes an injected probe function rather than
abstracting the terminal-hangup check behind a trait.

**Injected probes for testability (D3).** Each platform-dependent fact is a
function that takes its probe as a parameter, so the decision logic built
on top of the fact is testable on any host and only the raw syscall itself
is `cfg`-gated. This recovers the one genuine advantage the rejected
`Platform` trait would have had — exercising another platform's
fail-closed path from a Linux dev box — without paying for runtime
dispatch, and it is not a new idiom: `TtyWatch::watching(probe: fn() -> bool)`
(`src/tui.rs:1041-1043`) already does exactly this for pty hangup
detection, and its own test suite (`src/tui.rs:1778-1790`,
`TtyWatch::from_probe`) exercises both hung-up and not-hung-up branches
from whatever host the test happens to run on.

**What is behind the boundary (D4).** Platform facts are in scope for this
boundary. Docker's and the `claude` CLI's behavior are explicitly out of
scope — those remain `Backend` capabilities with runtime withdrawal, which
is existing machinery and not something this ADR changes. Docker Desktop
on macOS has different bind-mount and uid semantics than Docker on Linux;
the worktree-ownership fix already in this codebase
(`DockerBackend::create_container` passing `--user <uid>:<gid>` sourced
from the worktree's own host owner, `src/backend/docker.rs:140` and the
Cerberus measurement recorded in `docs/environments/cerberus.md`'s "Docker
container-written file/directory ownership" row) is itself a Unix concept
— it does not obviously generalize to a boundary about platform facts.
Whether `claude -p` behaves identically across hosts is a measure-first
question the adapter owns, per `LESSONS.md`'s L1 ("Measure the Claude CLI,
never trust its docs or its exit codes"), not something a platform module
should assume on the adapter's behalf. Note that the process-model concern
that would have made this boundary larger mostly dissolves under ADR
0001's D1: all three measured targets are Unix underneath, so there is far
less "platform fact" surface here than there would have been with native
Windows in scope.

## Alternatives considered

**A `Platform` trait with four implementations, including a "generic"
fallback** — the owner's original proposal going into the interview — was
rejected on the compile-time-vs-runtime argument above: it buys dispatch
nobody exercises (nothing selects a `Platform` impl at runtime the way
`--backend` selects a `Backend` impl) and it permits constructing
combinations, like a `WindowsPlatform` running on a Linux host, that are
category errors given how this boundary is actually used.

## Consequences

The negative consequence here is real and must survive review, not get
smoothed over: with `#[cfg]`-selected modules, the macOS module is **not
compiled at all** when building on Linux. Breaking it is invisible to a
Linux dev session — `cargo build`, `cargo clippy`, and `cargo test` on
Cerberus or in the cloud container will not notice a broken macOS-only
function — and only CI, once it runs a macOS lane, catches it. A `Platform`
trait would have kept every implementation type-checked on every host
regardless of which one was running. This is a real cost the interview
accepted deliberately in exchange for not paying for unused runtime
dispatch, and it is the specific reason ADR 0004's D9 wires a cheap cross
`cargo check` against the macOS target into the per-push CI lane — that
lane exists to compensate for exactly this gap, not as unrelated CI
hygiene.

## Open questions

None identified in the interview record for this decision beyond what ADR
0004 already carries forward about the compensating CI lane's own
implementation status.
