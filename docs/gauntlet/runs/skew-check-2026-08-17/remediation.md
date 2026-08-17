# Doctrine-side skew remediation — 2026-08-17

Fixes findings 1–4 from `findings.md` (Phase 0 skew check, run on
`sergeant/01M0748HEH4SHSCFX0MCN05PY0`). Findings 5 and 6 are engine
defects, filed as issues #164 and #165 respectively, and out of scope
here — nothing in `src/` was touched.

## What changed

**Finding 4 (ARG_SHAPE) — `sgt work retry` → `sgt retry`.**
`.sergeant/workflows/repo-to-icm/20-harvest/references/partition-checkpoint-protocol.md`,
lines 14 and 110: corrected both citations from the nonexistent
`sgt work retry` to the real top-level `sgt retry <ID>`. Mechanical fix —
verified via `sgt retry --help` and `sgt work --help` (the `work`
subcommand's only children are `list`, `show`, `transcript`, `retained`,
`reap`).

**Finding 1 (FLAG_MISSING) — `--intent-file`.**
`.sergeant/workflows/dispatch/05-classify-risk/CONTEXT.md`: the stage
mandated, three times and in bold, that safety-sensitive objectives "must
be given an explicit `--intent-file`," calling it "not a delegated
judgment call." `sgt run --help` has no such flag and no other
intent-transport mechanism exists (verified). Did not invent a
replacement flag or pretend one exists. Instead:
- Added an explicit engine-gap note under "Behavior contract" stating
  plainly that `--intent-file` does not exist in `sgt run` today, and
  that the only channel that reaches a Work at all is the plain-text
  positional `<INTENT>` argument, which the engine does not validate.
- Named the honest interim path: fold the required content into that
  plain-text argument by convention, and record in the stage's own
  output that the safety-sensitive path applied even though nothing in
  the CLI enforced it structurally.
- Left the keyword-match routing decision itself intact — that part
  *is* real and enforceable by the actor; only the enforcement mechanism
  is missing.
- Filed the gap: [issue #166](https://github.com/miztertea/sergeant-rs/issues/166),
  cross-referenced from both the "Behavior contract" section and the J5
  governing-constraint bullet.

**Finding 2 (FLAG_MISSING) — `sgt-watch --sync-all`.**
`.sergeant/workflows/dispatch/80-monitor/CONTEXT.md`, line 51: the
reconciliation trigger clause cited `sgt-watch --sync-all` as an
on-demand way to force bulk fleet reconciliation. `sgt watch` has no
`--sync-all` flag and is read-only by design (ADR 0009) — verified via
`sgt watch --help`. The *automatic* half of the claim (dispatch runs
reconciliation itself before creating new work) is real and unchanged.
Rewrote the trigger clause to say only the automatic half is true today,
and added an engine-gap note stating there is currently no CLI verb for
on-demand bulk reconciliation outside of that automatic run. Filed the
gap: [issue #167](https://github.com/miztertea/sergeant-rs/issues/167).

**Finding 3 (VERB_MISSING) — `sgt dispatch`.**
No engine gap here — the workflow-level capability this finding is about
(running the `dispatch` workflow) already exists as `sgt run --workflow
dispatch`; the defect was purely that no file in the package ever stated
that mapping, so "sgt-dispatch" throughout the package's stage prose
reads as a present-tense CLI verb. Added an explicit disclaimer section
("No `sgt dispatch` verb") to `.sergeant/workflows/dispatch/CONTEXT.md`
and a shorter pointer to it in `index.md`, naming the real top-level verb
list (`sgt --help`), quoting the `sgt dispatch --help` failure, and
stating the real invocation (`sgt run --workflow dispatch`). Did not
touch the ~60 individual "sgt-dispatch does X" bullets across the
package's stage files — they remain accurate as upstream-tool provenance
citations (the pattern the package already uses elsewhere, e.g. the
existing 2026-08-16 ICM-R3 correction in this same `CONTEXT.md`); the fix
is the one missing disclaimer that resolves the ambiguity for a reader,
not a rewrite of every citation.

## Filed

- [#166](https://github.com/miztertea/sergeant-rs/issues/166) — `sgt run` has no way to pass a fuller intent document (`--intent-file`), so the risk-classification stage's mandate is unenforceable. (Finding 1.)
- [#167](https://github.com/miztertea/sergeant-rs/issues/167) — No CLI-triggerable bulk fleet reconciliation (`sgt watch` has no `--sync-all`, nothing else exposes it on demand). (Finding 2.)

## Files touched

- `.sergeant/workflows/dispatch/05-classify-risk/CONTEXT.md`
- `.sergeant/workflows/dispatch/80-monitor/CONTEXT.md`
- `.sergeant/workflows/dispatch/CONTEXT.md`
- `.sergeant/workflows/dispatch/index.md`
- `.sergeant/workflows/repo-to-icm/20-harvest/references/partition-checkpoint-protocol.md`

`AGENTS.md` and `docs/DEVELOPMENT.md` were not touched (owned by a
separate Work). No file under `src/` was touched.
