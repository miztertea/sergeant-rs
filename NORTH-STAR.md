# NORTH STAR

Adjudicated 2026-08-11 (Cerberus, day 2). This is the destination every
contract, proposal, and AGENTS.md revision cites. It was produced by four
blind research seats, an adversarial synthesis, three steelmanned
competing paths, orchestrator dispositions, and owner rulings — the full
argument record lives in `docs/gauntlet/notes/north-star-{draft,
arbitration,dispositions}-2026-08-11.md`. Everything here is our thinking
as of its date: binding until evidence beats it, then amended in place
with a dated entry.

## Destination

**REVISED 2026-08-17 (owner rulings, in dialogue with Captain; recorded in
`docs/adr/0014-product-workspace-split-owner-rulings.md`).** The original
destination paragraph is preserved below the revision, per this document's
own rule that a superseded statement stays legible rather than being edited
away. It was not wrong when written; two of its clauses have since been
ruled false by decisions this repository has now taken.

*Sergeant is an AgentOS distro — instructions, skills and workflow
templates embedded in `sgt` and written to your estate by `sgt init` —
that turns a general-purpose coding harness into an operator of your
estate, carried by a durable intent-execution engine that gives every
Work its own worktree and a declared mutation surface — authorization,
not a seal — runs intents to completion against it, and journals what
core can prove happened outside that surface as dirty evidence at
retirement rather than silently absorbing it.*

**Now true (2026-08-18), closing the note below.** `sgt init` embeds and
writes the distro — `AGENTS.md`, `skills/`, `.sergeant/common/contexts/`,
and `.sergeant/workflows/` (241 files, measured) — into a fresh estate,
per-file idempotent so re-running `sgt init` stays a no-op (issue #165).
The historical note this replaces is kept legible rather than deleted, per
this document's own rule:

*Not true yet, and marked as such (2026-08-17). The embedding clause
above is the destination, not the current behavior: `sgt init` today
writes `sergeant.toml`, `repos/`, and a `.gitignore`, and no workflow
templates at all (issue #165; measured in
`docs/gauntlet/runs/skew-check-2026-08-17/findings.md` findings 6 and 10).
The Phase 0 skew check flagged this paragraph — revised the same day it
was written — as a present-tense assertion of behavior the binary does not
have, which is exactly the instruction-fiction class this document's own
amendments were correcting. A destination document is entitled to describe
where it is going; it is not entitled to describe that as already working.
This note stands until `sgt init` actually writes the distro.*

What is still not true: the `curl … | sh` installer step named in the
finished-loop paragraph below is a separate, pre-existing release-pipeline
concern (`release.yml`, Phase 6) this change does not touch or claim to
close — only the embedding clause above is what this note is about.

The finished loop: `curl … | sh` → `sgt init` → `sgt claude` → say *"let's
work on the payment api"*, *"why is the ingress controller erroring?"*,
*"research this PRD across the backend group"* → Captain shapes those into
intents and dispatches them → return to a finished change on a retained
branch, a readable transcript, and an honest, bounded cost. **The intent
carries its own authority: `needs_input` means the ladder ran out, not that
a human is required.** The distro ships templates — ways you *could* work —
and you write your own locals.

Acceptance: the stranger gets from `curl` to that finished change without
reading this repository.

Four amendments are folded into the text above:

1. **"A cloned directory of instructions, skills and conventions" is
   struck.** It is now false twice: the distro is embedded in the binary
   (ADR 0014 decision 1), and its contents are templates rather than
   conventions (decision 3). This is a deliberate departure from firstmate's
   distribution-unit claim — "there is no application to install, because
   the cloned repository *is* the distro" — which the owner's own research
   corpus names as the lineage's strongest available prior art. Recorded as
   a departure, not an oversight.
2. **"Whether or not anyone is watching" is struck.** Owner ruling: that
   describes a consequence of durability, not the thesis. *"That's not what
   sergeant is about. It's about executing intentions."* Durability serves
   intent execution; unattendedness is what it buys, not what it is.
3. **Stranger-first framing.** The destination is stated from the
   Redditor's install line inward, rather than from this repository's
   structure outward.
4. **"Isolated worktrees" is stated honestly (amended 2026-08-20, owner
   ruling on issue #180 — `docs/proposals/backlog-closeout-2026-08-20.md`
   kickoff ruling 5).** Each Work's worktree is a declared mutation
   surface, not an enforced boundary: nothing here runs an OS sandbox or
   blocks a write outside it. What core actually does is observe — at
   retirement, per-binding integrity findings
   (`runtime::integrity::IntegrityFinding`: an assigned worktree left
   uncommitted, missing, on the wrong branch, detached, or answering from
   the wrong common dir) and, for every bound mount, whether its
   committed HEAD moved during the Work's window
   (`EstateDriftObservation`, always unattributed — core cannot prove who
   moved it) — and charge what it can prove as dirty evidence, never
   silently absorbing it. A shared mount two Works touch at once is
   accepted risk under this contract, named rather than implied away by
   "isolated" alone; prevention or OS-level sandboxing remains a
   non-goal.

   > **Amended again 2026-08-21** (owner ruling R4 of the *Sergeant speaks
   > Codex* sprint; `[[repo-is-a-snapshot]]` — this paragraph records what was
   > known at this commit, and a later measurement may amend it again).
   >
   > *"Non-goal" here scopes core, not adapters.* Sergeant's core still runs no
   > OS sandbox and blocks no write: the mutation surface is declared and
   > observed, and integrity findings plus estate-drift observations remain
   > the only things sergeant will assert about what a Work actually changed.
   > What this amendment adds is that **an adapter MAY use its harness's
   > native enforcement** where the harness has it, and that doing so does not
   > make core an enforcement layer. The Codex adapter does exactly this: it
   > scopes `codex`'s own sandbox (`workspace-write`, with the Work's declared
   > binding surfaces as the writable roots) to the surface core already
   > declares, and `backend/docker.rs` has done the same thing with
   > bind-mounts and `--network=none` since before this was written down.
   >
   > Three consequences, stated rather than implied:
   >
   > 1. **Observation stays the source of truth.** An adapter's enforcement is
   >    a belt over core's braces. Sergeant charges dirty evidence from what it
   >    observed, never from what an adapter claims to have prevented — because
   >    an enforced surface and an observed surface produce different
   >    retirement stories, and only the observed one is designed.
   > 2. **Enforcement is a capability, so it is admitted by measurement like
   >    every other.** As of this date the Codex adapter's row reads
   >    *enforcement-claimed, not locally proven*: the harness accepts and
   >    echoes back the requested policy, and whether the OS sandbox denies an
   >    out-of-surface write could not be verified on the development host,
   >    whose nested-container environment cannot initialize bubblewrap. An
   >    unverifiable claim is recorded as unverified, not promoted.
   > 3. **A shared mount two Works touch at once remains accepted risk** under
   >    this contract, exactly as the 2026-08-20 amendment says, on every
   >    backend — an adapter's sandbox scopes one Work's writes, not another
   >    Work's.

