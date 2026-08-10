# Evidence policy — citation and quote+hash discipline

Layer 3 (stable across runs), shared by `20-harvest` and `30-normalize` — the
two stages of this workflow that mint or carry forward source-cited
behavior-unit records. This file is the operative, workflow-owned
distillation of `docs/icm/record-shapes.md` §3 (behavior-unit record) and the
lesson recorded there as L11 in `LESSONS.md`; it does not amend either
document, and if the two ever disagree, `docs/icm/record-shapes.md` governs
and this file is stale and should be corrected (edit-source principle,
`docs/icm/convention.md` §1a rule 3).

## The required fields

Every behavior unit is one NDJSON line with exactly these fields (no more,
no fewer, at this stage — no `representation`/`workflow`/`stage`/`rationale`/
`alternatives_considered`/`engine_gap`; classification is a later stage's
job, not `20-harvest`'s or `30-normalize`'s):

```json
{
  "id": "BU-0042",
  "statement": "Before changing a repository, verify that the requested repository belongs to the loaded project.",
  "source": {
    "path": "AGENTS.md",
    "locator": "Standard Workflow / Load context",
    "quote": "Before making changes to a repository, first verify that the\nrequested repository belongs to the currently loaded project.",
    "quote_hash": "sha256:<hex>"
  },
  "scope": "cross-repository work",
  "trigger": "a work request names or implies a project repository",
  "outcome": "repository membership is established before mutation",
  "authority": "user-context actor",
  "confidence": "high",
  "notes": "optional: mechanism/intent separation, open questions"
}
```

| Field | Rule |
|---|---|
| `id` | Unique within this run's corpus, stable once assigned. `BU-####` (zero-padded, monotonic) unless the run's contract names a different scheme — pick one and hold it for the whole run. |
| `statement` | Normalized, implementation-independent language. One behavior only — see "One behavior per unit" below. |
| `source.path` | Path to the source artifact, relative to the subject repository's root as pinned in `../00-contract/output/contract.md`. |
| `source.locator` | Precise enough for a human to re-open the exact evidence: a heading path, function name, line range, or test name. "Somewhere in the file" is not a locator. |
| `source.quote` | The quoted source text, **verbatim**, ≤500 characters. A longer span records its first 500 characters here plus a `span_bytes` count; the hash still covers the full span. This field is not optional and not a summary — without the preimage the hash below cannot be checked by anyone (L11). |
| `source.quote_hash` | `sha256:<hex>` over **the exact contiguous byte span quoted from the cited file — no normalization, no trimming beyond the span choice itself.** Compute it, don't estimate it: hash the literal bytes you are about to put in `quote` (or the literal bytes of the full span, if `quote` is a 500-char prefix). |
| `scope` | The procedural context the behavior applies within — not yet a classification, just where to look. |
| `trigger` | What condition makes the behavior apply. |
| `outcome` | What durably differs once the behavior has been followed. |
| `authority` | Who or what enforces or performs it. |
| `confidence` | `high` / `medium` / `low` — how directly the source supports the statement. Never a substitute for citation: even `low` carries a real, checkable `quote_hash`. |
| `notes` | Optional. Use it for mechanism/intent separation (see below), open questions, or a `citation: disputed` marker (see "When a citation cannot be verified"). |

## Verify the hash yourself before writing it down

A `quote_hash` that does not verify against `quote` — or a `quote` that does
not appear contiguously at `source.locator` in `source.path` — is
indistinguishable from invention, and later lint (`70-lint`) rejects it on
that basis, not on trust. Do not write a hash you have not actually computed
against the exact bytes you copied.

**The hash always covers the exact raw bytes of the quoted span as they sit
in the source file — never the JSON-escaped string form the NDJSON line
itself uses for `quote`.** JSON escaping (a literal newline becoming `\n`
inside the record) is a serialization detail of this NDJSON line, not a
change to the span; a literal newline byte in the file and the two
characters `\`+`n` in the JSON text must hash identically, because both
represent the exact same underlying byte.

**Extraction is capture-once, reuse-twice**, so `quote` and `quote_hash`
can never drift from each other over a stray trailing newline:

```sh
QUOTE="$(sed -n '3,5p' path/to/file)"    # command substitution strips the
                                          # trailing newline(s) `sed` would
                                          # otherwise add — $QUOTE is now
                                          # the exact span, byte for byte
printf '%s' "$QUOTE" | sha256sum         # hash exactly what you captured
```

Use `$QUOTE`'s captured value verbatim as `quote` (JSON-escape it for the
NDJSON line — escaping is encoding, not a change to the bytes hashed), and
the `sha256sum` output as `quote_hash`. Do not retype the span by hand and
do not hash a separately-typed value — capture once with the command above,
then reuse that single captured value for both fields; retyping risks
whitespace/line-ending drift that silently breaks the hash. This is also
exactly what `80-adversarial-review` reproduces when it recomputes
`quote_hash` independently (`printf '%s' "$QUOTE" | sha256sum` against the
byte range it identifies itself) — following the identical recipe here is
what makes that later recomputation agree with this one.

## One behavior per unit

A `statement` that conjoins independently-triggerable behaviors ("verify X
and also log Y and also notify Z") is not one unit — split it into separate
units, each independently citing the evidence that supports it (the same
`source.quote`/`quote_hash` may back more than one unit when the source span
genuinely states more than one behavior).

## Mechanism versus behavioral intent

Record the *durable behavior*, not the *old repository's specific mechanism*
for achieving it, in `statement`. Where the source names a particular
script, sentinel file, tool invocation, or other implementation detail that
is not itself the behavior, keep that detail — it is useful — but put it in
`notes`, not `statement`. A `statement` that only makes sense in terms of a
mechanism a future implementation might not have is a violation of the
normalization this record exists to do.

## When a citation cannot be verified

A unit whose statement cannot be re-anchored to a real contiguous span in
the cited file is not silently dropped. Mark it `"confidence": "low"` and
add a note containing the literal text `citation: disputed`, explaining what
was attempted and why it did not verify. An unverifiable citation is itself
a fact worth recording, not an error to hide.

## What this file does not cover

The ICM decomposition ladder, representation vocabulary, and classification
record shape (§4/§5 of `docs/icm/record-shapes.md`) are out of scope here —
this workflow's `20-harvest` and `30-normalize` stages produce and rewrite
*uncommitted-to-any-representation* behavior units only. Classification is a
later stage's contract.
