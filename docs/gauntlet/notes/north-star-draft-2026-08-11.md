# NORTH STAR — sergeant-rs

Synthesis of four seat papers (product, core, ecosystem, history), 2026-08-11.
Every ruling cited below is treated as a hypothesis with provenance. Where measured
evidence beats a ruling, the ruling is named and overturned in §5.

---

## 1. DESTINATION

**Identity sentence (two clauses, both load-bearing):**
*Sergeant is an AgentOS distro — a cloned directory of instructions, skills and
conventions that turns a general-purpose coding harness into an operator of your
estate — carried by `sgt`, a durable intent-execution engine that runs those
intents to completion in isolated worktrees whether or not anyone is watching.*

The first clause is the product. The second is the core. The record has been
running two self-descriptions in parallel — "Depot: a local agent execution
surface" (`reference/proposal-depot-rust-execution-surface.md:1-16`) and "an agent
distro / project-aware first mate" (`reference/sergeant-upstream/README.md:6-11`,
U-R4). They are not rivals; they are the two layers. But the record must stop
alternating: **the distro is the product, the engine is a component of it, and
"Depot" is retired as a name** (frozen proposals keep it as history).

**The finished loop, as a stranger experiences it:**

1. `git clone` the distro. One command puts `sgt` on PATH — **a prebuilt binary,
   not a 10-minute DuckDB compile** (`README.md:20-27`, `CLAUDE.md` Commands).
2. `sgt init` — trust prompt, estate scaffolded, `repos/` mount created, `sgt
   doctor` reports healthy. The verb does not exist today (`src/cli.rs` `Command`
   enum: Status, Run, Work, Respond, Doctor).
3. Open a harness inside the clone. `AGENTS.md` is loaded and the harness now
   knows what sergeant is, what estate it operates, which repos are in the working
   set, each repo's conventions, and when to use `sgt` versus answer inline.
4. Say *"let's work on the api bug."* The harness builds a structured intent,
   picks a workflow, submits, and drives the CLI on your behalf — relaying
   questions to you and answering them where you delegated that.
5. Walk away. Come back to a **finished change landed in your repo**, a readable
   transcript of how it got there, and an honest cost number.

**Acceptance test for "finished":** a stranger who has never read this repo
reaches step 5 in under five minutes of wall-clock setup, and the artifact of
step 5 survives worktree teardown. Today step 1 costs ten minutes, step 2 has no
verb, step 3's `AGENTS.md` is an 11-line ICM stub, and step 5's output is
silently discarded (E6). Nothing in the destination is exotic; all of it is
unbuilt or half-built.

**The estate model:** the estate is the clone. It is self-contained — nothing of
ours lives at `~/` (U-R2). `repos/` is a **working set**, not a home for clones:
each entry is a full clone, a worktree of a clone elsewhere, or a symlink, and
`sgt` requires only that it is a git repo it can cut surfaces from. The estate
manifest declares the expected entries; `sgt init`/`doctor` verify or populate.
The payoff is coherence: frontend, backend and API separately versioned but
jointly reasoned about — a light monorepo assembled by declaration.

**The recursion:** sergeant-rs's own repo is an estate entry. The distro develops
its own core, using its own workflows, through its own binary (U-R1 + U-R3). This
is the strongest available fitness test and it has never been run (§4.4).

---

## 2. OWNERSHIP LAYERS

