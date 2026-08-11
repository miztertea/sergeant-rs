# resources/ — gauntlet workflow scripts, as launched

The single home for every orchestration script the gauntlet loop has run,
committed as plain `.js` (owner direction 2026-08-11; supersedes the
`reference/gauntlet-workflows.zip` archive, whose full contents were
extracted here at the migration commit — git history retains the zip
itself). Plain files over an archive because scripts are *method
evidence*: diffs between successive milestones' scripts are where the
economy revisions and protocol changes are visible, and a zip hid exactly
that.

Conventions:
- One file per workflow invocation, committed **as launched**; edits after
  launch land as new commits, never rewrites.
- Subfolders by program series. New series add a folder; nothing is
  renamed retroactively.

- `m-series/` — M0–M6 build gauntlets + lean round-2 scripts (P0, ledger
  entries M0–M6).
- `n-series/` — N1 decomposition/closure/fixer, N2 build/run/compare
  (runs 1–3), N3 wave-1 build + run-3 compare (Programs A/B, PR #27/#43).
- `s-series/` — S1 instrument + round 2 + analysis, S2 waves 1–3
  (coverage/stabilization, PR #28). Wave 2 introduced the guard-map /
  independent-prober protocol now standard in
  `reference/notes/gauntlet-pattern.md`.
