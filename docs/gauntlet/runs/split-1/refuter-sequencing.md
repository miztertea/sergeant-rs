# SPLIT-1 — Refuter: sequencing

Refuting `docs/gauntlet/runs/split-1/critic-sequencing.md` against
`reference/proposal-product-workspace-split.md` (§6, §7, §9), ADR 0014,
and `.sergeant/common/contexts/bounded-judgment.md`. Fresh context; I have
not read the other three critic files or any other refuter's output.
Default posture: refuted unless I could not break the claim.

Commands run to verify structure and cross-checks:

```
$ grep -n "^#" reference/proposal-product-workspace-split.md
```
confirms §6 (Work groupings, line 333), §7 (Gates, line 420), §9
(Overnight captain scope, line 459), §11 (Open decisions, line 511) exist
where cited.

```
$ grep -n "edition" docs/adr/0014-product-workspace-split-owner-rulings.md reference/proposal-product-workspace-split.md
$ grep -ni "overnight\|unattended\|morning" docs/adr/0014-product-workspace-split-owner-rulings.md reference/proposal-product-workspace-split.md
```
confirms ADR 0014 never mentions "overnight" or "unattended" at all, and
lists "Edition-marker format" as an open question, not a ruling.

---

## Finding 1 — Phase 2 depends on a decision the gate table places after Phase 2

**Verdict: SURVIVES**

Checked directly: §6 Phase 2 lists "Add edition markers (§4.7)" as a task.
§7's gate table places "edition-marker format settled" at **G3**, which
gates entry to **Phase 3** — i.e. it fires only after Phase 2 is declared
complete. **G2** (the gate that actually gates entry to Phase 2) reads only
"Phase 1 merged; no BU citation resolves to content that will not ship" —
no edition-marker condition. §11 open decision 2 confirms the format is
unresolved as of the proposal's own text ("frontmatter field, or a
generated manifest? Affects Phase 2"). ADR 0014's own "Open questions"
section repeats the same open item verbatim, so this isn't a case where
the critic missed a settled owner ruling elsewhere — I checked, and there
is none.

