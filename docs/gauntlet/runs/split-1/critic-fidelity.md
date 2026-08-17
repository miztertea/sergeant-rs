# SPLIT-1 — Critic: fidelity

Blind critic seat 1 of 4. Axis: **fidelity** — does the proposal say what was
decided, and only what was decided? Artifact graded:
`reference/proposal-product-workspace-split.md`, §1–§13. Fidelity authority:
`docs/adr/0014-product-workspace-split-owner-rulings.md`.

Method: read the proposal in full; read ADR 0014 in full; grepped the
proposal text for `Notion`, `Gmail`, `cerberus`, `three repositor`, `invert`,
`workflow diff`, `PACE`, `succession` to check the four named corrections and
trace every §4/§6/§13 item back to an ADR 0014 decision, an earlier ADR, or
named cited evidence. Did not read any other file under
`docs/gauntlet/runs/split-1/`.

---

## Findings

### 1. [error] The Notion correction is entirely absent from the proposal

**Where:** whole document — `grep -i notion reference/proposal-product-workspace-split.md` returns nothing.

**What it should trace to and doesn't:** ADR 0014's "Alternatives considered"
records, in the owner's own words, that Captain proposed **Notion as the
project's knowledge system of record** and was rejected ("that's my personal
idea store"), independently reinforced by the owner's own Notion decision
page and `reference/notes/ideaos-agent-contract.md`. The SPLIT-1 contract
names this as the first of four corrections the panel must verify survives.

The proposal's §3.2 ("Three substrates in the workspace") and §3.3 (scoped
retrieval) design the entire knowledge layer that Notion was rejected in
favor of, yet never once states that Notion was considered and rejected, or
why. A reader of the proposal alone has no way to know this alternative was
ever on the table.

**Why it matters:** this is the single most explicit of the four named
corrections — the owner's own words are preserved in ADR 0014 for exactly
this purpose — and it does not appear anywhere in the document that is
supposed to carry it forward. Silence is a form of laundering: a design that
was contested reads as though it was the obvious and only answer.

---

### 2. [error] The three-repository correction and the dependency inversion are entirely absent

**Where:** §3.1 "Two repositories" (lines 127–142) — presents the two-repo
topology as settled architecture with no history.

**What it should trace to and doesn't:** ADR 0014 decision 5 records: *"Captain
initially proposed three repositories (product, estate, record) and an
extraction of the distro out of the dev repo. The owner corrected both: two
repositories, and the dev estate consumes the released distro rather than a
build-tree approximation of it."* This is the second corrected of the four
named corrections.

§3.1 states the final topology cleanly — `sergeant-rs` as product,
`sergeant-rs-workspace` as estate mounting it — with no acknowledgment that
Captain's own first answer was a third repository and an inverted
dependency (distro extracted *out of* the dev repo rather than the estate
*mounting* the released product). The "Alternatives considered" pattern that
ADR 0014 itself uses to preserve this history has no counterpart in the
proposal.

**Why it matters:** the direction of the mount (workspace consumes the
*released* distro, not a build-tree copy) is a load-bearing constraint for
§3.4's "trap to avoid" and the bootstrap hazard named in ADR 0014's
consequences. Presenting it as the only design ever considered, rather than
a corrected one, obscures exactly the failure mode (retrieval fog around
*why* a decision was made) that §2 of the proposal claims to diagnose.

---

### 3. [error] The Gmail/inbox-convention correction is entirely absent — the proposal's own self-diagnosis omits its most self-implicating instance

**Where:** §2.1 "Six Memory Failure Modes" (lines 72–85), specifically the
"Arrival gap" and "Instruction fiction" rows, and §1's "Relationship to
existing decisions" table (lines 13–29) where the inbox proposal's
surfacing "via the inbox convention" is mentioned without any account of
how that convention was nearly missed.

**What it should trace to and doesn't:** the SPLIT-1 contract names, as the
fourth correction, that Captain **reached for Gmail** when told a proposal
was "in the inbox," having not read `docs/environments/cerberus.md`'s inbox
convention — explicitly called out as *"an instance of the very retrieval
failure the proposal diagnoses."*

§2.1's "Arrival gap" row cites exactly three incidents (LESSONS L20, the
`pkill` double-kill, the CI proposal's missed gate) — the same three the
SPLIT-1 contract's *assumptions* axis independently asks the panel to
verify. The Gmail incident is a fourth, more recent instance of the same
failure mode, involving the very document being written, and it is not
included anywhere in the diagnosis.

**Why it matters:** this is the sharpest test the fidelity axis sets —
"a proposal that reads as though Captain was right all along has been
laundered" — and here the omission is not of a rejected *design* but of a
rejected *diagnostic input*, curated out of a self-assessment that exists to
establish credibility for the rest of the document. A diagnosis section that
cites three failures of this type but silently excludes the fourth, more
recent, more self-incriminating one is not neutral measurement; it is
selective evidence.

---

### 4. [warning] The `sgt workflow diff` correction survives in outcome but not in narrative

**Where:** §4.7 (lines 303–308), §6 Phase 3, §12 non-goals (line 529), §13
decision register ("Fork mechanism").

