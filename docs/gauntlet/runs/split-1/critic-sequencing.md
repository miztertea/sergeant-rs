# SPLIT-1 — Critic: sequencing (seat 4 of 4)

Axis: do the gates hold, and is the authority split honest? Grading
`reference/proposal-product-workspace-split.md` §6, §7, §8, §9 against
`.sergeant/common/contexts/bounded-judgment.md` and `docs/adr/0014-product-workspace-split-owner-rulings.md`
(the fidelity authority — used here only to check whether §9's claimed
authority actually exists there, not to grade the ADR itself).

---

## Finding 1 — Phase 2 depends on a decision the gate table places after Phase 2

**Severity: error**

**§6, Phase 2:** "Add edition markers (§4.7)." **§4.7:** "Each shipped
template declares the distro version it was forked from." **§7, Gate G3**
(gates entry to **Phase 3**, i.e. fires *after* Phase 2 is done):
"Template triage complete; edition-marker format settled." **§11, Open
decision 2:** "Edition-marker format — frontmatter field, or a generated
manifest? Affects Phase 2 and `sgt doctor`."

Phase 2's own task list requires templates to be stamped with an edition
marker. But the proposal's own gate table places "edition-marker format
settled" at G3 — the gate that fires only once Phase 2 is already
complete — and §11 confirms the format is still an open question, not a
settled one. Phase 2 cannot execute "add edition markers" without first
knowing the marker's shape (a frontmatter field parses and diffs
differently than a generated manifest; `sgt doctor`'s drift detector in
Phase 3 has to agree with whatever Phase 2 wrote). The dependency the
gate table encodes is backwards: the decision Phase 2 needs is scheduled
to be settled by the gate that comes after it.

**Concrete failure:** Phase 2 runs, picks a marker shape by default (say,
a frontmatter field) to unblock itself, stamps all 216 templates. G3
convenes afterward and the owner rules a generated manifest instead
(rejecting frontmatter, e.g. because manifest-based markers were the
actual answer to open decision 2). Every template stamped in Phase 2 now
needs a second pass, and Phase 3's `sgt doctor` drift detector — built
against whichever shape actually got ruled — was designed against
templates that don't yet reflect it.

---

## Finding 2 — Phase 5's structural checks are bundled with the one check that actually needs Phase 4

**Severity: warning**

**§6, Phase 5:** lists seven "Structural checks" (template `index.md`
parses, `.sergeant/index.md` catalog matches directory, every `@@ref`
resolves, bounded-judgment coverage, no reachable reference to
`.sergeant/drafts/`, `AGENTS.md` routing rows resolve, no shipped
template references non-shipping content) and then, separately, one
"Cross-repo check... which only the workspace can run" (the co-version
skew check). Both are gated identically: "Phase 5 — *Depends on: Phase 4
(needs both repos mounted in one estate).*"

Only the cross-repo check genuinely needs Phase 4 — it diffs doctrine
claims against the built binary's `--help` surface, which requires the
workspace's cross-repo vantage. The other seven checks are statements
about `sergeant-rs` alone (does a template parse, does an `@@ref`
resolve, does an index count match a directory) and need nothing from
`sergeant-rs-workspace` existing. ADR 0014's own consequences section
names the reason Phase 5 is gated on Phase 4 for the *skew* check
specifically — "a stock-template change cannot be exercised by the
estate that ships it until released. Pointing the estate at a working
tree is a legitimate testing posture and a corrosive default" — but that
argument doesn't extend to structural checks that never touch the
built binary.