I tried to break this on materiality grounds ("a second pass over 216
files is cheap, so who cares") but the finding's claim is about the
sequencing logic being backwards, not about the cost of recovery, and the
contract's sequencing axis explicitly asks "Can Phase 2 genuinely precede
Phase 3... or does decontamination need the embedding format settled
first?" — this is exactly responsive to that, and the inconsistency is
real: Phase 2's own task list requires a decision the gate table schedules
to be settled later. Finding survives.

---

## Finding 2 — Phase 5's structural checks are bundled with the one check that actually needs Phase 4

**Verdict: SURVIVES**

Checked: §6 Phase 5 lists seven structural checks (index.md parses, index
catalog match, `@@ref` resolution, bounded-judgment coverage, no
`.sergeant/drafts/` leakage, routing-table resolution, no shipped
reference to non-shipping content) plus one cross-repo skew check, both
gated identically on Phase 4 in §7 ("Depends on: Phase 4 (needs both
repos mounted in one estate)").

I looked for a design-level counter-argument the critic might have missed:
§3.1 does say `.sergeant/local/workflows/` in the workspace holds
"workspace-local validators," which could be read as an architectural
decision that *all* validators, not just the cross-repo one, live in the
workspace by design. That's a real consideration the critic didn't cite.
But it doesn't fully refute the finding — §3.1 explains *where validator
code is stored*, not why the seven purely-`sergeant-rs`-scoped checks
(parsing, ref resolution, index counts) must wait for two repos to be
mounted before they can run even once. Nothing in §3.1–§3.4 argues that
these specific checks need workspace vantage; the only argument offered
anywhere (ADR 0014 consequences: "a stock-template change cannot be
exercised by the estate that ships it until released") is scoped
explicitly to the skew check, as the critic already noted. The contract's
sequencing axis directly asks "Is Phase 5's cross-repo skew check actually
blocked on Phase 4, or could it run earlier against a working tree?" — the
finding answers that question correctly for the bundled checks. The
concrete failure (contamination missed in Phase 2 triage isn't caught
until after Phase 4 has already moved the evidence) is plausible given
Phase 2's own 162-candidate list is conceded as an upper bound needing
triage (§2.2, referenced accurately). Finding survives, with the caveat
that §3.1 slightly weakens the "never argues for" framing — I'm noting
that as a partial correction, not a refutation.

---

## Finding 3 — §9 grants execution-level J2 authority that ADR 0014 never rules on

**Verdict: REFUTED**

The critic is right that ADR 0014 decision 10 and its Consequences section
settle *content* (both ladders ship inline; "Standard workflow loop"
section is removed) and never mentions unattended or overnight execution
at all (confirmed by grep — zero hits in the ADR). But the finding
conflates two different questions: what content changes (J4, already
settled by ADR 0014) and what judgment is needed to draft that content
(properly J2 — "Inspect evidence, choose, and record the rationale," per
`bounded-judgment.md`'s own definition of delegated actor judgment). §9's
label is "J2 — delegated drafting; **owner approves**" — the label itself
names the exact safeguard the critic says is missing: nothing merges
without the owner. §9 separately and explicitly lists "Merging any PR to
`main`" as **not authorized, J0**. So the worker/gatekeeper split §4.6
requires is intact: Captain drafts and proposes (worker), owner approves
and merges (gatekeeper) — this is precisely "a Work item... may propose an
`AGENTS.md` change; it may never be the sole approver," which §4.6 states
as the rule, satisfied here by construction.

The critic's sharper point — that *timing* (overnight, unattended, only
reviewed in the morning) is itself a distinct authorization ADR 0014 never
granted — doesn't hold up under the ladder's own governance test. The
ladder governs "material decisions: choices that affect scope, acceptance,
user-visible behavior, security, privacy, authority, destructive action,
irreversible state, promoted artifacts." A drafted-but-unmerged PR changes
none of these — it is not irreversible, not promoted, and not itself
public behavior; the owner's merge decision is what would be. Whether the
draft was produced at 2am or 2pm doesn't change what's being delegated
(drafting judgment within already-decided content bounds) or what gate
sits after it (owner merge, J0). I could not find text in §9, ADR 0014, or
`bounded-judgment.md` that treats "unattended" as its own governance
category requiring separate citation. The finding's concrete failure
(drift in wording/placement choices during drafting) is a real risk, but
it's the ordinary risk of *any* J2 delegation, not a risk specific to
overnight execution — so it doesn't establish that J2 was the wrong label,
only that delegated drafting always carries some risk, which the ladder
already accepts by design. Refuted.

---

## Finding 4 — three of seven gates are green-lit by the party performing the work behind them

**Verdict: SURVIVES**

Directly checked the §7 table: G2, G5, G6 list "Captain" in the Owner
column; G0, G1, G3, G4, G7 list "Owner." This is a plain textual fact, not
an inference.

I tried to refute this by arguing the gate-Owner column might denote mere
mechanical execution rather than discretionary sign-off (e.g., if G2's
condition were computed/asserted rather than judged). It isn't: G2's
condition is "no BU citation resolves to content that will not ship" —
this requires the same subjective triage judgment Phase 2 itself performs
(§2.2 concedes the 162-candidate contamination list is an upper bound
needing sampling, i.e., triage is judgment-laden, not mechanical). G6's
condition, "Gate F green on a real commit," is Captain both building the
validator (Phase 5) and declaring it satisfied, with no named independent
check on the validator's own sufficiency before Phase 6 begins. I also
checked G5 specifically, since its condition ("Split merged; both repos
mounted in one estate") is comparatively closer to a factual/mechanical
check than G2 or G6 — this is the weakest of the three instances the
critic groups together, and the finding's prose only substantiates G2 and
G6 concretely, gesturing at G5 more thinly. That weakens the finding's
scope slightly but doesn't break its core claim, which survives on G2 and
G6 alone: two gates where the party whose own prior-phase work is being
certified is also the certifier, which is exactly the failure mode named
in the proposal's own §4.6 ("whoever defines 'did it work' must not be
whoever did it"). The contract's sequencing axis asks this directly
("§7's gates: is any gate green-able by the same party it is meant to
constrain?") and the answer, checked against the actual table, is yes for
at least two of the seven. Finding survives.

---

## Finding 5 — the PACE "Alternate" branch authorizes exactly the guess the ladder forbids

**Verdict: SURVIVES**

Checked §9's PACE block verbatim: "**Alternate** — where a decision is
J2-adjacent but unclear, take the conservative option and record it in the
PR body." Checked `bounded-judgment.md`'s J0 definition: "No higher rung
resolves the question, evidence conflicts, authority is missing... Do not
guess," and for a workflow stage, J0 requires ending the turn with a
direct question rather than proceeding (there is no live user to ask
overnight, so the "Captain skill" branch — "ask the question live and wait
for the user's answer" — is also unavailable, which the critic doesn't
raise but strengthens the point independently).

I tried the most charitable refutation: that "J2-adjacent but unclear"
just means ordinary delegated judgment within an already-bounded class,
which J2 itself licenses ("Inspect evidence, choose, and record the
rationale and rung"). That reading is plausible but the phrase's own
wording cuts against it — "unclear" whether a decision is even
"J2-adjacent" is a statement about rung applicability itself being in
doubt, which is precisely the condition `bounded-judgment.md` assigns to
J0 ("no higher rung resolves the question"), not to J2 ("this class of
decision" must already be named and bounded — an unclear case is by
definition not cleanly inside a named bound). I also checked whether
AGENTS.md content could be read as "local, reversible" (J1, sidestepping
the whole question) — it can't: AGENTS.md is the single always-on file
governing all future agent behavior project-wide, squarely "public
behavior" and "downstream stage's interpretation" under the ladder's own
materiality test, so J1's "local, non-contractual" carve-out doesn't
apply. The critic's own concrete failure scenario (an ambiguous Guardrails
sentence, picked as "conservative," deleted, silently drifting scope
inside the doctrine file itself) is realistic and matches exactly the kind
of case J0 is designed to catch and this branch is designed to bypass.
Finding survives.

---

## Finding 6 — §9 never names itself as the tripwire event §4.6 requires

**Verdict: REFUTED (immaterial)**

Checked: §9 has no sentence stating that its own J2 assignments are the
"explicit, reviewed specification change" §4.6's tripwire requires. That
textual absence is accurate.

But the finding fails the materiality test. §4.6's tripwire requires that
a rung-lowering be reviewed, not that the reviewed document narrate its
own review process inline. The review the tripwire calls for is actually
happening — this is precisely what the SPLIT-1 gauntlet (this file
included) is doing to §9 right now, per the SPLIT-1 contract's own
framing ("Captain wrote the artifact under review... a critic that finds
Captain's own reasoning congenial has probably not done the job"). Whether
or not §9's prose contains a self-referential pointer to that mechanism
changes nothing about whether the review occurs or what its outcome is;
the adjudication step downstream is unaffected by this omission either
way. The critic's own text concedes as much ("The SPLIT-1 gauntlet this
document is now undergoing can supply that review in practice"), which is
the concession that breaks the finding's own claim to consequence. A
documentation nicety whose absence doesn't change what happens next is
refuted on materiality grounds per the refutation criteria.

---

## Finding 7 — §9 assumes ladder access that Phase 1 itself is what creates

**Verdict: REFUTED (immaterial)**

Checked: ADR 0014 decision 10 does say "`@@bounded-judgment` resolves only
inside an active stage's `CONTEXT.md` — so a direct in-session Captain...
had no ladder at all," and §9 is indeed a direct-session scope with no
cited stage `CONTEXT.md`. That much is accurate.

But "no ladder at all" describes the *automatic macro-resolution* path
only. `.sergeant/common/contexts/bounded-judgment.md` is a plain file in
the repository, directly readable by any session — including a
direct-session Captain — without `@@bounded-judgment` auto-expansion or an
active workflow stage; I read it directly myself in the course of this
refutation with a single Read call, which is exactly the access pattern an
overnight Captain run would also have. The "bootstrap hazard" the critic
describes is a discoverability/ergonomics gap (the ladder isn't served to
you automatically), not a hard block on access (the ladder is one file
read away). The critic already hedges this to "note" severity and writes
"not necessarily disqualifying... the pre-rewrite `AGENTS.md` plus this
proposal's own §9 table may be enough scaffolding for one run" — which
concedes the practical risk is low. Given the underlying access gap is
easily closed by the actor simply reading the canonical file (something
any competent overnight run would do, and something this refutation itself
demonstrates is trivial), and the critic's own hedge undercuts the claim's
consequence, this is refuted on materiality grounds: it flags a real but
non-blocking asymmetry that would change nothing about whether the
overnight run in fact has ladder access.

---

## Summary

| # | Verdict |
|---|---|
| 1 | SURVIVES |
| 2 | SURVIVES |
| 3 | REFUTED |
| 4 | SURVIVES |
| 5 | SURVIVES |
| 6 | REFUTED (immaterial) |
| 7 | REFUTED (immaterial) |

Four findings survive independent adversarial verification (1, 2, 4, 5) —
all four are direct, checkable textual/logical inconsistencies in §6, §7,
and §9 that I could reproduce by reading the cited sections myself, not
inferences that depend on taking the critic's framing on faith. Three are
refuted: Finding 3 conflates a settled content decision (properly J4) with
ordinary delegated drafting judgment (properly J2), and the safeguard it
says is missing (owner-gated merge) is explicitly present in the same
sentence it's criticizing; Findings 6 and 7 identify real but non-material
gaps that the critic's own text substantially concedes don't change what
happens next.

One general observation on the contract's framing, per this unit's own
instruction to flag it: several surviving findings (1, 2, 4, 5) are direct
answers to questions the sequencing axis explicitly poses verbatim in
`docs/gauntlet/contracts/SPLIT-1.md` ("Can Phase 2 genuinely precede Phase
3," "is any gate green-able by the same party it is meant to constrain,"
etc.). That the contract primed the critic to look in these exact places
doesn't make the findings less true — I verified each one independently
against the actual table and task-list text, not against the critic's
say-so — but it does mean the sequencing axis's coverage is narrower than
it might look: it found what it was pointed at. Given Captain wrote both
the contract and the proposal, that the pointed-at spots turned out to
contain real inconsistencies is worth the adjudicator's attention on its
own terms, separate from whether any individual finding survives.