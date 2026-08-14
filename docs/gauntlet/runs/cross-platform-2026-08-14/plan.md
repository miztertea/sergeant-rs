# Cross-platform sprint — as-planned record, 2026-08-14

The wave plan this integration branch executes, written before the work ran
so the plan can be judged against the outcome rather than reconstructed from
it. Outcome and envelope land in a sibling `close-out.md`.

Origin: the `grill-with-docs` interview held live in-session on 2026-08-14
(R-NS-6 — interviews are never dispatched Work), whose ten decisions are
recorded in `docs/adr/0001`–`0004` and `docs/glossary.md`. This sprint is
the implementation half of those records.

## Waves

Ordered by dependency, not by priority. Groups within a wave run
concurrently; a wave starts when the prior one has merged.

| Wave | Group | Issues | Grouping rationale |
|---|---|---|---|
| 1 | G1 — test integrity | #83, #70 | Both live in the test harness; same file area, so one worker rather than two racing. Blocks G4 |
| 1 | G2 — TUI liveness | #16 | Single ticket. `src/tui.rs` only, no overlap with G1 |
| 2 | G3 — platform boundary | #81, #82, #18 | ADR 0002 made real. One `src/platform/` created once and all three facts moved behind it; three separate Works would invent three shapes |
| 2 | G4 — scripts + CI | #86, #87 | #86's shellcheck half *is* a CI change; same `.github/workflows/` files as #87 |
| 3 | G5 — storage preconditions | #85 | Single ticket by dependency: its filesystem detection belongs behind G3's boundary, not as another ad-hoc `/proc` read |
| 4 | G6 — perf observations | #12, #10 | Batched, not synergistic — different subsystems, no shared files. Grouped only because both are measure-then-decide |
| 5 | G7 — RSS measurement | #8 | Measurement only. #8's own filed falsifier is a 10+ rep read-burst run; no code |
| 5 | G8 — RSS eviction | #4 | Runs on G7's verdict, not in parallel with it |

## Dependency edges, and why each exists

- **G1 → G3**: both touch `src/backend/claude.rs` (#83's version-probe error
  surfacing; #18's `session_liveness_excluding`). Different functions, same
  file — the conflict shape that forced #26/#11 into one Work on 2026-08-13.
- **G1 → G4**: #87's own text requires the flaky suite settled first. Three
  CI lanes multiply spurious reds, and a flaky matrix trains people to
  ignore CI, which is worse than no matrix.
- **G3 → G5**: #85 detects filesystem type from `/proc/mounts` — a platform
  fact. Landing it first adds a read that G3 then has to move.
- **G3 → G6**: #12 touches doctor, which G3 and G5 both edit.
- **G7 → G8**: measure before fixing. #8's verdict (slow leak vs asymptote)
  changes what #4 should do.

## Model policy

Sonnet throughout, per owner direction (2026-08-14). Opus is earned at **G8
(#4)** alone: bounding or evicting terminal `Work` structs from
`WorkRegistry.works` sits on the journal-is-only-truth invariant and in the
adjacent-append hazard class (LESSONS L6).

## Acceptance shapes that differ from ordinary fixes

- **G1/#83 is diagnose-then-fix-if-proven.** The mechanism is a hypothesis,
  not a diagnosis: `ETXTBSY` via fd inheritance is consistent with every
  observation but unproven, and the issue records an invalid experiment that
  must not be repeated. The Work is authorised to report "not proven, here
  is what I ruled out" rather than fix the wrong thing confidently.
- **G3 ships macOS arms stubbed and explicitly unmeasured** (owner
  direction). We have no macOS host in the estate yet, and
  `docs/DEVELOPMENT.md`'s rule is measured-not-assumed. #18 therefore stays
  open after G3 and closes in a later session on the MacBook. G3's
  acceptance is "boundary created, Linux arms verified", not "#18 closed".
- **G7 produces a finding, not a diff.** A Work whose durable outcome is a
  recorded measurement is a legitimate outcome; it must not invent a fix to
  look productive.

## Not in this sprint

**Owed an owner ruling** (R-NS-6 — a live interview, never a Work item):
#80 data-dir identity contract and the manifest asymmetry; #64 surfaces_root
default flip; #68 auto-spawn consistency sweep; #60 actor env contract; and
the **dashboard disposition**, which settles #21 and #15 together.

That last one was checked rather than assumed during planning.
`north-star-arbitration-2026-08-11.md:196` proposes deleting `src/web.rs` +
`web/` + the `sgt web` verb, citing "T-series §5.1 item 14 already proposes
disabling the route and leaving a stub with two live reactivation issues
(#15, #21)". That is the **argument record, not the ruling**:
`north-star-dispositions-2026-08-11.md` contains no mention of web,
dashboard, or freeze; `NORTH-STAR.md` still lists the dashboard as a surface
and its Wave 3 actively plans `#11/#16`; and `src/web.rs` is still 779 lines
with `sgt web` still a verb. #11 was fixed on 2026-08-13, which the proposed
freeze would have foreclosed. So #21 and #15 are neither closed nor built —
they wait on a disposition that is itself owed (LESSONS L12: summaries are
orientation, not authority).

**Gated on an unblock condition**: #25 (Codex adapter — needs an environment
where Codex can be measured), #17's Rule C (needs the 1M-event progressive
load measurement).

**Closed during planning**: #78 (stale `docs/img/tui-fleet.png`) — closed
won't-do by the owner, superseded by a planned TUI redesign.