**Core — the `sgt` binary.** Owns anything requiring a single owner enforcing
durability, ordering, or race-freedom that a script re-running from scratch cannot
honestly provide (core seat's boundary test; adopted). Concretely: the journal and
blob store, all projections, the `Backend` trait's PREPARE/LAUNCH boundary, the
`WorkState` machine, the loopback API, admission blocking, live-turn stall
detection, promote/finalize, backlog dedup identity and provenance, the structured
intent schema.

**OS — `AGENTS.md`, skills, workflow content, conventions.** Owns domain knowledge
and judgment: how to talk to sergeant, what a good intent looks like, review-axis
vocabulary, when an ask deserves the Work apparatus at all, operator skills
(`sergeant-help` and successors). Holds no durable state of its own.

**Estate — `repos/`, the manifest, per-repo instructions.** Owns the working set
and what sergeant should know about each member: conventions, toolchain, which
repos change together. Machine-local truth (`.sergeant/data/`, `repos/` contents)
is in-estate and gitignored; the manifest is tracked.

**Surfaces — CLI, TUI, dashboard, harnesses.** Own presentation and steering only.
The harness is a client like any other, not a privileged layer.

**Boundary rules (violating one is the bug, not the design):**

- **R-NS-1 Durability test (core↔OS).** Judgment content out, durable guarantees
  in. If a thing needs a dedup key, an ordering, or crash-tolerance, it is core.
  If it needs an opinion, it is OS. Review findings split exactly here: the axis
  vocabulary is content, the finding's identity and provenance are engine.
- **R-NS-2 Regeneration test (OS↔generated).** Canonical doctrine is hand-authored
  and a regeneration pass must never touch it; discovery catalogs
  (`.sergeant/index.md`) are generated and disposable. `AGENTS.md` is canonical;
  it *references* the catalog rather than being it.
- **R-NS-3 No-second-home rule (estate).** Nothing of sergeant's lives at `~/`,
  and sergeant never claims to know where your canonical clones live. `repos/` is
  a mount, not a `dev_root`.
- **R-NS-4 Statelessness rule (surfaces).** A surface may never be the only place
  a fact exists, and reaches state only through the API (`tests/m6_surfaces.rs`
  t5 already enforces this). A TUI reveals constraints; it does not invent them.
- **R-NS-5 Estate opacity.** The core touches an estate repo only through a
  declared `repos/<name>` entry and only to cut a surface. No inference of
  ownership from the current directory (upstream's hard-won rule).

---

## 3. THE ROADMAP RE-SEQUENCED

Ordering principle: **what falsifies the destination loop beats what degrades it,
and both beat what enables a future surface.** The 2026-08-11 dogfood run is the
only measurement of the actual loop and outranks every proposal drafted before it.

### WAVE 0 — the loop does not work without these

| Item | Why here |
|---|---|
| **E6** finalize/promote (currently a no-op) | The only defect that makes the journal report success while the deliverable evaporates. Step 5 of the destination is a lie until this lands. |
| **E2** PATH/toolchain parity for daemon-launched subprocesses | Produced a zero-deliverable run and a self-misdiagnosis. The product must own its env contract; the operator's share of blame does not make it not-a-defect. |
| **E1** cost signal on canceled turns + **E7** `sgt work transcript` | "Walk away" requires a bounded bill and a readable record. With E1 open, an unattended run is financially unbounded (compounds L16/L17). |
| **Blocked exit-door fault-injection test** | The record proves `begin_retry` accepts `Blocked`; it does not prove an operator can walk out a work parked by *ambiguous recovery evidence*. Upstream bled fifteen issues on this exact shape. |

### WAVE 0.5 — E3 interactive hold

Ecosystem seat wants E3 first; product seat wants E6 first. **E6 wins**: E3
falsifies one workflow class (2 of 35 admitted packages), E6 falsifies all of
them. But E3 goes immediately next, ahead of onboarding, because U-R4's
destination sentence explicitly includes *"passing messages back and answering
them"* — the conversational half of the loop is E3-shaped, and E3 additionally
unblocks G5, the WORKFLOW-IF-E3 packages, T-series's whole attention model, and
R-H0-7. One capability, four consumers.

### WAVE 1 — day-one onboarding (strictly after Wave 0; product seat Q5, upheld)

Prebuilt-binary install path · `sgt init` (bootstrap + doctor reuse) · `AGENTS.md`
rewritten as canonical front door · `README.md` recentered on clone-and-trust ·
operator-skills layer established (`sergeant-help`'s home) · `CLAUDE.md`→
`AGENTS.md` symlink · fake-backend fidelity review (the suite's credibility is the
foundation every other claim rests on) · library re-homing (retire
`respond-to-worker`, absorb `sergeant-setup`'s prereq stage into doctor).

Onboarding a stranger into a loop that discards its output is worse than no
onboarding. This is the single most important sequencing call in the document.

### WAVE 2 — the estate

`repos/` manifest + estate-root data-dir default (U-R2's flip off XDG) · per-repo
instruction contract (§4.3) · `sgt doctor` estate verification · E5 config
discoverability · E4/`sgt daemon stop` namespaced with `sgt fleet drain`
(delta #1, engine-gap G4 — genuinely core: a race-safe admission gate) ·
**live-turn stall detection** (delta #2, promoted out of "later" because #46/#47
measured a 45-minute real stall that 352 green tests could not see).
`sgt project list/status/sync` is **not** core — it is a stateless multi-repo git
read, i.e. a skill; only the manifest is tracked state.

### WAVE 3 — queued intents

Backlog as a **separate durable type** with its own event vocabulary
(`backlog.captured` / `backlog.promoted`), not a widened `WorkState` — a state
that never runs would poison every consumer that assumes a Work runs. Promotion
must be idempotent on dedup identity so the L6 adjacent-append window fails
closed. Structured intent (U-R6, delta #7) ships as a field on the same schema.
**Delta #5 (gate/finding routing) is deleted as a separate object** — a review
finding is a queued intent. Dedup keys carry full causal context; short labels
collided across seven upstream issues.

### WAVE 4 — surfaces

T-series **minimal slice only**: Home composer, one legible Work thread with a
working respond action, and #11/#16's measured reliability bugs. Then re-measure
with a second dogfood round before anything else.

### NOT YET (good work, wrong time — each with its unblocking condition)

- **T-series full spec** (five-view Work surface, Attention drawer, `/`+`@`
  grammars, responsive matrix) — *unblock: E3 landed + second dogfood round.*
  Building an attention drawer for a hold-state that cannot occur is decorating a
  circuit that doesn't close.
- **H1 contract-v2 items 1–3+8** (identity/ownership/handles/auth-mode) —
  *unblock: R-H0-3's token-free probe finds a second transport on a real host.*
- **N4 Docker execute stages** — *unblock: Wave 0 closed AND the transport-seam
  question ruled.* Double-blocked: N4 is itself a runtime strategy.
- **G3 durable callback delivery** — *unblock: an actual consumer.* When built,
  model it as a Work with a webhook backend, not a second retry subsystem.
- **G1 wake scheduler** — *unblock: a promotion policy the human wants automated.*
  U-R5 defers it deliberately; note it is the missing half of `WorkState::Waiting`,
  not a new concept.
- **`sgt project graph`** (multi-repo architecture graph) — *unblock: estate
  landed.* Keep the name distinct from `sgt work show --graph`; do not collide.
- **Clean-distro extraction** (separating the distro from the dev repo) —
  *unblock: OS layer stable.*
- **`05-classify-risk`** submission-time safety gate — *unblock: the Work-vs-inline
  routing question is answered (§4.2), since it is the same gate wearing one hat.*

### NOT EVER

- **"Fleet" as a domain object.** U-R4 is right: fleet is a view over all running
  work. `sgt fleet cleanup`'s multi-Work grouping dies with it; `sgt run --repo`
  repeated is the answer.
- **PM semantics** — epics, sprints, assignees, prioritization judgment. U-R5's
  non-ownership list is correct and should be a permanent boundary, not a phase.
- **Operator-specific integrations** — `sgt-graphify`, `sgt-treehouse-init`,
  `wiki-daily-digest`/`wiki-digest`. Hardcoded to one author's paths; not sergeant's
  state; park permanently.
- **Upstream's re-hash-before-every-action intent ceremony** — journal immutability
  already provides the integrity it bought.
- **Reconstructed supervision machinery** — tmux pane identity, response-lock
  files, systemd per-task monitors, `_sgt-harness.sh`'s registry. The daemon *is*
  the supervisor; re-importing these re-imports the bugs.
- **External DAG engine** (upstream #131→#132), **`tracing-opentelemetry` bridge**
  (D7), **journal snapshot loading** (B1). Settled; re-litigate only on new evidence.

---

## 4. GAPS UNCOVERED

1. **No install path that is not `cargo build`.** The five-minute promise dies on
   a ~10-minute cold DuckDB compile. No release artifact, no `cargo install` path,
   no measured install time exists anywhere in the record — and no proposal names
   this as work. It is the first thing a stranger hits.
2. **No Work-vs-inline routing judgment.** Dogfood measured the `research` run as
   "inert ceremony for a single-turn shape" — the Work wrapper added audit trail
   and nothing else. Nothing in the record defines what an intent must clear to
   deserve the apparatus. Unowned.
3. **No per-repo instruction contract.** The destination says the harness "knows
   how that repo works." The manifest will declare *which* repos; nothing declares
   *what sergeant reads from* one. `--workspace`/`sergeant.toml` is single-repo and
   undiscoverable (E5).
4. **Self-hosting is ruled but unmeasured.** U-R3 says development work runs as
   sergeant Works "from here on"; zero dogfood run targeted this repo's own
   defects. The recursion that would prove the whole thesis has never executed.
5. **No cross-repo coherence mechanism.** The estate is pitched as a light monorepo
   for joint reasoning, but no Work can span two `repos/` entries' surfaces. The
   central value claim has no implementation and no contract.
6. **No soak evidence.** #19 is open; every backend durability claim rests on 13
   turns and two cascades. "Walk away" implies hours.
7. **Test-suite credibility is unbounded for timing.** 352 green tests coexisted
   with a 45-minute real stall because the fake backend settles at launch. The
   fidelity review is queued, not done.
8. **No unsupervised-run safety envelope.** E1 (no signal on canceled turns) plus
   L16 (guard fires at ledger granularity) plus L17 (stopping a coordinator does
   not stop its effects) means a stranger's walk-away run can overspend with no
   signal and no brake. Strangers have no orchestrator to catch this.
9. **No dedup identity scheme** for backlog items and routed findings, despite the
   upstream collision scar (7 issues) being explicitly cited as the reason for
   full-causal-context keys.
10. **No estate trust model.** "Trust it once" appears in the mission; what trust
    grants, what it scopes, and how it is revoked is unspecified.

---

## 5. TENSIONS RULED ON

*Adjudication items for the owner. Each is my call, not a summary of the seats.*

1. **AGENTS.md drift.** U-R4's premise is confirmed by measurement (11 lines, pure
   ICM plumbing, no product orientation). **Ruling: rewrite, not amend** — the
   current file has no salvageable canonical content.
2. **Canonical vs regeneratable `AGENTS.md`.** **Ruling: split.** `AGENTS.md` is
   hand-authored doctrine; `.sergeant/index.md` is generated discovery. A
   `repo-to-icm` pass may never write `AGENTS.md` (R-NS-2).
3. **Depot vs first mate.** **Ruling: layering, not conflict** — but retire "Depot"
   as a live name; the identity drift is caused by two names for one thing.
4. **"Inert ceremony" vs go-through-sgt-always.** **Ruling: the OS layer owns the
   routing judgment.** `AGENTS.md` teaches the harness when *not* to use `sgt`;
   the engine does not grow a heuristic. Judgment content out, per R-NS-1.
5. **E6 vs E3 priority.** **Ruling: E6 first, E3 immediately after.** E6 falsifies
   every run; E3 falsifies one class but unblocks four consumers.
6. **E2 as "operational polish."** **Ruling: overturned — E2 is a falsifier.** It
   produced a zero-deliverable run. The operator's shared blame is exactly the
   point: the product must surface its env contract.
7. **Onboarding in parallel with the E-list.** **Ruling: strictly sequenced after
   Wave 0** (product seat Q5, upheld). Inviting strangers into a loop that
   discards output converts a rough edge into a reputational defect.
8. **Delta #5 (gate) vs delta #8 (backlog).** **Ruling: merged.** A review finding
   is a queued intent under U-R5's own definition; a separate `gate` object
   duplicates ordering, dedup and provenance for nothing.
9. **Backlog as new type vs widened `WorkState`.** **Ruling: new type**, with
   idempotent promotion keyed on dedup identity to survive L6. The cheap reuse
   would force every Work consumer to special-case a state that never runs.
10. **Review output: engine state or workflow content.** **Ruling: split at
    identity** — vocabulary and severity are content; dedup key, ordering and
    provenance are engine (R-NS-1).
11. **`sgt project status/list/sync` as VERB-CANDIDATE.** **Ruling: overturned.**
    The function map's own SKILL verdict for `sgt-sync` applies to its siblings;
    only the manifest is tracked state. The map is an inventory, not a boundary.
12. **T-series "primary interactive surface."** **Ruling: corrected to secondary**
    per U-R4 before the T-series contract is written. The claim overstates the
    ruling it is executing.
13. **T-series scope.** **Ruling: cut to the minimal slice** (composer, thread,
    respond, #11/#16). ~1,900 lines of spec for a client that changes no engine
    semantics, gated on a hold-state E3 has not built, is premature by evidence.
14. **R-H0-2's "Option B-lite" split.** **Ruling: inverted by evidence.** The
    orchestrator recommends landing items 1–3+8 (identity/ownership/handles/auth)
    now and deferring 4–7. But the items Cerberus actually measured live are #7
    (protocol-derived capability withdrawal on 2.1.227) and #4 (turns ending
    unsettled) — and #7 is a defect in the *one* transport that exists today.
    1–3+8 are type-level refactoring for transports not yet probed on any host
    (R-H0-3 open). Land #7's capability-provenance fix with the E-list; defer the
    rest behind the probe.
15. **N4 sequencing.** **Ruling: behind Wave 0 and behind the seam question.** The
    scope draft and the ecosystem seat agree; dogfood is the deciding evidence.
16. **D2's status.** **Ruling: `claude -p` is the admitted default transport and
    the North Star commits to it — explicitly provisional**, pending an admission
    suite. "Confirmed at M4" is current best measurement, not permanent closure.
17. **Upstream #52's philosophy ("workflow/process concern, no code change").**
    **Ruling: rejected for the product layer.** The destination ships to strangers
    who have no process. A defect resolvable only by operator discipline is a
    product defect. (Accepted only for the orchestration layer, which has an
    operator by construction — and L15/L16/L17 show even that is generous.)
18. **U-R3 ahead of its evidence.** **Ruling: uphold the ruling, add an acceptance
    test** — the first Wave 0 fix (E6) must itself be run as a sergeant Work
    against this repo. If the recursion cannot carry its own core's first fix, the
    thesis is wrong and we should learn it now, cheaply.
19. **Proposals drafted before dogfood.** **Ruling: dogfood outranks both** T-series
    and H-series scope sections. Not a defect in either's reasoning — a timing
    fact this document corrects.
20. **"Five minutes" vs `cargo build --release`.** **Ruling: the destination
    sentence wins and the build must change.** A prebuilt binary is a Wave 1
    requirement, not a nicety; nothing else in §1 matters if step 1 costs ten
    minutes.