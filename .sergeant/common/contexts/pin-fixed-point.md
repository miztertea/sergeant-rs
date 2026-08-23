# Pin fixed point

Resolved as `@@pin-fixed-point` from `.sergeant/common/contexts/pin-fixed-point.md`
per `docs/icm/convention.md` §4. Shared stage context — two or more
consumers across the distro's packages (`docs/icm/convention.md` §1 rule
2): `implement-change/00-orient`, `review-change/00-pin-fixed-point`,
`investigate/00-frame`.

Pin the revision a run is judged against, and confirm it before anything
downstream depends on it: the fixed comparison point must resolve, and
where the point is meant to bound a diff, that diff must be non-empty.
Fail here, not inside a spawned seat three stages later where a bad ref
just produces an empty, silently-passing report.

## Contract

- **The fixed point is whatever the intent or the user names** (a commit
  SHA, branch, tag, `HEAD~N`, or — for `investigate` — simply "the current
  revision" when no diff is in play). The caller states which of these
  shapes applies to its own trigger.
- **Confirm it resolves** (`git rev-parse` or the equivalent) before
  proceeding. A ref that does not resolve is not a judgment call to work
  around; it is a stop.
- **Where the caller's contract involves a diff**, confirm the diff
  against the fixed point is non-empty before spawning anything downstream
  that depends on it existing.
- **Nothing downstream may re-pin.** Once confirmed, every later stage in
  the same run treats this revision as fixed; a stage that finds itself
  wanting a different comparison point has found a new run, not a local
  adjustment.

## What this context contributes when loaded inside a stage

Per `docs/icm/convention.md` §7.4: this file does not claim authority
independently of its caller; it names the J0 the caller must honor and the
J2/J1 the caller keeps.

- **J0 the caller must honor:** no fixed point was given and none can be
  inferred from the intent — ask, never guess `HEAD~1` or `main`. A
  resolution failure (bad ref, empty diff where one is required) is the
  same J0, not a retry-with-a-guess.
- **J2 the caller retains:** exactly how the fixed point is phrased back
  to the record, and, for a diff-bearing caller, the exact diff command
  used to capture it.
- **J1 the caller retains:** wording of the failure message when the ref
  doesn't resolve.

There is no stage library in this engine. This file is shared *text*
pulled into a stage's own `CONTEXT.md` by `@@` reference — never a stage
of its own, never included or inherited. A change here does not
propagate: every consumer's own narrowing has to be re-read by hand
against it, which is drift by construction, the same class of defect as a
stale pointer to a package that no longer exists.