Original text, historical (adjudicated 2026-08-11):

> *Sergeant is an AgentOS distro — a cloned directory of instructions,
> skills and conventions that turns a general-purpose coding harness into
> an operator of your estate — carried by `sgt`, a durable
> intent-execution engine that runs those intents to completion in
> isolated worktrees whether or not anyone is watching.*
>
> The finished loop: clone → `sgt` on PATH → `sgt init` (trust, estate,
> `repos/` working set) → open your harness → say *"let's work on the api
> bug"* → the harness shapes a structured intent, picks a workflow, drives
> the CLI on your behalf → walk away → return to a finished change on a
> retained branch, a readable transcript, and an honest, bounded cost.
> Acceptance: a stranger reaches that last step in under five minutes of
> setup. (Stranger onboarding itself is gated — see Waves.)

## Ownership

- **Core (`sgt`)** owns durable execution: journal, blobs, projections,
  the Backend boundary, WorkState, the API, admission, holds and message
  delivery to running executions, the terminal output POINTER, backlog
  identity and provenance, the intent schema, the spend envelope.
  (Amended 2026-08-11, R-MVP1-2 held: promote/finalize EXECUTION is
  workflow content invoking a shared deterministic helper — the engine
  learns no output vocabulary; only the pointer is core.)
- **OS (AGENTS.md, skills, workflows, conventions)** owns judgment and
  dialogue: how to talk to sergeant, what deserves the Work apparatus,
  how to ask and guide. AGENTS.md is hand-authored canonical doctrine;
  generated catalogs are disposable.
- **Estate (`repos/`, manifest, per-repo instructions)** owns the working
  set. `repos/` is a mount, never a dev_root; nothing of ours lives at
  `~/`; machine-local truth is in-estate and gitignored.
