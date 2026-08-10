# 80-adversarial-review: review summary

## Candidate packages reviewed

None. `.sergeant/drafts/workflows/` does not exist anywhere in this
worktree — `50-synthesize` never materialized a package, and `60-draft`
confirms it in its own relay ("no packages were materialized outside
`output/`"). There is no candidate package for Axis 1's publication/layer/
name-collision checks, Axis 2's provenance/hidden-translation checks, or
Axis 3's engine-gap checks to run against.

## Why: this run's own inputs are unresolved

Before applying any axis, this stage checked step 0 of its own `CONTEXT.md`:
whether any Inputs-table artifact opens with `# AMBIGUOUS — NOT RESOLVED`.
All six do — `00-contract/output/contract.md`,
`30-normalize/output/behavior-units.normalized.ndjson`,
`40-classify/output/classifications.ndjson`,
`50-synthesize/output/candidates.md`, `60-draft/output/draft-report.md`, and
`70-lint/output/lint-report.md`. `00-contract` failed to establish this
run's subject repository and pinned revision from the Work's initiating
task, and every intermediate stage (`10-inventory` through `70-lint`)
relayed that fail-closed state cleanly and honestly rather than inventing a
corpus to fill the gap. This is recorded as **AF-0001** (boundary-honesty,
high) — per this stage's own `CONTEXT.md` step 0, the propagation reaching
this stage's inputs is itself the finding to record, and this stage does
not review the marker-relay artifacts as if they were ordinary output.

## Axis-by-axis disposition

- **Axis 1 — Boundary honesty: applied, in full, where checkable.**
  - Publication/layer/name-collision boundaries: not applicable — no
    candidate packages exist to check them against.
  - Blindness boundary: applied. Grepped every artifact under
    `00-contract/output/` through `70-lint/output/` (the full Inputs-table
    chain plus every intermediate stage's output) for the literal string
    `reference-corpus`. Two hits, both inside `00-contract/output/contract.md`,
    both classified per `references/challenge-checklist.md`'s buckets: one
    is `contract.md`'s own exclusion-record entry (`find . -maxdepth 2
    -iname "*reference-corpus*"` — none found in this worktree, so nothing
    to exclude), the expected non-finding hit; the other is prose quoting
    `../_config/run-discipline.md`'s own worked example verbatim
    ("graded against `reference-corpus/`"), which is bucket (b), also not a
    finding. No hits anywhere else in the run's output tree. Neither hit
    sits inside a citation field (no `source.path`/`source.locator`/`quote`
    naming a location inside `reference-corpus/`), so no contamination
    finding applies.
  - As part of this axis's "approach every artifact assuming it might
    contain an error" instruction, this stage additionally re-ran the shell
    checks `00-contract/output/contract.md`'s own "What was checked" list
    cites, to verify the foundational document's evidence itself (the one
    substantive artifact this run actually produced). This surfaced
    **AF-0002** (invention, medium): the claim that
    `git -C reference/sergeant-upstream rev-parse --is-inside-work-tree`
    "fails" does not reproduce — it returns `true`. The sibling check
    (`ls reference/sergeant-upstream/.git` failing) does reproduce and is
    the check that actually controls the vendored-subtree classification
    per `00-contract/CONTEXT.md`'s own governing rule, so this discrepancy
    does not appear to overturn the ultimate AMBIGUOUS determination — it
    is recorded as a real evidentiary weakness, not a claim that the
    determination itself is wrong.
  - Every other checkable factual claim in `contract.md`'s "What was
    checked" list was independently re-verified and reproduced correctly:
    the worktree's top-level contents (`.sergeant/`, `AGENTS.md`,
    `reference/sergeant-upstream/` only, plus the `.git` worktree pointer),
    the absence of any `UPSTREAM.md` anywhere in the worktree, the absence
    of a repository-level vendoring/pin statement in
    `reference/sergeant-upstream`'s `.gitignore`/`README.md`/`AGENTS.md`
    headers, and that `.agents/skills/PROVENANCE.md` is the only
    provenance-shaped document found (a grep for `vendor` across
    `reference/sergeant-upstream/**/*.md` turns up only that subtree's own
    internal skill-vendoring mechanism, not a pin for the subtree itself).
    The Work's `intent` field, independently queried via
    `sgt --data-dir ... --json work show 01KZNT2Y5BX7S26PJB3B1QVADW`, matches
    `contract.md`'s quote exactly (no subject path, no revision named). This
    stage also confirmed that `00-contract/CONTEXT.md` §1's own rule
    explicitly forecloses using the outer worktree's `git rev-parse HEAD`
    (`d27227d9b8203705998f4c79370440def577b619`) as a substitute pin for the
    vendored subtree's own provenance — so that path, considered and
    rejected during this review, is not itself a finding.

- **Axis 2 — Invention: not applicable in its literal form, applied in
  spirit.** `30-normalize/output/behavior-units.normalized.ndjson` and
  `40-classify/output/classifications.ndjson` contain no behavior-unit or
  classification records (only the AMBIGUOUS relay prose) — there is no
  citation sample to draw, no `quote_hash` to recompute, no rationale to
  check for discrimination, no `stage`-rung record to re-apply the
  reimplementation test to, and no materialized package to check for hidden
  translation. This stage instead applied the axis's underlying
  "re-verify claims yourself" principle to the one substantive document
  this run actually produced, `contract.md`'s own evidentiary claims —
  see AF-0002 above.

- **Axis 3 — Engine-gap refutation: not applicable.** No
  `representation: engine-gap` records exist anywhere in this run's output
  (`40-classify/output/classifications.ndjson` contains no records at all).
  Nothing to re-attempt lower rungs against.

## Finding counts

| Axis | High | Medium | Low | Total |
|---|---|---|---|---|
| boundary-honesty | 1 | 0 | 0 | 1 |
| invention | 0 | 1 | 0 | 1 |
| engine-gap-refutation | 0 | 0 | 0 | 0 |
| **Total** | **1** | **1** | **0** | **2** |

Two findings total, in `output/findings.ndjson`: AF-0001 (boundary-honesty,
high) and AF-0002 (invention, medium). No accept/reject disposition is
assigned here — that is `90-reconcile`'s job, per this stage's own
`CONTEXT.md`.
