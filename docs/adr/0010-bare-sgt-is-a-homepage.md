# ADR 0010: Bare `sgt` is a homepage; the TUI is a verb

**Status:** Accepted, 2026-08-14.

## Context

Today, bare `sgt` with no subcommand opens the TUI directly: "§30: bare
`sgt` is the TUI. It is a client like any other, so it gets the same
auto-spawn path the CLI commands get" (`src/cli.rs:424`, doc comment at
`src/cli.rs:52`: "Subcommand to run. Omitted, `sgt` opens the TUI (§30)").
`tests/m6_surfaces.rs:414` pins exactly this: running bare `sgt` "must
auto-spawn a daemon like every other client." ADR 0009 (this same day's
interview) rules that observation surfaces must never auto-spawn a daemon
— which raised the question of what bare `sgt` should do once the TUI can
no longer be the thing a daemon-free command falls into by default. A
stranger who clones this repo, puts `sgt` on `PATH`, and types `sgt` with
nothing else is dropped straight into the TUI today, silently starting a
daemon in the background to have something to populate it with.

## Decision

**`sgt tui` becomes an explicit verb (D6).** The owner's framing: the TUI
should not be its own special case reached by typing nothing — it should
just be `sgt tui`, a verb like any other. Bare `sgt`, with no subcommand at
all, instead becomes a daemon-free homepage: an ASCII-art logo plus a
condensed quickstart.

**This dissolves the ADR 0009 carve-out debate rather than answering it.**
Once bare `sgt` touches no daemon at all, the question ADR 0009 settled
against a TUI carve-out — should a human-facing surface get to auto-spawn
because refusing would be a bad first impression — simply does not arise
for bare `sgt` anymore. There is no daemon question to answer for a
command that never asks the daemon anything.

**First contact, fixed directly.** Today a stranger who installs `sgt` and
types it gets dropped into a TUI and a silently-started daemon in the same
motion. `NORTH-STAR.md`'s own acceptance criterion is that a stranger
reaches a finished change in under five minutes of setup — a logo plus a
condensed quickstart serves that goal directly, by orienting a stranger
toward `sgt init` instead of dropping them into a cockpit with nothing on
screen yet.

## Alternatives considered

**Keep bare `sgt` as the TUI**, the status quo per §30 of the founding
proposal (`reference/proposal-depot-rust-execution-surface.md`, cited at
`src/cli.rs:424`), was the implicit alternative and is rejected: it
collapses "homepage for a stranger's first contact" and "cockpit for
someone with a daemon already running" into one command, and it is exactly
what forces the daemon question the moment someone types `sgt` alone.

## Consequences

`sgt tui` is a new explicit verb; bare `sgt`'s behavior changes from
launching the TUI to rendering the homepage. Neither is implemented by
this ADR.

This is a deviation from the founding proposal, and `docs/DEVELOPMENT.md`
is explicit about what that requires: "Departures from either proposal
live in GAUNTLET.md's deviation register and are settled there —
re-litigate one only by arguing its ruling is wrong, not by noticing the
deviation exists." §30 specifying bare `sgt` as the TUI is exactly such a
departure once this decision lands, and it needs its own entry in
`GAUNTLET.md`'s deviation register — a quiet code change without that
entry would be the wrong way to land this.

## Open questions

**Whether the homepage should be estate-aware was proposed, not ruled
on.** The orchestrating session proposed making the homepage read
`sergeant.toml` to report the estate's name and repo count when run from
inside one — reading the manifest is not observing the daemon, so ADR
0009's rule would not block it — versus a fixed "run `sgt init`" message
when outside an estate. The owner did not rule on this refinement; it is
recorded here as a live option, not a decision, and should not be
implemented as though it were settled.

The `GAUNTLET.md` deviation register entry this decision requires has not
been written; this ADR names that it is owed, not its content.
