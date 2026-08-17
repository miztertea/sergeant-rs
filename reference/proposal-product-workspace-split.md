# Product / Workspace Split, Doctrine Rewrite, and Release Pipeline

**Status:** Proposed — revised 2026-08-17 after SPLIT-1 (sent back, scoped to §2)
**Date:** 2026-08-17
**Audit basis:** `main` @ 11aa8a1
**Scope:** Repository topology, always-on doctrine, knowledge-layer design, distro
packaging, release engineering
**Product behavior:** Changed — the distro becomes an embedded, co-versioned
artifact; workflow packages become templates rather than published procedure

---

## Relationship to existing decisions

This proposal amends `NORTH-STAR.md` in three places and supersedes part of
`.sergeant/index.md`'s framing. It absorbs, re-sequences, and partially defers
`~/inbox/proposal-ci-cd-release-engineering.md` (2026-08-16), which remains an
inbox item and is **not** accepted wholesale by this document.

| Existing decision | Disposition here |
|---|---|
| NORTH-STAR "Destination" paragraph | **Amended.** "A cloned directory of instructions, skills and conventions" is now false twice: the distro is embedded in the binary and written by `sgt init`, and its contents are templates, not conventions. |
| NORTH-STAR Gated: "stranger onboarding + prebuilt binary" | **Amended.** This proposal implements the prebuilt-binary half. Requires a dated owner ruling per NORTH-STAR's own amendment rule. |
| NORTH-STAR "whether or not anyone is watching" | **Struck.** Durability serves intent execution; unattendedness is a consequence, not the thesis (owner ruling, 2026-08-17). |
| ADR 0004 D9 nightly trigger | **Removed**, per the inbox proposal §18. Adopted unchanged. |
| ADR 0001 platform boundary | **Unchanged.** Linux, macOS, Windows-under-WSL2. No native Windows. |
| `.sergeant/index.md` "17 published workflows" | **Reframed.** Published-procedure framing becomes template framing. The count stops being narrated prose and becomes a computed assertion. |
| CLAUDE.md BU-0109 (one developer per installation) | **Unchanged and reinforced.** See §3.4. |

---

## 1. Executive summary

sergeant-rs has three artifacts with three audiences cohabiting one repository:
the **distro** (always-on doctrine, skills, workflow templates), the **engine**
(`src/`, `tests/`), and the **development record** (run evidence, contracts,
rulings, corpora). The record is 1,301 of 1,325 tracked markdown files and
743,000 words. The product surface is `AGENTS.md` at 2,349 words.

The consequence is not untidiness. It is a measurable retrieval failure, and it
has already cost delivered work:

- A shipped workflow template mandates a CLI flag that does not exist,
  in bold, as a non-negotiable step (§2.4). This is the strongest instance
  and it is a live correctness defect, not merely an organizational one.
- A session drove `no-mistakes` by hand for a full day, reimplementing
  `validate-and-ship`'s gate-driving and custody-reconciliation stages
  badly and inventing a workaround around a `--keep-local` flag it did not
  know existed — because `docs/DEVELOPMENT.md` restated the procedure in
  prose and never named the workflow, and "a reader who finds the prose
  complete never reaches the catalog" (LESSONS L20).
- A session quoted the `pgrep` bracket rule in a dispatch brief and then
  killed its own shell twice the same day.
- The 2026-08-16 CI/CD proposal reconciled carefully against ADR 0001,
  ADR 0004, and the S-series while missing that NORTH-STAR holds its central
  deliverable behind a gate.

The second item carries its own small demonstration. An earlier draft of this
proposal described it as "seven stages," taking the figure from
`docs/DEVELOPMENT.md`, while LESSONS L20 names two specific stages. Both
documents record the same incident at different magnitudes, and neither
points at the other. That is routing fog, found while correcting a citation
about routing fog.

This proposal does five things:

1. **Splits the repository in two.** `sergeant-rs` is the product. A new
   `sergeant-rs-workspace` is a Sergeant estate that mounts it and owns the
   development knowledge library.
2. **Rewrites `AGENTS.md`** as Layer-0 doctrine: the routing table, the
   authority ladder, the minimality ladder, and the guardrails — with CLI
   mechanics removed, because the binary documents its own surface.
3. **Designs the knowledge layer** on three substrates (append-only evidence,
   canonical rulings, rebuildable derived views) with scoped rather than flat
   retrieval.
4. **Makes the distro an embedded, co-versioned artifact.** `v0.3.0` is one
   thing: a binary that ships the doctrine it knows how to use.
5. **Builds the release pipeline** around that single artifact, adopting the
   inbox proposal's Slices 1–3 now and re-scoping Slice 4 for two artifact
   classes.

Governing constraint throughout: **this is a solo-maintainer project, and the
plan states its degradations rather than claiming enforcement it does not have.**

---

## 2. The diagnosis, measured

### 2.1 Six Memory Failure Modes, scored against this repository

