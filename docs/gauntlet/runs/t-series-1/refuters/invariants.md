# T-SERIES-1 — refuter: invariants

Refuting `docs/gauntlet/runs/t-series-1/critics/invariants.md` against
`reference/proposal-tui-t-series.md`, per
`docs/gauntlet/contracts/T-SERIES-1.md`. I did not write the proposal or the
critic findings. Independently re-verified every factual claim against the
repository as it stands now — the live `tests/m6_surfaces.rs`, `NORTH-STAR.md`,
`docs/DEVELOPMENT.md`, `reference/notes/ideaos-agent-contract.md`, ADR 0009,
and `src/api.rs`/`src/cli.rs` — rather than taking the critic's citations on
trust.

---

## Finding 1: `inv-estate-doctor-bypasses-client-boundary` — CONFIRMED, error severity holds

This is the unit's only error-severity finding, and the one I attacked
hardest.

**Does T2-14 actually propose a second crate-internal reach path, or could
"narrow typed local operations" mean something that stays inside `ApiClient`?**
Re-read §5.3, §16.2, §16.3 in full, not just the quoted fragments. §5.3's own
code block is unambiguous about which facts route where:

```text
daemon-owned facts     ApiClient only
estate manifest edits  shared local Estate operations
installation checks    shared local Doctor report
```

This puts "shared local Estate operations" and "shared local Doctor report"
in explicit contrast to the "ApiClient only" row, not underneath it. §16.2
confirms the same reading operationally: "CLI owns Clap/stdout/JSON/exit
code. TUI owns forms/focus/rendering" around a shared Rust function
(`list_repositories`, `add_repository`, ...) — a division of labor between
two direct callers of the same function, not a division between an HTTP
handler and an HTTP client. §5.3 also states outright that this "is an
explicit refinement of the old 'TUI imports only ApiClient' source-scan
rule" — the proposal's own words concede the rule is being changed. The
critic's reading is correct; there is no API-preserving interpretation of
T2-14's own text.

**Does `tests/m6_surfaces.rs` actually enforce what the critic says?**
Re-read `t5_the_tui_is_a_client_like_any_other` (`:2308`) and
`t5b_the_structural_scan_sees_every_spelling_of_a_path` (`:2348`) directly,
not from the critic's excerpt. Confirmed verbatim: t5 asserts
`crate_paths(&source) == vec!["api"]` over all of `tui.rs`'s non-test code,
and t5b exists specifically because an earlier disposable-copy experiment
left the structural scan green while `tui.rs` held a live handle on the
daemon's blob store — the doc comment above t5 says so directly ("the test
that fools it, `t5b` below"). This is a hardened gate built in response to a
measured regression attempt, not an incidental default. `crate::estate::...`
or equivalent for the "shared local Estate operations" T2-14 specifies would
add a second entry to `crate_paths`, which t5 rejects by construction. The
critic's technical claim is exactly right.

**Does ADR 0009 actually establish that `tui.rs` only ever runs with a live
daemon, closing off the CLI's stated justification?** Read ADR 0009 in full.
"No exceptions (D5)... the TUI join[s] `sgt doctor`, `sgt watch`, and
`sgt daemon stop` in the no-spawn set." The ADR's own "Consequences" section
records the owner rejecting a TUI carve-out on exactly the theory §5.3
implicitly leans on (a human-facing surface that "just works" regardless of
daemon state) — "granting the TUI an exception for being human-facing would
have reintroduced the same failure this decision otherwise closes everywhere
else." T2-16 in the proposal restates the same ruling for T-Series. This
means the only reason §23.3 gives for rejecting a new-daemon-route
alternative — "would distort local/no-daemon semantics" — describes a cost
that is categorically impossible for `tui.rs` to incur, since `tui.rs`
never executes without a reachable daemon. The critic's central move (the
CLI's no-daemon requirement is being used to justify weakening a test that
only constrains the *other* client) survives this check.

**Is there a design precedent that makes T2-14's direct-import model the
long-intended architecture rather than a T-Series-specific slip, which
would soften the finding?** I went looking for one and found partial
support that, on inspection, corroborates the finding instead of undercutting
it. `docs/gauntlet/notes/estate-manifest-design-2026-08-11.md`'s "three pens,
one file" section, written the same day as NORTH-STAR.md, says: "CLI verbs
... are conveniences editing the same file ... TUI later = the same verbs
with a screen" — i.e., the original manifest design already anticipated the
TUI calling the same local edit functions the CLI calls, which is what T2-14
now proposes. But this note predates t5b, which was added later specifically
to close a regression where a "same verbs" style reach fooled the original
(pre-t5b) version of this scan. The design note is evidence that T2-14's
mechanism isn't invented carelessly — but it is exactly the kind of stale
architectural intent the codebase moved past when it hardened the test, and
the proposal cites neither the note's precedent nor the test it now
collides with. If anything this raises my confidence in error severity: an
implementer who trusted the older "same verbs" design intent, as the
proposal itself apparently does, has no signal anywhere in the 2,300-line
document that a currently-green, deliberately-hardened test sits directly in
that path.

