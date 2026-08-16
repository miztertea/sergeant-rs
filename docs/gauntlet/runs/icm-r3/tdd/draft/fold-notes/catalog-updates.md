# Proposed catalog updates (not live edits — draft only)

## `.sergeant/index.md`

Current line 39:

```markdown
| `tdd` | published | [`workflows/tdd/index.md`](workflows/tdd/index.md) |
```

Remove this row on promotion — `tdd` is no longer a `.sergeant/workflows/`
entry once rehomed. `.sergeant/common/contexts/` is not itself catalogued
in `index.md` (the index's own scope is `status: published` workflows,
`record-shapes.md` §1 rule 2), so no replacement row is added; `@@tdd` and
`@@test-quality` become discoverable the way every other shared context is
— from the consuming workflows that reference them.

## `AGENTS.md`

Line 233 currently lists `tdd` among "published workflows" the 126-unit
corpus rewrite consumed content into:

```markdown
Most of the 126-unit corpus this rewrite consumed belongs to specific
published workflows (`tdd`, `prototype`, `wayfinder`, `to-tickets`,
`triage`, `diagnose-bug`) rather than to this always-on file...
```

On promotion, drop `tdd` from that list (it is no longer a published
workflow) or rephrase the sentence to "published workflows and shared
contexts" if a future pass wants one sentence to cover both classes.
Line 179's "TDD-first implementation" reference needs no change — it
already refers to the discipline, not the package's placement.
