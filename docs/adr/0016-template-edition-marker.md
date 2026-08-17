# ADR 0016: Templates carry an `edition` field instead of a diff verb

**Status:** Accepted, 2026-08-17.

## Context

ADR 0014 decision 3 rules that shipped workflow packages are templates, not
published procedure: they are "ways you could work," not "how you should
work," and a user is expected to fork one into `.sergeant/local/` and
rewrite it. Decision 4 records that Captain proposed a `sgt workflow diff`
verb to detect drift between a fork and current stock, then withdrew it
under CONSTRUCTION R1 once templates were reframed as examples — there is
nothing to diff *against* a body the user is expected to have rewritten
freely. But the corpus's underlying complaint survives the verb's
withdrawal: a fork has no invalidation mechanism at all. No edition number,
no survey date, no revision program. A user who forked `diagnose-bug` eight
releases ago has no way to tell whether stock has since fixed a bug, added
a stage, or reworked the whole approach their fork is still descended from.
Decision 4's ruling is that an edition marker on each template answers
this, not a verb.

This ADR is Phase 2b of the decontamination work
(`reference/proposal-product-workspace-split.md` §6, Phase 2 — Template
decontamination): apply that marker to the 17 packages under
`.sergeant/workflows/` and to `skills/`, and define its shape in
`docs/icm/record-shapes.md`, which already owns front-matter shapes
(§7.3, `docs/icm/record-shapes.md` §1).

## Decision

**Every shipped template's front matter carries an `edition` field: the
distro version (`Cargo.toml`'s package `version`, e.g. `0.1.0` — ADR 0014
decision 2's co-versioned `v0.x.y` identity) that wrote it as stock
content.**

- `edition` lives in the same front matter `record-shapes.md` §1 already
  defines for workflow `index.md`, and is added to `skills/*/SKILL.md`
  front matter under the same name and meaning (CONSTRUCTION R2: reuse the
  shape already defined rather than invent a sidecar one for a second
  directory).
- `sgt init`/update sets `edition` to the current distro version whenever
  it writes a stock copy. A copy the user forks into `.sergeant/local/`
  keeps whatever `edition` value it had at fork time — nothing rewrites it
  afterward, including edits the user makes to the fork's body.
- Drift becomes a plain string comparison: a fork's `edition` against the
  currently-shipped stock package's `edition` for the same name. Unequal
  means the fork predates the current stock edition; equal means it
  doesn't. No parsing beyond reading one front-matter field, no diff, no
  merge.
- This ADR adds the field and its definition only. It does not implement a
  reader, a `sgt doctor` check, or any engine change — per the milestone's
  own constraint, this rung does not require the engine to learn new
  vocabulary. Comparing editions is future work the field makes possible,
  not work this ADR performs.

## Alternatives considered

- **Content hashing the template body.** Rejected on CONSTRUCTION grounds
  before it could pass R6: a hash is checkable by string comparison too,
  but only after adding a hashing step somewhere in the toolchain, and it
  answers the wrong question. It would detect that a fork's *body* differs
  from current stock — true of nearly every fork on day one, since forking
  is an invitation to rewrite the body (ADR 0014 decision 3). What a user
  actually needs to know is not "does my body differ" but "how stale is
  the stock edition I forked from," which a body hash cannot answer:
  a fork with a byte-for-byte-unchanged body still needs to know if stock
  has moved on, and a fork with a heavily rewritten body still descends
  from one particular stock edition either way. An edition field answers
  the real question directly; a hash would answer an adjacent question and
  require the user to separately track which edition the hash was taken
  against, which is the same missing record this ADR is meant to supply.
- **A sidecar manifest** (e.g. `.sergeant/workflows/.editions.json` or one
  file per package recording its provenance). Rejected: the constraint is
  that the marker live in existing front matter, not a new file
  (Ponytail R2/R6 — reuse the `index.md` front matter `record-shapes.md`
  already defines rather than open a second file per package to track one
  field). A sidecar also invites the exact drift it exists to prevent: the
  manifest and the template it describes can independently go stale
  relative to each other the moment either is edited without the other,
  where front matter travels with the file it describes by construction.
- **A survey/revision date instead of a version string.** Considered and
  rejected as the *comparison key*, though the underlying idea (some
  marker of when a fork was taken) is what `edition` already is. A date
  requires the reader to separately know which distro version shipped on
  that date to reason about what changed; the version string used by
  ADR 0014 decision 2's co-versioning already carries that information
  directly and is the identity the binary and doctrine are versioned by
  regardless of this ADR.
- **Bumping the workflow's own `version` field to double as the edition
  marker.** Rejected: `record-shapes.md` §1 already defines `version` as
  counting changes to *this specific package's* stage sequence and
  context content — a per-package author-controlled counter, bumped
  independently by whoever last published a change to that package.
  Overloading it to also mean "distro release this was shipped in" would
  make it impossible to tell, from the field alone, which meaning a given
  bump reflected, and would force every package's `version` to advance
  in lockstep with every distro release whether or not that package
  changed — the opposite of what `version` currently signals.

## Consequences

- Seventeen `.sergeant/workflows/*/index.md` files and five
  `skills/*/SKILL.md` files now carry `edition: 0.1.0`, matching
  `Cargo.toml`'s current package version. Every future distro release that
  touches a template's stock content must set that template's `edition` to
  the new release version when writing it — a release that forgets this
  silently defeats the marker for every user who forks after that release.
- `docs/icm/record-shapes.md` §1 is the sole normative definition of
  `edition`; a future `sgt doctor` check or discovery surface that wants to
  report drift reads this ADR and that section rather than inventing its
  own semantics.
- The marker records descent, not correctness. A fork whose `edition`
  matches current stock could still have a body that diverges completely
  from what stock does today, because forking is expected to mean
  rewriting. `edition` only ever answers "how far back does this fork's
  stock ancestry go," never "does this fork still behave like stock."

## Open questions

- Whether `sgt doctor` should surface `edition` drift, and in what form
  (a warning, a count, a per-package list) is unresolved here — decision 4
  rules there is no diff *verb*, but says nothing about whether `doctor`
  reports the comparison this field now makes possible. Left to whichever
  milestone implements the reader.
- Whether editions should ever be allowed to skip (e.g. a template
  unchanged across three releases keeping the old `edition` rather than
  being rewritten to the newest version number on every release) is not
  decided. This ADR assumes `sgt init`/update always writes the current
  distro version into every stock file it touches, which is the simpler
  rule and does not preclude a later change if a real skip case turns up.