**Does the proposal already name this trade-off anywhere, softening the
"never once names `tests/m6_surfaces.rs`, `t5`" claim?** I grepped the
full document for `m6_surfaces`, `t5`, and `source-scan` independent of the
critic's own search. Found only §5.3's abstract reference to "the old ...
source-scan rule" — no file name, no test name, no acknowledgment anywhere in
§16 (the extraction spec), §19.5/§19.6 (Estate/Doctor parity tests — read in
full; both test the CLI/shared-function output parity, never the client
import-boundary), or §21 item 40 ("Estate/Doctor behavior is shared through
small typed extractions, not duplicated" — stated as an achieved acceptance
fact, adjacent to item 39's still-intact "Work daemon facts still enter only
through ApiClient," with no distinction drawn between the two paths' test
consequences). Confirmed as claimed.

**Non-goals check.** Not grading an unbuilt implementation — the finding is
about what T2-14's text claims and its silence on a named test it changes,
not about missing code. Not re-litigating any owner ruling — the finding
accepts both R-NS-derived boundaries (equal clients; no-daemon TUI) as
settled and only disputes that T2-14 as worded can be enacted without
touching one of them unstated. Not designing the implementation — both of
the critic's offered fixes (extend the API; or explicitly name and revise
t5) are pointed at the gap, not prescriptive of which one wins.

**Severity.** No explicit severity rubric is stated in this proposal's
contract, so I applied the same bar the FOUNDATION-1 refuter precedent used
and this contract's own language supports: a section is error-severity if it
cannot be enacted as written without silently violating a currently-enforced
invariant. T2-14 fails that bar directly — dispatching §16.2/§16.3 as
literally specified either fails t5 on the first `cargo test --test
m6_surfaces` run (blocking the Work) or requires an implementer to
unilaterally decide to rewrite a named, hardened architecture test the
proposal never flags as being revised (producing an artifact — a weakened
client-boundary guarantee — the proposal never disclosed it was asking for).
Both outcomes match the error bar. Not inflated.

**Verdict: CONFIRMED. Severity unchanged (error).**

---

## Finding 2: `inv-t2-40-unjustified-r7` — CONFIRMED, warning severity holds

**Does §11.2's own text argue for R7, or only for R2/R6 as the critic
claims?** Re-read §11.2 in full. The route's justification is "reuses:
current estate/workspace discovery; current workflow loader and validation;
root publication boundary; current embedded fallback; existing Axum and
`ApiClient` patterns" — every clause is a reuse-of-existing-capability
argument (R2) or a "thin new route over existing pieces" argument (R6).
Nothing in the section's prose names a requirement that fails at R6 and
forces R7. Confirmed as claimed.

