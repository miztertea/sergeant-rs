# Identify spec source

Resolved as `@@identify-spec-source` from
`.sergeant/common/contexts/identify-spec-source.md` per
`docs/icm/convention.md` §4. Shared stage context, two or more consumers:
`implement-change/00-orient`, `review-change/10-identify-spec-source`,
`author-document/00-map-sources`.

Locate the intent, spec, or acceptance criteria a run will be judged
against, by a fixed priority order ending in asking — never by inventing
one because none was obviously lying around.

## Contract

- **Priority order:** an explicit reference in the intent or a commit
  message (an issue link, a spec path); then a path the user or intent
  supplied directly; then a matching spec/PRD/brief file under the
  repository's own convention (`docs/`, `specs/`, or wherever the caller's
  own package states); then — if nothing surfaces — ask.
- **If the intent explicitly states there is no spec**, that is a valid,
  recorded answer: "no spec source" is written down, not filled in with an
  invented standard.
- **The located source is recorded by path or pasted contents**, whichever
  makes it reusable by a later stage (a spawned seat has no other access
  to this Work — see `@@fan-out-evidence`/`@@panel`).

## What this context contributes when loaded inside a stage

- **J0 the caller must honor:** no spec/acceptance source is found
  anywhere in the priority order *and* the intent does not itself state
  acceptance criteria — stop and ask rather than judging the work against
  a standard nobody actually set.
- **J2 the caller retains:** judging whether a candidate file genuinely
  matches the branch, feature, or intent name among several plausible
  ones — the priority order names where to look, not which of several
  hits is the real one.
- **J1 the caller retains:** none beyond ordinary tool mechanics; the
  priority order itself is fully specified and admits no equivalent local
  variant.

There is no stage library in this engine. This file is shared text pulled
into a stage's own `CONTEXT.md` by `@@` reference, not a stage of its own.
A change here must be hand-propagated to every consumer's own narrowing —
this is drift by construction, and it is named here rather than hidden.