**What it should trace to and doesn't:** ADR 0014 decision 4 records a
two-step history: *"Captain proposed a drift-diff verb, then withdrew it
under R1 once templates were reframed as examples. The corpus's
counter-argument — a fork has no invalidation mechanism ... — is answered by
an edition marker on each template, not by a verb."* This is the third named
correction: Captain killed the verb, and separately had to be shown that the
underlying property (drift visibility) still needed a mechanism.

§4.7 states the final position — no `workflow diff` verb, an edition marker
instead — using close paraphrase of ADR 0014's own language ("no edition
number, no survey date, no revision program"). That much is faithfully
carried forward. But the proposal presents this as a single, clean piece of
design reasoning; it does not narrate that Captain's own R1-grounds
elimination of the verb was incomplete until the corpus's counter-argument
forced the edition-marker fix. The correction is real but its *shape* — an
initial answer proving insufficient — has been smoothed into a seamless
derivation.

**Why it matters:** lower severity than findings 1–3 because the outcome is
correctly stated and the source language is closely tracked. But the fidelity
axis specifically asks whether "the four recorded corrections ... survive
rather than being smoothed away" — narrative smoothing of *how* a decision
was reached is still a fidelity concern, just a milder one than outright
omission.

---

### 5. [warning] PACE and succession-of-authority are presented as settled doctrine with no traceable source

**Where:** §4.4 (lines 251–260), §9's PACE application (lines 479–483), §13
decision register: *"Authority ladder | J5–J0 retained; PACE and succession
added below J0"* (line 546).

**What it should trace to and doesn't:** ADR 0014's thirteen decisions cover
distro delivery, versioning, templates, the `workflow diff` withdrawal,
repository topology, workspace history, `DEVELOPMENT.md` placement,
NORTH-STAR amendments, the inbox proposal's re-scoping, both existing
ladders shipping inline, knowledge-organization priority, anti-capture
balance, and model policy. None of the thirteen rules on adding new rungs
below J0, and none mentions PACE (Primary/Alternate/Contingency/Emergency)
or a succession-of-authority concept. ADR 0014's context section lists
`O-SMEAC` among the corpus items the owner had Captain read — a framework
where a PACE plan plausibly originates — but the proposal itself never names
this or any other source for §4.4. The connection has to be inferred by a
reader who happens to also hold ADR 0014's context paragraph in mind; the
proposal text does not make it.

**Why it matters:** this is exactly the failure mode the fidelity axis is
built to catch — "invented scope." §4.4 introduces a structural change to
the bounded-judgment ladder (a decision-authority mechanism with real
teeth in §9's overnight scope) and §13 records it in the decision register
in the same unqualified voice as items that trace directly to an explicit
owner ruling. Nothing in the document distinguishes "the owner ruled this"
from "Captain designed this and it awaits ruling." Contrast with how the
proposal correctly flags the *NORTH-STAR* amendment as "requires a dated
owner ruling" (line 23) — §4.4 gets no equivalent flag despite having no
ADR 0014 backing at all.

---

### 6. [note] Several §4 doctrine items are framed as "adopted verbatim" or given unattributed authority-bearing quotes

**Where:** §4.2 (lines 218–222) opens with an unattributed blockquote — *"A
permission system alone cannot produce judgment. Behavioral instructions
alone cannot enforce authority"* — presented as the premise for the
CAN/SHOULD split; §4.5 (line 264) states the failure-attribution taxonomy is
"Adopted verbatim" without naming verbatim from what; §3.4's "bound /
referenced / reachable" context tiering (line 195) is asserted with no
source.

**What it should trace to and doesn't:** ADR 0014's context section lists
several owner-read corpus items that are plausible sources for this
material (Agent Behavior Framework, Work-Centered Intelligence, Work
Filesystem), but the proposal text draws no line from any specific item to
any specific §4 claim. §13's decision register entry "Instruction registers |
CAN and SHOULD written separately" inherits the same gap.

**Why it matters:** lower severity — none of this contradicts ADR 0014, and
the source material plausibly exists among the inputs the owner had Captain
read. But the fidelity axis's standard is that every doctrine ruling in §4
"must trace to ADR 0014, to an earlier ADR, or to cited evidence" — an
unattributed quote presented as premise, or a taxonomy claimed "verbatim"
with no named source, does not meet that bar on the document's own terms,
even where the underlying content is probably legitimate.

---

## Summary

Three of the four named corrections (Notion, three-repositories/inversion,
Gmail/inbox-convention) do not appear in the proposal text at all — not
softened, not reframed, simply absent. The fourth (`workflow diff`) survives
in outcome but loses its corrective shape. Independently, the fidelity axis's
general standard — every §4/§6/§13 item traces to ADR 0014, an earlier ADR,
or cited evidence — turns up one clear invented-scope item (§4.4's PACE and
succession-of-authority addition, carried into the §13 decision register
without qualification) and a handful of unattributed-quote items that are
probably legitimate but are not shown to be.

The pattern across findings 1–3 is consistent: the proposal is faithful about
*what* was decided but silent about *the corrections that produced it*. That
silence is exactly the shape the axis warns against — "a proposal that reads
as though Captain was right all along has been laundered" — even though no
single sentence in the document affirmatively claims Captain was right the
first time. The laundering here is by omission, not by false narration.
