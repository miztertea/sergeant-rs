# ADR 0011: Delete the dashboard

**Status:** Accepted, 2026-08-14.

## Context

`docs/DEVELOPMENT.md` still lists `src/web.rs` — "the embedded dashboard"
— alongside the CLI and TUI as one of the three clients that reach state
only through the loopback API. `NORTH-STAR.md` still lists the dashboard
among its surfaces (`Surfaces (CLI, TUI, dashboard, harnesses)`) and its
Wave 3 text still actively plans `#11/#16` against it. `src/web.rs` was
779 lines and `sgt web` was still a live verb at the time of this
interview.

The dashboard's deletion was *argued for*, not yet *ruled on*, before this
ADR. `docs/gauntlet/notes/north-star-arbitration-2026-08-11.md:196`
proposes exactly this: "**Delete** `src/web.rs` (779) + `web/` (224) +
the `sgt web` verb — T-series §5.1 item 14 already proposes disabling the
route and leaving a stub with two live reactivation issues (#15, #21);
deletion is the lower rung than a disabled stub that still owns issues."
But that file is the arbitration record — one seat's argument in a
multi-seat adjudication process — not the ruling that came out of it.
`docs/gauntlet/notes/north-star-dispositions-2026-08-11.md`, the actual
disposition record from that same day, contains no mention of `web`,
`dashboard`, or `freeze` anywhere in its text. Issue #11 — one of the two
issues the arbitration's proposed stub would have kept open for
reactivation — was fixed on 2026-08-13, after the arbitration text was
written and before this interview, which the arbitration's proposed freeze
would have foreclosed. So going into this interview, the dashboard was
neither closed nor built: it was waiting on a disposition that had never
actually been made.

## Decision

**Delete `src/web.rs`, `web/`, and the `sgt web` verb (D7). #21 and #15
close as won't-do.** `NORTH-STAR.md`'s ownership section states the
principle this decision applies: "A surface adds usability, never
functionality" (owner, 2026-08-11). The owner has since named the human
surfaces this repo actually keeps — `sgt init`, `sgt doctor`, `sgt tui`
(ADR 0010), and the new homepage (ADR 0010) — and the dashboard is not
among them. The arbitration record's own argument for *how* to remove it
is adopted along with the decision to remove it: delete rather than
freeze-and-stub, because a disabled stub that still owns two open
reactivation issues forever is a maintenance claim this repo would not
actually be honoring, and deletion is the lower rung.

**This ADR is the first actual ruling on the dashboard, not a restatement
of an old one.** The provenance matters and is recorded precisely:
`north-star-arbitration-2026-08-11.md:196` proposed this outcome three
days before this interview; the dispositions record from that same
adjudication never ratified it; `NORTH-STAR.md` kept the dashboard listed
as a surface and its Wave 3 plans afterward; and #11 — one of the two
issues the proposed freeze would have kept pinned open — was fixed in the
meantime. Nothing between the arbitration and this interview ever actually
decided the dashboard's fate; this decision is that first ruling.

## Alternatives considered

**Freeze `src/tui.rs` at its P0 proof and disable-but-stub `src/web.rs`**
— the arbitration's own originally-argued shape, which would have kept
`sgt web`'s route disabled while leaving the code in place and #15/#21
open as live reactivation issues — is the alternative on record, and is
rejected here on the arbitration's own stated reasoning: a stub carrying
two open issues indefinitely is a maintenance claim, and deletion commits
to less than that stub would.

## Consequences

This is a real deletion with real test fallout, not a documentation-only
change once implemented: `src/web.rs` (779 lines) and `web/` (224 lines)
are removed, `sgt web` stops being a verb, and `tests/m6_surfaces.rs`'s
dashboard-specific tests go with it. None of this deletion is performed by
this ADR, which records the decision only.

ADR 0009's no-spawn sweep list loses `web` as an entry — there is no
longer a dashboard verb for the no-auto-spawn rule to apply to.

Issues #21 and #15 close as won't-do rather than being fixed or kept open
for later reactivation; anyone reading either issue after this ADR should
not expect the dashboard route they describe to return.

## Open questions

The exact scope of "m6's dashboard tests go with it" — which specific
tests in `tests/m6_surfaces.rs` are pure dashboard tests to delete outright
versus tests that exercise the three-clients-through-one-API invariant
more broadly and need rewriting rather than deletion — is not enumerated
in the interview; that triage is implementation work this ADR does not
perform.