Applying the diagnostic (owner's knowledge base, 2026-07-11):

**Revised 2026-08-17 after SPLIT-1.** The original scorecard claimed RED on
all six. Two scores did not survive adversarial review and are corrected
below; the revision is recorded rather than silently applied, per this
repository's own convention. The corrected scorecard is weaker and remains
sufficient — see §2.1.1.

| Mode | Score | Evidence in this repository |
|---|---|---|
| Routing fog | AMBER | "Why did we choose X" has four candidate homes: `docs/adr/`, `reference/proposal-*.md`, GAUNTLET rulings, `docs/icm/`, with no stated rule for which owns a given question. **Corrected:** the original also claimed the bounded-judgment ladder "declares itself canonical in three places at once." That is false. One self-declaration exists (`.sergeant/common/contexts/bounded-judgment.md`), and `docs/icm/convention.md` explicitly *defers* to it — "canonical sources fixed elsewhere, referenced here rather than duplicated," which is the fix for routing fog, not an instance of it. |
| Dark corner | AMBER | **Corrected:** the original claimed no durable home for episodic memory or rejected approaches. `LESSONS.md` and `GAUNTLET.md`'s deviation register provide partial homes for both. The narrowed claim that survives: decisions reached in conversation have no home until someone writes a proposal — evidenced by ADR 0014 existing only because this proposal's own session forced it. |
| Arrival gap | RED | The ladder is unreachable in direct in-session mode. Two recorded instances stand: the `pkill` double-kill (a session quoted the bracket rule in a brief, then violated it twice the same day) and the CI proposal reconciling against ADRs while missing NORTH-STAR's gate. A third, `--intent-file`, is stronger than either — see §2.4. **Corrected:** the original also cited LESSONS L20; that citation overstated what the record documents and is withdrawn. |
| Stale signal | RED | `gh pr edit` carries three dated superseding rows; the Claude CLI version carries three drift rows. Staleness is detected by human diligence, not by the system. |
| Boundary blur | RED | `AGENTS.md` ↔ `docs/DEVELOPMENT.md` overlap. `AGENTS.md`'s own "the source that owns that topic wins" rule is an admission that boundaries blur. |
| Instruction fiction | RED | `.sergeant/index.md` narrates its own count; three `@@refs` resolve to nothing; NORTH-STAR described a clone model now abandoned; and a shipped workflow mandates a CLI flag that does not exist (§2.4). |

### 2.1.1 Why the corrected scorecard still carries the argument

Four RED and two AMBER is a weaker result than six RED, and the correction
matters: the assumptions critic's characterisation of the original —
*"motivated reasoning toward a RED verdict the section had already committed
to"* — was accepted at adjudication.

What the argument actually needs is not a perfect score. It needs that the
failures are real, recorded, and have already cost delivered work. Four RED
scores, two of them evidenced by incidents in this repository's own lessons
and environment files, meet that bar. The two AMBER scores narrow to claims
that still hold. A diagnostic that came back partly green would have been
more credible than one that came back uniformly red, and the corrected
version is the one to reason from.

### 2.2 Template contamination, measured 2026-08-17

Per *Skill Libraries as Simulated Work Environments*: procedure decays on a human
timescale, state decays on every commit, and nothing in the artifact tells you
which kind you are holding.

```
template .md files under .sergeant/workflows + skills        216
  hard references into reference/sergeant-upstream/           91  files
  BU-#### citations into the behavior-unit corpus            611  across 89 files
```

**Revised 2026-08-17 after SPLIT-1.** The original reported a third figure —
162 "contamination candidates" — as the headline. It is withdrawn. It rested
on a broad keyword grep with no stated methodology, and independent
reconstruction lands near 117. The critic's finding was upheld: a number
whose derivation cannot be repeated is not evidence.

The two counts above hold exactly and were independently re-measured twice.
They are also the ones that carry the argument on their own: **91 template
files reference frozen evidence that is leaving for the workspace repo, and
611 citations point into a corpus that will never ship to a user.** Moving
the record without decontaminating the templates dangles references inside
the product's own shipped artifact — and that follows from the unambiguous
counts alone, without any judgment about how many files are "contaminated."

The procedure/state test still needs applying file by file. That triage is
Phase 2's work, not a number this proposal is entitled to assert in advance.
Adversarial sampling suggested the real figure is closer to the 100 mark than
to 162, and the test does not cleanly partition every file — some are neither
purely procedure nor purely state. Phase 2 must therefore produce a
three-way classification, not a binary one.

### 2.3 `@@ref` integrity, measured 2026-08-17

Scope for every count below: **all 216 `.md` files** under
`.sergeant/workflows/` and `skills/`. The original mixed two scopes without
saying so and reported four numbers wrong; they are corrected here and were
re-measured twice.

```
template .md files                                216
stage CONTEXT.md files (numbered stage dirs)       80
blocks titled "Bounded judgment" (all 216 files)   82
  files citing @@bounded-judgment                  79   (was reported 77)
@@refs in use   bounded-judgment 109  (was 107)
                tdd               14  (was 10)
                name               8  (was  6)
                test-quality       4
                domain-modeling    1   ticket-shaping 1   triage-state-machine 1
contexts that exist   bounded-judgment.md  tdd.md  test-quality.md
```

Note that "80 stages" and "82 blocks" are drawn from different scopes — the
82 includes workflow-root and skill files, not only numbered stage
directories. The original presented them as one series.

Three refs resolve to nothing. All three are narrated in
`wayfinder/CONTEXT.md` as known gaps, which is the interesting part: **only prose
distinguishes a live reference from an acknowledged gap.** That distinction must
become machine-legible.

### 2.4 A shipped template mandates a flag the binary does not have

Found 2026-08-17, after SPLIT-1 closed, while checking how intents are
passed to `sgt run`.

`.sergeant/workflows/dispatch/05-classify-risk/CONTEXT.md` requires
`--intent-file` three times, in bold, for any objective matching its
safety-sensitive keyword set (auth, security, secrets, payments, databases,
migrations, production, destructive, persistent state, state transitions),
and states explicitly that this is **"not a delegated judgment call; the
keyword match is fixed."**

No such flag exists. Not on `sgt run`, not on `sgt dispatch`, not anywhere
in `src/`. An actor following this published template on a
security-or-payments objective is instructed to take a mandatory,
non-negotiable step it cannot perform.

This is the strongest instruction-fiction instance in the repository and the
clearest argument in this document, because it is exactly what Phase 5's
cross-repo skew check automates — and it was found by hand, by accident,
while asking an unrelated question about shell quoting. It is also a live
correctness defect in shipped content, not merely an organizational one.

**Consequence for the plan:** the doctrine↔binary skew check moves out of
Phase 5 and into Phase 0 in reduced form (see §6). It does not need both
repositories mounted to compare doctrine's CLI claims against `--help`
output; it needs one repo and a built binary. Phase 5 retains the full
cross-repo version.

---

## 2.5 Corrections to this proposal's own development

Added 2026-08-17 after SPLIT-1's fidelity axis found that this document
presented its conclusions with none of the history that produced them — that
the corrections survived in ADR 0014 but "not in the document that is
supposed to carry them forward." Its refuter called this the strongest
finding in the file. The anti-duplication rule is satisfied by
summary-plus-pointer; it was not satisfied by silence.

Five positions in this document were reached by correcting Captain's first
answer. Full records in ADR 0014.

1. **Notion as the knowledge system of record.** Captain proposed migrating
   the compiled knowledge layer to the owner's Notion workspace. Rejected —
   it is a personal idea store, and making it the project's record would
   make every contributor depend on it. The owner's own Notion decision page
   and `reference/notes/ideaos-agent-contract.md` had already ruled the same
   way; Captain had read the latter and proposed it anyway. §3.2's
   three-substrate design is what replaced it.
2. **Three repositories, and the wrong dependency direction.** Captain
   proposed product / estate / record, with the distro extracted *out of*
   the dev repo. The owner corrected both: two repositories, and the dev
   estate consumes the *released* distro rather than a build-tree copy.
   The corrected direction is load-bearing for §3.4's trap and for the
   bootstrap hazard in ADR 0014's consequences.
3. **`sgt workflow diff`.** Captain proposed the verb, then withdrew it on
   R1 grounds once templates were reframed as examples — then had to
   restore the underlying property when the research showed a fork has no
   invalidation mechanism. §4.7's edition marker is the corrected form: the
   property without the verb.
4. **The inbox.** Told a proposal was "in the inbox," Captain searched
   Gmail. The inbox is a documented host convention in
   `docs/environments/cerberus.md`, a file Captain had already read for
   other facts. This is the diagnosis in §2 happening to the author of §2.
5. **PACE and succession-of-authority.** Captain added both rungs below J0
   without a ruling, and SPLIT-1 flagged them as invented scope carried
   into the decision register. The owner subsequently adopted them
   (2026-08-17). The content stands; the defect was asserting doctrine and
   then citing the assertion.

A sixth, recorded for the same reason: Captain reported two SPLIT-1 findings
to the owner as conclusions before the adversarial round ran. Both were
refuted — the ADR 0008 storage-path collision, and the authority problem in
§9. Relaying unrefuted critic output as findings defeats the purpose of
having a refutation stage.

## 3. Target architecture

### 3.1 Two repositories

**`sergeant-rs` — the product.**
`src/`, `tests/`, `scripts/`, `Cargo.*`, `AGENTS.md`, `skills/`,
`.sergeant/workflows/` (templates), `docs/adr/`, `NORTH-STAR.md`,
`docs/DEVELOPMENT.md`, `docs/environments/`, `docs/glossary.md`,
`SECURITY.md`, `CONTRIBUTING.md`, `CHANGELOG.md`.
Ships `v0.x.y` = binary + embedded distro, one tag, one changelog.

**`sergeant-rs-workspace` — the estate and knowledge library.** Starts clean; git
history is recoverable from `sergeant-rs`.
`sergeant.toml` (mounting `sergeant-rs`), `.sergeant/local/workflows/`
(workspace-local validators), `DEVELOPMENT.md`, and the knowledge library.

Ownership rule, inherited from `AGENTS.md` and scaled across repositories: *when
two sources disagree about a behavior, the one that owns that topic wins.*

### 3.2 Three substrates in the workspace

Correcting a two-way split: repository memory is itself canonical and is **not**
derivable from session logs.

| Substrate | Contents | Discipline |
|---|---|---|
| **Evidence** | `docs/gauntlet/runs/`, `resources/`, `reference-corpus/`, `reference/sergeant-upstream/` | Append-only. Never edited. Superseded is *marked*, never deleted. No review gate. |
| **Canonical rulings** | Owner rulings, milestone contracts, deviation register, proposals | Git-canonical. The compiler may index but never regenerate. Reviewed. |
| **Derived views** | Topic indexes, decision registers, lesson digests, host-fact wings, restart briefs | Fully rebuildable from evidence + rulings. Deletable without loss. No review gate. |

ADRs stay in `sergeant-rs` — a decision that binds present-day behavior ships
with the code. The *argument* that produced it goes to the workspace.

**Two change disciplines, not one.** Workspace behavior (local workflows, estate
config) is reviewed. The knowledge library is appended and compiled. Applying one
policy to both is how a 27,834-word ledger becomes unmaintainable.

### 3.3 Scoped retrieval, not flat search

"What's the gotcha on this host" is a **scope query, not a similarity query.**
Partition by durable operator axis — host, repo, workflow, milestone — before any
semantic search. A flat grep across 1,100 files is the irrelevant-token-ratio
failure by construction. Host-environment facts get a hard-scoped partition of
their own.

**Gate on Five Context Failures.** A derived view belongs only when it cures a
named failure:

| Failure | Curing view |
|---|---|
| Session amnesia | Restart brief per repo/milestone |
| Codebase blindness | Repository shape map |
| Decision blindness | Decision register with rationale + supersession chain |
| Impact blindness | Symbol/contract cross-reference |
| Prompt deafness | Routing table that fires *before* generic retrieval |

A compiled index mapping to none of the five is deleted. This is the defense
against the corpus growing from 1,100 files to 3,000.

### 3.4 Scope ceiling

Sergeant supplies **motion** — project topology, assignments, worktrees, review,
validation, handoff. The workspace is a **constructor**, not the root every Work
inhabits. Explicitly out of scope: a central mission authority, a federated
ownership model, an organizational knowledge system. Reinforced by BU-0109 —
federated authority and conflicting ownership do not exist at single-developer
scale, so this project structurally cannot validate claims about them.

**Trap to avoid:** the workspace repo must never become the default context
loaded into every Work. Context is tiered **bound / referenced / reachable**. An
actor with a grounded pointer does not search, and search is where relevance
dilution begins.

---

## 4. Doctrine decisions

### 4.1 `AGENTS.md` becomes Layer-0 only

Per the Four-Layer model: Layer 0 automatic memory · Layer 1 tool-native · Layer
2 coordination · Layer 3 project-specific. Boundary rule: **Layers 2–3 explain
why and what; Layer 1 explains how to use the tool.**

Consequence: the entire "Standard workflow loop" section — eight numbered steps
naming verbs, flags, and JSON fields — is Layer-1 content occupying Layer 0. It
is removed, not trimmed. Every `--turns`, `--ceiling-secs`, and
`envelope.turns_spawned` reference is both bloat and skew surface. The binary
documents its own surface; §6 Gate F enforces agreement.

`AGENTS.md` retains: the routing table (Layer 2 — which source is authoritative
for which knowledge type), both ladders in full, the guardrails, and one pointer
to `docs/DEVELOPMENT.md`. It should get materially **shorter** than 2,349 words.

### 4.2 CAN and SHOULD are separate registers

> A permission system alone cannot produce judgment. Behavioral instructions
> alone cannot enforce authority.

**SHOULD** — disposition, norms, escalation expectations: both ladders, the
routing table, review posture.
**CAN** — enforceable authority: `sgt`'s refusals, submit-time capability
preflight, worktree isolation, the daemon's single-owner data-dir lock.

They are written in separate sections and **no SHOULD sentence may imply
enforcement.** Today's Guardrails section mixes them in one voice, so a reader
cannot tell which lines are enforced by code. This is also the mechanistic
explanation for instruction fiction: prose cannot enforce, so prose that sounds
like a constraint eventually diverges from behavior.

### 4.3 Both ladders ship inline in `AGENTS.md`

**Ponytail Minimality Ladder (R1–R7)** — construction: *should this exist?*
R1 does this need to exist (YAGNI) · R2 already in this codebase, reuse · R3
stdlib · R4 native platform feature · R5 installed dependency · R6 one line · R7
only then, the minimum that works. Carries its *what minimality does not mean*
clause: it never excuses skipping tests, docs, observability, recovery, or
necessary architecture.

**Bounded-Judgment Ladder (J5–J0)** — authority: *may I decide this?*
Unchanged from `.sergeant/common/contexts/bounded-judgment.md`, which remains the
stage-specialization reference. Stage blocks *narrow* it; they no longer
introduce it.

**Routing ladder** — currently prose in "When NOT to use `sgt`". Restated in the
same shape as the other two.

### 4.4 Two rungs added below J0

The ladder has no answer for "the named authority is unavailable," which for an
unattended overnight Work is the ordinary case, not an edge case.

- **PACE** — Primary / Alternate / Contingency / Emergency, an ordered fallback
  per signal route, declared by the intent.
- **Succession of authority** — who assumes decision authority when the named
  authority is unreachable, and what classes of decision succession does *not*
  transfer.

### 4.5 Failure attribution taxonomy

Adopted verbatim for classifying any `needs_input`, deviation, or bad run:

| Symptom | Attribution |
|---|---|
| Wrong packet | Orchestration / knowledge / policy |
| Sound packet, poor judgment | Reasoning / model fit |
| Correct decision, bad effect | Execution / tool / environment |
| Correct effect, missing evidence | Observability |
| Repeated known failure | Recovery / learning / governance |

Exists specifically to prevent the capture signal *"deviations are reclassified
rather than corrected."*

### 4.6 Anti-capture, honestly degraded

Four separations: **worker** (performs) · **specification** (approves normative
change) · **detection** (defines and evaluates signals) · **arbitration**
(resolves contested classification).

Rules adopted:

- A Work item that hits a constraint may **propose** an `AGENTS.md` change; it
  may never be the sole approver.
- Whoever defines "did it work" must not be whoever did it.
- **Related-party check:** a proposal that would excuse the proposer's own
  recorded deviation escalates; it never self-merges.
- **Tripwire:** any rung-lowering (J0 → J2) is an explicit, reviewed
  specification change. Never emergent drift.

At N=1, separation is **contextual, not identity-based**, supermajority is 1/1,
and recusal metrics are undefined. This is stated, not concealed: *useful
scaffolding, not full architectural enforcement.*

**Self-audit.** This plan's own direction — "`needs_input` means the ladder ran
out, not that a human is required" — is a deliberate move toward autonomy, and
*"human-review paths quietly become autonomous"* is a named capture signal. The
tripwire in §4.6 exists to keep that move explicit and reviewable rather than
emergent. Recorded here so a later reader can hold this proposal to it.

### 4.7 Templates carry an edition marker

A fork has no invalidation mechanism — no edition number, no survey date, no
revision program. Each shipped template declares the distro version it was forked
from. `sgt doctor` reports drift; there is no `workflow diff` verb. Templates
that carry **state** (ownership, layout, architecture) do not ship at all.

---

## 5. Amended North Star statement

> A stranger runs `curl … | sh`, `sgt init`, `sgt claude` — and from then on
> talks to their estate in plain language. *"Let's work on the payment api."*
> *"Why is the ingress controller erroring?"* *"Research this PRD across the
> backend group."* Captain shapes those into intents; `sgt` executes them to
> completion in isolated worktrees. **The intent carries its own authority:
> `needs_input` means the ladder ran out, not that a human is required.** The
> distro ships templates — ways you *could* work — and you write your own locals.
>
> **Acceptance:** the Redditor gets from `curl` to a finished change on a branch
> without reading this repository.

Departure to record explicitly: firstmate's distribution-unit claim — "there is
no application to install, because the cloned repository *is* the distro" — is
the lineage's strongest prior art, and this proposal departs from it in favour of
embedding for co-versioning. The amendment must say so rather than quietly drop
it.

---

## 6. Work groupings

Phases are ordered by dependency, not importance. Phase 0 and Phase 1 are
independent of the split and correct regardless of whether the rest lands.

### Phase 0 — Measure and correct triggers
*No dependencies. Correct regardless of everything below.*

- Inbox Slice 3 in full: remove `pull_request`, `schedule`, and
  `release: published` from `matrix.yml`; add `workflow_call` +
  `workflow_dispatch`; add `workflow_call` to `coverage.yml`.
- Inbox Slice 1 measurement: OpenSSF Scorecard baseline, CodeQL default setup
  and initial triage, `cargo-deny` against the current graph, OSPS Baseline
  v2026.02.19 Level 1 control-by-control.
- NORTH-STAR amendment (§1 table, three entries) drafted for owner ruling.
- **Single-repo skew check** (moved here from Phase 5 by §2.4). Extract every
  `sgt <verb> --<flag>` claim from `AGENTS.md`, `docs/DEVELOPMENT.md`,
  `skills/`, and all 216 template files; diff against the built binary's
  `--help` surface. Needs one repo and a build, not both repos mounted.
  Fix `--intent-file` and whatever else it surfaces. The full cross-repo
  version stays in Phase 5b.

### Phase 1 — Doctrine
*No dependencies. Highest value per unit of effort.*

- `AGENTS.md` rewritten per §4.1–§4.4: Layer-0 only, CAN/SHOULD registers, both
  ladders inline, routing ladder restated, PACE and succession added, all
  Layer-1 CLI mechanics removed, all BU citations removed.
- `docs/DEVELOPMENT.md` absorbs the development-specific content that leaves.
- Failure-attribution taxonomy (§4.5) added to the shared bounded-judgment
  context.

### Phase 2 — Template decontamination
*Depends on: Phase 1's procedure/state test being settled.*

- Triage all 216 template files against the procedure/state test.
- Resolve 611 BU citations and 91 `reference/sergeant-upstream` references:
  inline the substance, or drop the citation. A shipped template may not
  reference content that does not ship.
- Resolve the three dangling `@@refs`: create the context, or mark the gap in
  machine-legible frontmatter.
- Add edition markers (§4.7).

### Phase 3 — Override mechanism
*Only new engine work in the plan. **Edition-marker format must be settled
first** (see G3, corrected): Phase 2 writes markers into 216 files, so their
format is a Phase-2 input, not a Phase-3 output.*

**Ponytail rung: R4/R6.** Recorded per `docs/DEVELOPMENT.md`'s requirement
that every design decision log its rung — the original omitted this, found by
SPLIT-1's invariants axis.

- **R4 (native platform feature)** — local-shadows-stock resolution is
  directory precedence, the same mechanism `/etc` vs `/usr/lib` and
  `sites-available` already use. No new abstraction, no registry, no merge
  semantics. Composition (local stages patching into a stock workflow) would
  be R7 and is explicitly declined until real use demands it.
- **R6 (one line)** — edition drift is a string comparison between a
  frontmatter field and the binary's own version constant. No diffing, no
  three-way merge, no `sgt workflow diff` verb.
- **R2 (reuse)** — `sgt doctor` already owns "report a fixable fault with a
  named remedy." Drift detection is one more check in an existing surface,
  not a new verb.
- `sgt workflow fork <name>` is the one genuinely new verb. R7 is claimed
  only here, and only because R2–R6 fail: no existing verb copies stock to
  local with provenance, and `cp` loses the edition stamp that makes drift
  detectable at all.
- Embedding: stock templates compiled into the binary, written by `sgt init`.

### Phase 4 — The split
*Depends on: `sergeant-rs-workspace` (created by the owner 2026-08-17).*

**This is not a `git mv`.** Owner ruling, 2026-08-17: *"We don't just want to
move the mess."* Extraction precedes migration for every document, not only
the obvious ones.

- Workspace `sergeant.toml` mounting `sergeant-rs`; `repos/` working set.
- **Workspace `AGENTS.md`** (owner ruling) — purpose-built for developing
  sergeant-rs, structurally modelled on the product's but Layer-3 in content:
  how to compile the record, the two change disciplines, how to open PRs
  against the mounted product. Short.
- Workspace `DEVELOPMENT.md`; two-discipline change policy.
- **Extract, then move.** For every document leaving the product repo:
  anything that binds present-day behavior becomes an ADR in `sergeant-rs`
  *first*; the argument that produced it goes to the workspace. Applies to
  all nine `reference/proposal-*.md`, to `docs/icm/`, and to the corpora.
  Moving a live ruling into a private repo would strand it.
- **`GAUNTLET.md` and `LESSONS.md` are decomposed, not moved** (owner
  ruling). 32,110 words become typed records in the workspace's OKF
  structure — rulings to the decision register, lessons to the lesson
  digest, deviation rows to their own type, run pointers to evidence. They
  are the compile pass's first real customer, which is the right way to
  prove it: if the compiler cannot handle this corpus, it cannot handle any.
- **`docs/environments/` splits by kind** (owner ruling): the *capability*
  stays with the product — `scripts/probe-env.sh` and the rule that a host
  is measured before it is trusted, because `sgt` should understand its own
  environment. The *measurements* — the dated per-host tables — become a
  workspace host wing. Same procedure/state line the template test uses.
- `sergeant-rs` sheds `docs/gauntlet/`, `reference/`, `reference-corpus/`,
  `resources/` once extraction is complete.

### Phase 5a — Structural validator
*Depends on: Phase 2. Runs against one repository — corrected from the
original, which bundled these with the one check that needs both.*

- Every template `index.md` parses and carries `type` + `status`.
- `.sergeant/index.md` catalog matches the directory — computed, not narrated.
- Every `@@ref` resolves, or is explicitly marked as a gap.
- Bounded-judgment coverage across stage contexts.
- No reference to `.sergeant/drafts/` reachable by name.
- `AGENTS.md` routing-table rows resolve to files that exist.
- No shipped template references non-shipping content.

### Phase 5b — Cross-repo skew check
*Depends on: Phase 4. The only check that genuinely needs both repositories
mounted in one estate.*

- Full doctrine↔binary agreement across both repos, including the workspace's
  own `AGENTS.md` and local workflows. **This is the co-version skew check
  in its complete form.** Its reduced single-repo ancestor runs in Phase 0
  (§2.4) and should already have caught `--intent-file`; 5b exists to keep
  it green across the release boundary rather than to find it once.

### Phase 6 — Release pipeline
*Depends on: Phases 3 and 5.*

- Inbox Slice 2: `deny.toml`, dependency review, Dependabot (cargo +
  github-actions, weekly grouped), full-SHA action pins, least-privilege
  workflow permissions, `SECURITY.md`, ruleset required checks.
- `CONTRIBUTING.md` — writable truthfully only after Phase 4.
- `release.yml`: `workflow_dispatch` only, dry-run default. One version, one
  artifact identity. Gate F joins Gates A–E.
- `dist` measurement against bundled DuckDB, CycloneDX SBOM, provenance
  attestations, artifact smoke tests, immutable publish, `CHANGELOG.md`.
- Full dry run adjudicated before the first real tag.

---

## 7. Gates

Each gate is a hard stop. A phase does not begin until its gate is green.

**Corrected 2026-08-17 after SPLIT-1.** Two defects were confirmed: G3 placed
the edition-marker decision *after* the phase that consumes it, and three
gates listed Captain as both performer and gatekeeper.

| Gate | Before | Condition | Green-lit by |
|---|---|---|---|
| **G0** | Phase 0 | Owner has ruled on the NORTH-STAR amendment (§1) | Owner |
| **G1** | Phase 1 | Procedure/state test settled; CAN vs SHOULD boundary agreed | Owner |
| **G2a** | **Phase 2** | **Edition-marker format settled** — moved ahead of Phase 2, which writes markers into 216 files | Owner |
| **G2b** | Phase 2 | Phase 1 merged; no BU citation resolves to content that will not ship | Captain (declared) |
| **G3** | Phase 3 | Template triage complete | Owner |
| **G4** | Phase 4 | `sergeant-rs-workspace` reachable; override mechanism merged | Owner |
| **G5** | Phase 5a/5b | Phase 2 complete (5a); split merged (5b) | Captain (declared) |
| **G6** | Phase 6 | Skew check green on a real commit; Slices 1–3 landed | Captain (declared) |
| **G7** | First tag | Full release dry-run adjudicated; OSPS L1 assessed | Owner |

**Declared degradation.** G2b, G5, and G6 are green-lit by the party that
performs the work behind them. At one maintainer, identity-based separation
is unavailable and pretending otherwise would be the ceremony §4.6 warns
against. What is available and required instead: each of the three is a
*mechanical* condition — a validator exit code, a merge state, a green
check — not a judgment call, so "did it pass" is checkable by the owner
after the fact without re-doing the work. A gate that required Captain's
opinion of Captain's output would not be admissible here.

---

## 8. Owner manual checklist

Items that cannot be executed from a CLI session and block downstream phases.

| # | Action | Blocks |
|---|---|---|
| 1 | Create `sergeant-rs-workspace`, private | G4 / Phase 4 |
| 2 | Enable CodeQL default setup (Rust, `none` build mode) | Phase 0 |
| 3 | Enable secret scanning + push protection | Phase 0 |
| 4 | Enable Dependabot alerts + security updates | Phase 6 |
| 5 | Enable private vulnerability reporting | Phase 6 |
| 6 | Enable dependency graph | Phase 0 |
| 7 | Enable immutable releases | Phase 6 |
| 8 | Main ruleset: add required status checks once Tier-1 job names settle | Phase 6 |
| 9 | Rule on the NORTH-STAR amendment (§1, three entries) | G0 |
| 10 | Rule: accept / reject / defer the inbox CI proposal as re-scoped here | G0 |

Items 2, 3, 6 unblock the Phase 0 measurement and are the highest-priority
manual work. Item 1 is the long pole for everything after Phase 3.

---

## 9. Overnight captain scope

What may proceed unattended, and what may not. This section is itself a test of
§4.3–§4.4: an overnight run that resolves its own questions from intent and
delegated authority is the recursion proof; one that stalls on `needs_input` at
02:00 is evidence the ladder is underspecified.

| Scope | Work | Rung |
|---|---|---|
| **Authorized** | Phase 0 Slice 3 workflow edits; branch + PR | J4 — explicitly scoped here |
| **Authorized** | Phase 0 Slice 1 measurement: `cargo-deny` run, OSPS L1 assessment draft, Scorecard baseline capture | J4 |
| **Authorized** | Phase 1 `AGENTS.md` rewrite + `DEVELOPMENT.md` absorption, as a PR for morning review | J2 — delegated drafting; owner approves |
| **Authorized** | Phase 2 triage **report** — classify all 216 templates, produce the inventory | J2 |
| **Authorized** | NORTH-STAR amendment **draft** | J2 — draft only |
| **Not authorized** | Merging any PR to `main` | J0 |
| **Not authorized** | Applying Phase 2 template edits | J0 — awaits G3 |
| **Not authorized** | Any Phase 3+ work | J0 — gated |
| **Not authorized** | Creating repositories or changing GitHub settings | J0 — owner-only |
| **Not authorized** | Deleting or moving evidence | J0 |

**Corrected 2026-08-17 after SPLIT-1.** The original PACE ladder made
"Alternate" a licence to decide — *"take the conservative option and record
it"* — which authorises exactly the guess J0 forbids. PACE is a ladder of
**routes to an authority**, never a ladder of decision latitude. Corrected:

- **Primary** — the owner, live. Ask and wait.
- **Alternate** — the owner, asynchronously: the question is written into
  the PR body as a blocking open question and the workstream continues
  *around* it. The decision is not taken; it is deferred in a place the
  owner will see.
- **Contingency** — no owner route available and the decision is J0: stop
  that workstream, continue the others, leave the question recorded.
- **Emergency** — destructive or irreversible action in question: stop
  entirely, leave the tree clean, touch nothing further.

At no rung does PACE convert a J0 into a decision. Degrading the *route*
never degrades the *authority* — that is the whole distinction between this
and the failure mode SPLIT-1 caught.

**Every PR carries a decision log.** Per `bounded-judgment.md`'s Decision
evidence section, one row per material choice — `| Decision | Rung |
Evidence | Resolution |` — each tracing to ADR 0014, an earlier ADR, or
marked **undelegated — parked**. Anything untraceable is not resolved in
prose; it goes in the PR body as an open question with a recommendation.
This is what makes "owner approves" a review of decisions rather than an
audit of paragraphs.

---

## 10. Acceptance criteria

1. `AGENTS.md` contains no `sgt` flag documentation; the binary is the sole
   authority for its own surface.
2. `AGENTS.md` contains both ladders in full, inline, reachable without a
   workflow stage active.
3. CAN and SHOULD appear as separate, labelled registers.
4. No shipped template references content that does not ship.
5. Every `@@ref` resolves or is marked as a gap in frontmatter.
6. `.sergeant/index.md`'s count is computed and asserted, never narrated.
7. Gate F runs in CI and fails on a manufactured doctrine/binary skew.
8. A fresh session, given only the workspace's restart brief and no transcript,
   reacquires the current state of a milestone. (Fresh-session handoff test.)
9. The workspace's knowledge library carries no view that fails the Five Context
   Failures gate.
10. `v0.x.y` names exactly one artifact identity: binary plus embedded distro.
11. A failed qualification leaves no published tag or release.
12. `CONTRIBUTING.md` is true — every path it names is reachable in the product
    repo alone.
13. NORTH-STAR carries dated amendments for all three §1 entries.
14. Six Memory Failure Modes re-scored after Phase 5, with the deltas recorded.

---

## 11. Open decisions

| # | Question | Consequence |
|---|---|---|
| 1 | Does `docs/environments/` stay in the product repo or move to the workspace? | Host facts are workspace-shaped by nature, but `scripts/probe-env.sh` ships with the product. Currently assumed: stays. |
| 2 | Edition-marker format — frontmatter field, or a generated manifest? | Affects Phase 2 and `sgt doctor`. |
| 3 | Does the workspace repo go public later? | Affects whether evidence needs a secrets sweep before the split. |
| 4 | Which derived views ship in the first compile pass? | Five Context Failures gates the set; the order is still open. |

---

## 12. Non-goals

No native Windows. No crates.io, Homebrew, or package-manager publication. No
self-update. No nightly CI. No automatic releases from tags. No semantic-release
inference. No Codecov or second coverage convention. No custom signing keys. No
CODEOWNERS at N=1. No required reviewer who does not exist. No central mission
authority, federated ownership model, or organizational knowledge system. No
`sgt workflow diff`. No OpenSSF score chasing. No SLSA level claimed before it is
verified.

---

## 13. Decision register

| Decision | Ruling |
|---|---|
| Repository topology | Two repos: product, workspace |
| Workspace history | Clean start; history recoverable from `sergeant-rs` |
| Distro delivery | Embedded in the binary, written by `sgt init` |
| Versioning | Co-versioned; one tag names binary + distro |
| Workflow packages | Templates, not published procedure |
| Fork mechanism | `sgt workflow fork` + edition marker; no `diff` verb |
| `AGENTS.md` scope | Layer 0 only; CLI mechanics removed |
| Instruction registers | CAN and SHOULD written separately |
| Authority ladder | J5–J0 retained; PACE and succession added below J0 |
| Minimality ladder | Ponytail R1–R7, inline, with its non-exemption clause |
| Knowledge substrates | Three: evidence, canonical rulings, derived views |
| Retrieval model | Scoped by operator axis before semantic search |
| View admission | Five Context Failures gate |
| ADR location | Product repo |
| Workspace change policy | Two disciplines: behavior reviewed, knowledge appended |
| Anti-capture posture | Four separations, contextual at N=1, stated not concealed |
| Authority boundary | The merge, not the artifact — a PR is a request (ADR 0015) |
| PACE / succession | Adopted; routes to an authority, never decision latitude (ADR 0014 d.14) |
| OKF type vocabulary | Compiler proposes, owner ratifies; material divergence escalates (d.15) |
| Workspace `AGENTS.md` | Yes, purpose-built, Layer-3 in content (d.16) |
| Document migration | Extract binding rulings to ADRs first, then move evidence (d.17) |
| `GAUNTLET.md` / `LESSONS.md` | Decomposed into typed records, never moved intact (d.9) |
| `docs/environments/` | Capability ships with product; measurements go to workspace (d.18) |
| Workspace visibility | Private; no secrets sweep gates migration (d.18) |
| Skew check placement | Reduced single-repo form in Phase 0; full cross-repo in Phase 5b |
| Nightly CI | Removed |
| Release trigger | Manual `workflow_dispatch` only |
| Release qualification | Gates A–E from the inbox proposal, plus Gate F |
| SLSA | Generate evidence now; claim a level only after verification |