**Does the register's other R7 entry actually demonstrate the convention
being followed correctly, making T2-40's gap a real inconsistency and not
just a missing citation the register never required?** Re-read T2-31 (§8.7)
directly. Its register row is "R7 | Prefer narrowly wrapped
`ratatui-textarea`, dependency-tree gated" and its body lists five explicit
failed-lower-rung lines ("R1 fails... R2 fails... R3/R4 fail... R5 fails...
R6 fails..."), each naming what specifically doesn't work. This is the same
document demonstrating the convention two sections before T2-40 fails to
follow it — not an external standard being imported unfairly.

**Does `reference/notes/ideaos-agent-contract.md` actually bind this, or is
it aspirational prose?** Read directly: "An `R7` entry must name which lower
rungs were checked and why they failed. Critics on the simplicity axis grade
rung-skipping as a finding." `docs/DEVELOPMENT.md:88` independently commits
the repository to it: "Design decisions log their Ponytail rung." And the
proposal's own register closes with "Every new R7 names failed lower rungs"
(§22, immediately under the table) — a promise the document makes about
itself and then breaks for exactly this one entry. This is a self-referential
binding failure, not a critic-invented rule.

**Is there a substantive case that R7 is actually deserved here (a new
authenticated HTTP route, response schema, and Axum handler is arguably more
than "one line" of composition), which would mean the *tag* is right even
though the *justification* is missing — and does that change anything?** I
considered this. It is plausible that R6 genuinely fails here (a brand-new
route registration plus a versioned response contract is more machinery than
a "tiny local composition or extraction," R6's own definition per §5.8). But
this doesn't rescue the entry: the convention's failure mode is specifically
an *unjustified* R7 — a correct tag with no stated reasoning is still a
violation of "must name which lower rungs were checked and why they failed."
Whether R7 is ultimately the right rung or not is exactly the question the
missing sentence would answer; its absence is the defect regardless of which
way that question resolves.

**Does the proposal already address this elsewhere?** No — checked §11
(all subsections), §19.4 (workflow catalog tests), and §22's surrounding
rows; none supply the missing failed-rung narrative for T2-40.

**Non-goals / style check.** Not a style preference — the rung-logging
convention is cited as binding by both the contract and the document's own
closing line. Not designing the implementation — the fix is an additive
sentence, and (per the critic's own framing) dropping the R7 tag entirely is
an equally valid fix that changes nothing about the route.

**Severity.** Warning is correct: the endpoint's design is not in question,
and nothing about §11.2's substance depends on which rung it's filed under.
No basis to upgrade to error (no test breaks, no artifact is wrong) or
downgrade to info (this is a concrete violation of a convention the document
and `docs/DEVELOPMENT.md` both bind themselves to, not a cosmetic
preference).

**Verdict: CONFIRMED. Severity unchanged (warning).**

---

## Finding 3: `inv-r-ns-6-restatement-incomplete` — CONFIRMED, info severity holds

**Does §5.6 actually drop the conditional/per-transport clause and the
WORKFLOW-IF-E3 consequence, or does the critic overstate the omission?**
Re-read `NORTH-STAR.md`'s R-NS-6 verbatim: "sgt owns message *mechanics* to
a running execution (`needs_input`/`respond`, journaled); the harness owns
the *conversation*. Nothing conversational is ever engine work; whether a
transport's actor can ask mid-run is a measured per-transport capability
with runtime withdrawal, never new hold machinery. Consequence: the
WORKFLOW-IF-E3 category is empty — grilling-class packages are operator
skills." §5.6's text: "R-NS-6 distinguishes execution mechanics from the
harness-owned conversation. `respond` answers a parked request. It is not a
generic message operation." Confirmed: §5.6 keeps only the first sentence's
substance and drops the measured/per-transport/runtime-withdrawable
qualifier and the named WORKFLOW-IF-E3 consequence entirely. The critic's
characterization is accurate, not exaggerated.

**Does this omission actually matter for anything T-Series does, or is it
purely cosmetic — and does that bear on severity?** Checked §6.2's
non-goals list directly: "arbitrary active-Work guidance... continuous
interactive harness sessions... embedded PTY or harness process
supervision" are excluded regardless of any per-transport capability
question. I traced whether any other T-series decision (T2-17, T2-23's
Attention derivation, the composer/respond decisions in §15) depends on the
dropped conditional clause for its own correctness, and found none — every
one of them holds under the *narrower* reading §5.6 states, and none
requires the *broader* conditional the full R-NS-6 text actually rules.
This matches the critic's own conclusion and I found nothing to contradict
it.

**Does the proposal already restate the fuller ruling elsewhere, making this
a non-issue?** Grepped the document for "WORKFLOW-IF-E3", "grilling", and
"per-transport" independent of the critic's search — none appear anywhere
outside this single citation gap. Confirmed as a genuine, uncorrected
omission, not a cross-reference the critic missed.

**Non-goals / style check.** This is squarely what the contract asked for —
"§5.6... is the proposal's own restatement of it, grade the restatement,
don't accept it" — not scope creep, and not a demand to add features; the
fix is restoring two clauses of quoted ruling, costing nothing.

**Severity.** Info is right. The critic's own "does the section survive"
analysis — restoring the clauses "costs nothing and changes no T-Series
decision" — held up under my independent trace of every decision that cites
§5.6. No basis to upgrade: no invariant is actually violated by T-Series'
own scope, and no test or acceptance item depends on the missing text. No
basis to downgrade to nothing: the citation is measurably incomplete against
a named ruling this contract specifically asked the panel to check rather
than accept, and future readers of this proposal as a citation source would
be misled about R-NS-6's actual conditionality.

**Verdict: CONFIRMED. Severity unchanged (info).**

---

## Summary

| Finding | Verdict | Severity |
|---|---|---|
| `inv-estate-doctor-bypasses-client-boundary` | CONFIRMED | error (unchanged) |
| `inv-t2-40-unjustified-r7` | CONFIRMED | warning (unchanged) |
| `inv-r-ns-6-restatement-incomplete` | CONFIRMED | info (unchanged) |

All three findings survive adversarial refutation. I attempted to knock each
one down on its factual claims (re-read `tests/m6_surfaces.rs` t5/t5b in
full rather than trusting line citations, re-read ADR 0009 and
`docs/DEVELOPMENT.md`'s clients-are-equal invariant directly, re-read
`NORTH-STAR.md`'s R-NS-6 and `reference/notes/ideaos-agent-contract.md`'s
rung convention verbatim, grepped the full proposal independently for every
citation the critic claims is absent, and checked `src/cli.rs`/`src/api.rs`
directly for whether repo/group/Doctor are in fact daemon-independent
today), on an alternative design-precedent angle that could have softened
Finding 1 (the 2026-08-11 manifest-design note's "TUI later = the same
verbs" — found to corroborate rather than excuse the gap, since it predates
the hardened test the finding turns on), on scope (checked each against all
four of this unit's non-goals), on whether a substantively-justified R7 tag
would excuse Finding 2's missing narrative (it would not — the convention's
failure mode is the missing reasoning, independent of which rung is
eventually right), and on severity in both directions for all three (traced
concrete enactment failure for Finding 1's error tag; traced whether any
T-Series decision depends on Finding 3's dropped clause to check info
against inflation). None fell.