**Concrete failure:** a dangling `@@ref` or a template that still cites
`reference/sergeant-upstream/` survives Phase 2 triage undetected (the
162-candidate list is itself conceded to be an upper bound needing
triage — §2.2). Under the stated order, nothing catches it until Phase
5, which is gated on Phase 4 having already run: `sergeant-rs` has by
then shed `docs/gauntlet/`, `reference/`, `reference-corpus/`,
`resources/` (Phase 4's own task list). Fixing the missed contamination
now requires reaching into evidence that has already moved to the other
repository, instead of a same-repo fix available any time after Phase 2.
The gate table pays for a bundling choice it never argues for.

---

## Finding 3 — §9 grants execution-level J2 authority that ADR 0014 never rules on

**Severity: error**

**§9:** "**Authorized** | Phase 1 `AGENTS.md` rewrite + `DEVELOPMENT.md`
absorption, as a PR for morning review | **J2** — delegated drafting;
owner approves."

Per `bounded-judgment.md`, J2 requires "the active skill or stage
explicitly delegates this class of decision within named bounds... 'Use
your best judgment' without a bounded decision class named is not a J2
grant — the package must name the delegation." Checking ADR 0014 for
that delegation: decision 10 rules on *what* `AGENTS.md` should contain
(both ladders inline), and the Consequences section states *that*
"`AGENTS.md` loses its 'Standard workflow loop' section." Both are
content-level rulings the owner already made — squarely J4 material
("explicit user or bound Work decision... apply the recorded decision
without asking the user to reconfirm it"), not J2.

What ADR 0014 does **not** rule on anywhere is *who executes the rewrite
and under what supervision* — specifically, whether an unattended
overnight run may independently draft, and open a PR for, a full rewrite
of the single always-on doctrine file, with the only checkpoint being a
morning review of the finished artifact. §9 is the only place this
authorization appears, and §9 is part of the proposal under review here
— written by the same author (Captain) who would be the overnight actor
exercising it. That is the exact pattern §4.6 names as the failure mode
to prevent: "whoever defines 'did it work' must not be whoever did it."
Labeling execution-level, self-granted overnight authority as "J2 —
delegated drafting" borrows the *legitimacy* of ADR 0014's content-level
J4 ruling to cover a scope ADR 0014 never actually delegated.

**Concrete failure:** the overnight run treats "J2 — delegated drafting;
owner approves" as license to also resolve judgment calls ADR 0014
doesn't cover (e.g., how CAN/SHOULD prose is worded, where the routing
table pointer to `DEVELOPMENT.md` lands, exactly which guardrail
sentences count as "Layer-1 mechanics"). None of that drifted content is
flagged as new authority in the PR — it reads as implementing an
already-owner-decided spec, because §9 called it J2 under a ruling that,
on inspection, only covers the *shape*, not the *unattended execution*.

---

## Finding 4 — three of seven gates are green-lit by the party performing the work behind them

**Severity: error**

**§7:**

| Gate | Owner column | Who actually does the gated work |
|---|---|---|
| G2 | Captain | Phase 1 (§9: Captain drafts, J2) and Phase 2 (§9: Captain triages, J2) |
| G5 | Captain | Phase 4 (dispatched Work, presumably Captain-executed; no owner task for it appears in §8's manual checklist beyond creating the repo) |
| G6 | Captain | Phase 5 (builds Gate F itself) and Phase 6 pipeline work |

G2's condition is "Phase 1 merged; no BU citation resolves to content
that will not ship." The merge itself requires the owner (§9: "Not
authorized: Merging any PR to `main` — J0"), but the substantive
judgment — whether the 611 BU citations and 91 `reference/sergeant-upstream/`
references have actually all been resolved — is certified by Captain,
the same party whose Phase 2 triage produced the resolution in the first
place. G6 is sharper still: Captain builds Gate F (Phase 5), then G6
asks whether "Gate F green on a real commit" — Captain is both author of
the check and the one who declares it satisfied, with no independent
party named to confirm the validator itself is sufficient before Phase 6
(release pipeline construction) proceeds.

Contrast with G0, G1, G3, G4, G7, all owned by "Owner" — a party external
to the work being gated. G2/G5/G6 do not have that separation; the
"Owner" column names Captain, collapsing the worker/gatekeeper split the
proposal's own §4.6 says the plan needs ("whoever defines 'did it work'
must not be whoever did it").

**Concrete failure:** Captain's Phase 2 triage misclassifies a handful of
the 162 contamination candidates as clean (plausible — §2.2 concedes the
162 is an upper bound needing sampling, and sampling error is exactly
the kind of mistake a self-check won't catch). G2 fires anyway, because
the party judging "no BU citation resolves to non-shipping content" is
the same party whose classification produced that answer. Phase 3
proceeds to embed the still-contaminated templates into the binary.

---

## Finding 5 — the PACE "Alternate" branch authorizes exactly the guess the ladder forbids

**Severity: error**

**§9:** "**Alternate** — where a decision is J2-adjacent but unclear,
take the conservative option and record it in the PR body."

`bounded-judgment.md`'s J0 rung: "No higher rung resolves the question,
evidence conflicts, authority is missing... **Do not guess.**" For a
workflow stage, J0 requires ending the turn with a direct question, not
proceeding. The Alternate branch describes precisely a case where no
rung cleanly resolves the question ("J2-adjacent but unclear") and
instructs the actor to proceed anyway, selecting what it judges to be
the "conservative option" unilaterally. That is a guess, dressed as
caution, made by the same actor the ladder says must not guess in this
situation. Recording the choice in the PR body is decision evidence
after the fact, not the "explicit, reviewed specification change" §4.6's
tripwire requires before a rung is lowered — the PR sits unreviewed until
morning, by which point the judgment call is already embedded in a
rewrite of the doctrine file itself.

**Concrete failure:** mid-rewrite, the overnight actor hits an ambiguous
sentence in current `AGENTS.md`'s Guardrails section that plausibly reads
as either Layer-1 CLI mechanics (delete, per §4.1) or Layer-2 doctrine
(keep). Genuinely unclear — by the Alternate branch's own description.
It picks what it judges "conservative" and deletes it, noting the choice
in the PR body. If that judgment is wrong — the sentence was doctrine,
not mechanics — the PR silently ships a scope decision no one but Captain
made, inside the file whose entire purpose is to bound Captain's own
future authority. A tired morning review of a large diff is exactly the
condition under which that kind of loss goes unnoticed.

---

## Finding 6 — §9 never names itself as the tripwire event §4.6 requires

**Severity: warning**

**§4.6:** "**Tripwire:** any rung-lowering (J0 → J2) is an explicit,
reviewed specification change. Never emergent drift." §9 assigns J2 to
several classes of overnight work (Phase 1 drafting, Phase 2 triage,
NORTH-STAR draft) but nowhere states that these assignments *are* the
rung-lowering event the tripwire is meant to gate, or that they are
submitted here specifically for the review the tripwire requires. The
table reads as already-settled fact ("J2 — delegated drafting; owner
approves"), not as a flagged specification change awaiting the review
that would make it compliant with the proposal's own rule. The SPLIT-1
gauntlet this document is now undergoing can supply that review in
practice, but the proposal text itself does not point to that mechanism
or otherwise acknowledge that §9 is the tripwire firing — the compliance
is accidental (this critic's assignment exists), not designed into the
document.

---

## Finding 7 — §9 assumes ladder access that Phase 1 itself is what creates

**Severity: note**

ADR 0014, decision 10, diagnoses the current state: "`@@bounded-judgment`
resolves only inside an active stage's `CONTEXT.md` — so a direct
in-session Captain, the mode most prone to over-escalation, had no
ladder at all." §9's overnight scope is exactly a direct-session Captain
run (not a workflow stage — there is no stage `CONTEXT.md` cited for
"overnight captain scope"), operating against a rung table (J4/J2/J0) it
is expected to apply correctly across six phases of work. The
infrastructure that would give a direct-session Captain inline,
reachable ladder access is Phase 1's own deliverable (§4.3: "both ladders
ship inline in `AGENTS.md`... reachable without a workflow stage
active"). The overnight run that is supposed to safely execute Phase 1
is, by ADR 0014's own diagnosis, the exact mode that currently lacks the
ladder Phase 1 is meant to install. This is not necessarily disqualifying
— the pre-rewrite `AGENTS.md` plus this proposal's own §9 table may be
enough scaffolding for one run — but the proposal does not acknowledge
the bootstrap order or explain why it's sufficient.

---

## Summary

Two dependency errors in §6/§7 (Findings 1–2), two authority-split
violations in §9 (Findings 3, 5), a structural gate-ownership conflict
across three gates (Finding 4), and two lower-severity gaps in the
proposal's compliance with its own anti-capture rule (Findings 6–7). The
recurring pattern: where the proposal's authority language is checked
against what ADR 0014 actually rules versus what §9 additionally
asserts, §9 consistently claims more delegated authority than the cited
ruling grants, and the gates meant to catch that are, in three of seven
cases, owned by the same party the gate is supposed to constrain.
