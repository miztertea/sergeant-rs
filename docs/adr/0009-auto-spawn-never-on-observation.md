# ADR 0009: Auto-spawn — observation never materializes the daemon

**Status:** Accepted, 2026-08-14.

## Context

`docs/gauntlet/contracts/WATCH.md`'s R-WATCH-3 already ruled this principle
for one verb: "`sgt watch` joins `sgt doctor` and `sgt daemon stop` in the
deliberate no-auto-spawn set," refusing outright with a named remedy rather
than starting a daemon just to have something to attach to, because
"observation must not materialize the thing observed — fail-closed at both
ends of the process's life" (owner-ruled 2026-08-13). That same contract
already flagged a follow-on, out of WATCH's own scope: "the owner's
consistency direction — observation surfaces generally shouldn't
materialize the daemon — touches `status`/`work`/`analytics`/`web`, whose
auto-spawn is pinned by m2/m6/m8 contract tests and load-bearing in
AGENTS.md's standard loop step 2," recorded as its own backlog issue rather
than resolved on the spot. This decision is that follow-on being resolved.

Today `sgt status` on a cold estate starts a daemon and then reports it
healthy — the act of observing changes what there is to observe, exactly
the failure mode R-WATCH-3 already named for `watch`.

## Decision

**No exceptions (D5).** `status`, `work show`/`list`/`transcript`,
`analytics`, and the TUI join `sgt doctor`, `sgt watch`, and
`sgt daemon stop` in the no-spawn set. Auto-spawn survives only on the
verbs that actually mutate durable state: `run`, `respond`, `retry`,
`extend`, `cancel`.

**The TUI's case, argued on its own terms.** The owner's reasoning here is
what settled the whole set, not just the TUI: `sgt watch`'s existing
reconnect behavior (issue #16) presupposes a daemon existed in the first
place — the live tail died, so retry with capped backoff. Bare `sgt` with
no daemon running has nothing to reconnect *to*. That is a different
starting state from the one reconnect logic is built for, and it gets a
different answer: fail closed, name the state, and point at `sgt doctor`
as the remedy — not materialize a daemon on the spot so the cockpit has
something to render.

**The correction on record.** The orchestrating session proposed carving
the TUI out of this rule as an exception — a human-facing surface that
should "just work" regardless of daemon state, on the theory that a
stranger's first `sgt` should never greet them with a refusal. The owner
rejected that carve-out, and the session agreed it was wrong on its own
merits, not merely overruled: materializing a daemon so the cockpit has
something to show is exactly the lie R-WATCH-3's principle exists to
prevent, and granting the TUI an exception for being human-facing would
have reintroduced the same failure this decision otherwise closes
everywhere else. No exceptions survived that argument, including the one
proposed for the surface that felt most like it deserved one.

## Alternatives considered

**Carve the TUI out as a "just works" exception**, proposed by the
orchestrating session during the interview and described above, is the
recorded alternative. It was rejected — not merely by the owner, but
conceded by the session that proposed it once the reconnect-vs-no-daemon
distinction was argued through: reconnect logic answers "the daemon I had
is gone," not "there was never a daemon," and the TUI's own auto-spawn
today conflates the two.

## Consequences

Implemented. `AGENTS.md`'s standard-loop step 2 now states that neither
`sgt status` nor `sgt work list` auto-spawns a daemon and treats a
no-daemon refusal on a fresh boot as an empty fleet (`AGENTS.md:76-83`).

Pinned contract tests changed accordingly: `tests/m2_daemon_api.rs`'s
`t7_cli_end_to_end_auto_spawn_and_second_daemon_fails_closed` stays
scoped to the still-auto-spawning mutating verbs; `tests/m6_surfaces.rs`
now pins `t1_observation_verbs_refuse_without_a_daemon_and_name_the_remedy`
instead of the old "bare `sgt` must auto-spawn" assertion; and
`tests/m8_estate_cli.rs` no longer asserts a daemon exists after a bare
`status`.

`sgt doctor`'s own message changed alongside the code (`src/cli.rs`'s
`doctor::daemon_check`): the no-descriptor case now reads "no daemon
running; a mutating verb (`run`, `respond`, `retry`, `extend`, `cancel`)
starts one on demand — `status`, `work`, `analytics`, `watch`, and `tui`
refuse instead," and the stale-descriptor warning was corrected to match.

## Open questions

`sgt doctor`'s replacement wording for the no-daemon case is not specified
in the interview beyond the fact that the current message becomes false;
what it should say instead — naming which verbs still auto-spawn and which
don't — is left to implementation.

The exact rewrite to `AGENTS.md`'s standard-loop step 2 is not specified
here; this ADR records that the step's current auto-spawn assumption no
longer holds; how the step should read instead is separate work.