- **Surfaces (CLI, TUI, harnesses)** own presentation and
  steering through the API only. **A surface adds usability, never
  functionality** (owner, 2026-08-11): the TUI's "conversation" is the
  journal rendered; its attention drawer is `needs_input` from the same
  API; its composer issues the same commands.

Boundary rules R-NS-1..5 as drafted, plus:

- **R-NS-6 (execution ≠ dialogue).** sgt owns message *mechanics* to a
  running execution (`needs_input`/`respond`, journaled); the harness
  owns the *conversation*. Nothing conversational is ever engine work;
  whether a transport's actor can ask mid-run is a measured per-transport
  capability with runtime withdrawal, never new hold machinery.
  Consequence: the WORKFLOW-IF-E3 category is empty — grilling-class
  packages are operator skills.

## The waves (dispositions applied)

**REVISED 2026-08-11 (same day, owner direction — problem-first
re-sequencing): the thematic waves below are superseded by the layered
MVP plan in `docs/gauntlet/notes/mvp-bucketing-2026-08-11.md`** — core →
adapters (Claude + Docker) → CLI → stabilize/measure/cover → content
(AGENTS.md/skills/workflows) → ship to colleagues; P2-JOURNAL and
T-series are post-MVP enhancers that gate on the MVP and never block
it. New rulings absorbed there: the estate manifest as keystone
(`sergeant.toml` estate sections, pin-at-bind, three pens one file),
the adapter-boundary rule (core semantics never defined by adapter
flags — the `--setting-sources` de-leak), cost demoted to adapter
capability with the turn-count envelope core-owned, E3's WORKFLOW-IF-E3
category dissolved by R-NS-6 **while its submit-time capability preflight
survives as an MVP-1 item** (v2 correction, same day — the blanket "E3
dissolved" was absorbing a live cheap item), and the cheap-now rule
(small enabling code lands in MVP even when not direct-MVP). The wave
text below stands as the argument record.

- **Wave 0 — legibility & safety** (falsifiers of the loop): output
  pointer at terminal state + `promote` disposition executing (E6 as
  corrected — output is retained today, not surfaced), `sgt work
  transcript` (E7), canceled-turn usage (E1), the env contract owned by
  the product (E2), submit-time capability preflight, **the per-Work
  spend/turn envelope at the PREPARE/LAUNCH boundary** (sized per L16's
  arithmetic, precondition for any stranger), the blocked exit-door
  fault-injection test, capability-provenance (contract-v2 item 7 — the
  one item measured live). **Acceptance gate: the first fix in this wave
  runs as a sergeant Work against this repo.**
- **In parallel — the instrument**: AGENTS.md rewritten as the canonical
  front door (routing table + standard loop, upstream's shape, sgt
  verbs), minimal `sgt init`, the operator-skills layer (sergeant-help,
  grilling-class re-homes, the dev rulebook as repo content per
  clone-is-distro), `CLAUDE.md → AGENTS.md` symlink, library re-homing
  per the re-triage + absorbed sweep.
- **Wave 1 — the estate**: `repos/` manifest, data-dir default flipped
  in-estate, per-repo instruction contract, E5 discoverability, daemon
  lifecycle + admission verbs (drain = one journaled event pair),
  live-turn stall detection.
- **Wave 2 — queued intents**: backlog as its own durable type
  (captured → intended, two states), idempotent promotion on
  full-causal-context dedup keys, structured intent as progressive
  elaboration (free text stays legal at the CLI).
- **Wave 3 — surfaces, minimally**: T-series slice (composer, legible
  thread, respond, #11/#16), then a second dogfood round before more.
- **Gated ("not yet", each with its unblock condition)**: stranger
  onboarding + prebuilt binary (envelope + dogfood round 2); T-series
  full spec (dogfood round 2); H1 contract-v2 remainder (R-H0-3 probe
  finds a second transport); N4 Docker (Wave 0 + seam ruling); G3
  callbacks (a consumer); G1 scheduler (a promotion policy someone wants
  automated); estate graph (estate landed); clean-distro extraction
  (OS stable).
- **Never**: fleet as a domain object; PM semantics; upstream's
  author-specific integrations; the re-hash intent ceremony;
  reconstructed tmux-era supervision; the settled D7/B1/#131-class
  machinery.

**Amended 2026-08-15 (owner ruling, in dialogue with Captain): the T-series
unblock condition is satisfied.** Wave 3's "dogfood round 2" gate was
written against the record as it stood on 2026-08-11, when the one measured
dogfood run (`docs/gauntlet/runs/dogfood-2026-08-11/`) was the entire
evidence base. Since then the MVP shipped self-hosted (MVP CLOSE-OUT
ledger entry, 2026-08-13), WATCH shipped and was piloted (WATCH ledger
entry, 2026-08-13), the FOUNDATION-1 proposal-grading gauntlet and the
cross-platform bug sprint both ran as dispatched `sgt` Works against this
repo (`GAUNTLET.md`, 2026-08-14), and a MacBook-arrival measurement pass
closed #18/#81/#82/#95 (PR #126, 2026-08-15) — each an instance of the same
loop the gate was waiting to see proven again, not one discrete "round 2"
event by name. Per this document's own rule ("binding until evidence beats
it, then amended in place with a dated entry"), the letter of an unfired
named event yields to that accumulated evidence: **the T-series full-spec
gate is lifted.** The revised proposal
(`reference/proposal-tui-t-series.md`, superseding its 2026-08-11
predecessor) is queued for FOUNDATION-1-style proposal grading under
`docs/gauntlet/contracts/T-SERIES-1.md` before any build begins. The
sibling item sharing this gate's text — "stranger onboarding + prebuilt
binary" — is untouched by this ruling and remains a separate, unscoped
decision.

**Amended 2026-08-17 (owner ruling; ADR 0014 decisions 8 and 9): the
"stranger onboarding + prebuilt binary" gate is lifted.** The 2026-08-15
amendment above deliberately left this sibling item gated and unscoped. It
is now scoped. `~/inbox/proposal-ci-cd-release-engineering.md` (2026-08-16)
designed the release channel that the prebuilt-binary half of this item
requires, and did so without citing this document — reconciling carefully
against ADRs 0001 and 0004 and the S-series while missing that its central
deliverable sat behind this gate. That miss is itself the evidence: the
gate was invisible from the neighborhood that needed it, which is the
retrieval failure `reference/proposal-product-workspace-split.md` §2
measures. Per this document's own rule ("binding until evidence beats it,
then amended in place with a dated entry"), the gate yields.

Scope of the lift: the prebuilt binary and its release pipeline are
unblocked, re-scoped to carry **two artifact classes under one version** —
binary plus embedded distro (ADR 0014 decisions 1 and 2). **Stranger
onboarding itself remains gated**, on the split and doctrine work landing
first: a `CONTRIBUTING.md` or a quickstart that points into 1,301 files of
development record is a signpost into a swamp, and the acceptance test
above ("without reading this repository") cannot pass until that is true.
The proposal implementing this lift is queued for FOUNDATION-1-style
proposal grading under `docs/gauntlet/contracts/SPLIT-1.md` before any
build begins — the third such unit, after FOUNDATION-1 and T-SERIES-1.

## Gaps the record must close (owned by the MVP plan's buckets)

Install path, Work-vs-inline routing judgment (OS-owned), per-repo
instruction contract, the recursion proof (self-hosting measured, not
pledged), cross-repo Work spanning estate entries (the central value
claim — **amended 2026-08-13:** the engine leg landed under
R-MVP1-4/R-MVP1-5 — per-repository worktree bindings under one Work with
partial-materialize rollback, an instruction-policy-agreement refusal at
submit, and repeatable `--repo` plus client-side `--group` expansion,
pinned by `tests/m3_execution.rs::t2_multi_repo_workspace_binds_one_worktree_per_repository`
and `tests/m8_estate_cli.rs::run_group_expansion_itself_survives_an_unrelated_declared_repo_missing_from_disk`.
What remains uncontracted is narrower: cross-repo *delivery ordering* is
planned by hand — since the 2026-08-22 distro rebuild dissolved
`cross-repo-work` into `scope-intent`'s `targets.dependency_order` field
(Captain-side intent-shaping) plus an unbuilt runtime fan-out
(design-proposal-2026-08-22.md §I.3, J.14) — with no engine-side
dependency contract, and the multi-repo execution cwd —
`WorkSurface::execution_cwd` falls back to the surface root for
two-or-more repositories — is construction, not a ruling), soak evidence (#19),
fake-backend timing fidelity, unsupervised-run safety envelope, backlog
dedup scheme, estate trust model.

## Rulings index

Draft §5's twenty rulings stand as written except: #14 upheld in its
inversion of R-H0-2; #20's prebuilt-binary urgency overturned (gated
with onboarding); E6 reframed per D1; E3 dissolved per R-NS-6. Owner
rulings U-R1..R6 stand as amended in
`docs/gauntlet/notes/u-series-scope-draft-2026-08-11.md`, which this
document supersedes as the citation root.
