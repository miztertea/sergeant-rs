# MVP bucketing + post-MVP roadmap — 2026-08-11

Owner rules applied: (1) sequence by layer ladder — core → adapters →
CLI → stabilize → content — then ship to colleagues; (2) **never defer
something cheap just because it isn't direct-MVP** — small code now
that sets up later beats a bloated post-MVP backlog; (3) enhancements
(P2, TUI) gate on the MVP, never block it. Supersedes the North Star's
thematic waves; NORTH-STAR.md amended same day to cite this.

**v2, same day** — review-pipeline findings applied inline and
dispositioned at the bottom; three items escalated rather than decided.

## The happy path this plan builds

`gh repo clone` → build and put `sgt` on PATH per the clone-and-work
README (clone-is-distro; the crate is `sergeant-rs`, no crates.io publish
is in MVP scope, prebuilt binaries gated post-MVP) → `sgt init` → open
Claude → "I'd like to work on the payments api" → the harness
(AGENTS.md-taught) interviews, writes the manifest, `sgt repo add`
populates the working set → intent shaped, workflow picked, `sgt run` →
actors work in worktrees consuming each repo's own AGENTS.md (manifest
policy) → a Docker execute stage verifies the work honestly → you close
the laptop → you return: `sgt status`, the work finished or
blocked-with-reason, `sgt work show` names the branch and every
artifact's home, `sgt work transcript` reads the conversation, turn budget
and wall-clock ceiling enforced throughout. No tangle. **What it does not
offer** (R-NS-6, owned honestly): an actor hitting genuine mid-run
ambiguity has no hold to land in — it finishes on its best reading or
fails closed, and you restart with a clarified intent.

## MVP-1 — CORE

| Item | Notes / provenance |
|---|---|
| Estate manifest parse (`sergeant.toml` `[estate]`/`[[repo]]`/`[group]`) + pin-at-bind policy snapshot | design capture v2; #47-style fail-closed parse; atomic writes; advisory lock for sgt's own pens. Two v2 preconditions: the `[workspace]`→`[estate]` schema-migration fork (ESCALATED there) and estate discovery walking past inner `.git` boundaries (`workspace.rs:206-231` finds the member repo, not the estate) |
| Output pointer at terminal state | E6-as-corrected: retention already works (surface.rs), the pointer is the missing half |
| `promote`/finalize disposition executes | the other half of E6 |
| **Turn-count envelope at every turn-producing `Backend` verb** (adapter-neutral) + per-turn wall-clock ceiling + cost cap where the adapter reports usage | **not PREPARE/LAUNCH only**: `launch` spawns turn 1, `send` spawns every subsequent turn outside prepare/launch (`claude.rs:1382-1454`), so a launch-boundary check caps new attempts, not a running conversation. Re-scopes E1 to turn count (always measurable); does **not** fix the canceled-turn zero-`usage.updated` gap, which stays open as MVP-2's adapter item. Wall-clock ceiling is the same knob answering the soak's hang bound (arbitration CUT 10). L16 arithmetic stated in the contract |
| Structured-intent schema slot (optional fields, progressive elaboration) | U-R6; **cheap-now**: sets up the post-MVP backlog type without building it. The slot lands inside immutable journal events (`api.rs:1087` embeds the whole `Work`), so the contract states an additive-only evolution rule and rules explicitly on whether U-R5's promotion-provenance and dedup-identity fields are reserved now or added later — dedup identity is the corpus's highest-correction-cost item (seven upstream collision issues) |
| Submit-time capability preflight — `sgt run` refuses a workflow declaring an ask stage when the resolved backend's `Capabilities::ask` is low | E3's surviving two-line form (dispositions D1; arbitration CUT 2), distinct from R-NS-6's dissolution of the WORKFLOW-IF-E3 *category*. Honest bound: `Capabilities.ask` (`backend/mod.rs:159`) is declared, and this host's withdrawal is only detected after a turn ends (`claude.rs:193`) — the preflight bites on a statically-low capability now, and on a withdrawn one once MVP-2's provenance persists the measurement |
| Blocked exit-door fault-injection invariant | upstream's 15-issue scar; every fail-closed state proves its exit |
| Multi-repo binding: **measure `--repo ...` today first**, then group-expansion design ruling | the cross-repo value claim; measurement before design. This ruling gates MVP-3's group verbs |
| Terminal-work eviction (retention Rule A, **#4**) | **ESCALATED placement.** The ruling is adjudicated and places Rule A "IN N4", but the mechanism — in-memory projection eviction + journal re-derivation — is pure core needing no Docker, and its acceptance vehicle (s2-churn's RSS slope) exists today. Plan's position: land it standalone here, and have N4's adjudication pass drop it from that contract's Outcome. Owner confirms, since this amends an adjudicated ruling |

## MVP-2 — ADAPTERS

| Item | Notes |
|---|---|
| Claude: `--setting-sources` de-leak → manifest `instructions` policy translation | the boundary fix. Two v2 riders: mixed `instructions` across a multi-repo bind fails closed (one launch, one flag — `claude.rs:874-881`, `surface.rs:152-161`), and what `local` translates to is measured before it is specified (L1) |
| Claude: capability provenance (contract-v2 item 7) | the one cv2 item measured live (2.1.227 withdrawal); it is also the durable input MVP-1's preflight needs |
| Claude: partial-usage on interrupt — measure what the CLI can emit; document what it can't | fact-finding, not a promise. This is where E1's canceled-turn telemetry gap actually lives |
| Docker executor (N4) — **owner adjudication of the contract is a gating step of this bucket, not a completed fact** | `contracts/N4.md:1` is titled "DRAFT, not adjudicated" and carries three open owner asks (retention embeddings, the #44 gate, model spread). What *is* settled: both landing preconditions — #44 shipped, retention ruling adjudicated. Rule B disk measurement + **#23** doctor disk check ride along. Sequence: adjudicate N4 → build |
| One promoted workflow gets a real `kind = "execute"` Docker stage, end to end | without this MVP-4's "Docker verify stages" soak has no subject: every promoted candidate still reads "No `kind = \"execute\"` stage exists in the current engine" (e.g. `dispatch/15-check-admission`, `to-tickets/20-confirm-breakdown`). Leading candidate `validate-and-ship`'s gate-drive step — the gate invocation is deterministic, only finding-relay is judgment — fixed at contract time, matching N4's actor→execute→actor proof shape |
| Fake backend fidelity: deferred-finish + timing shapes (R-H0-7) | required for MVP-4's suite to be credible about time |

## MVP-3 — CLI

`sgt init` (scaffold estate, gitignore, doctor) · `sgt repo add/remove/list` ·
`sgt group add/remove/list` + `sgt run --group`, **both gated on MVP-1's
group-expansion ruling** (until then a harness expands `[group.*]` into
repeated `--repo` flags with zero new surface, `cli.rs:75-77`) · `sgt work
transcript` (minimal blob decode) · output-pointer surfacing in `work
show`/terminal output · doctor estate checks with named remedies ·
data-dir default flipped estate-resolved (U-R2), **new client-side code**
— `resolve_data_dir` (`cli.rs:212-230`) reads only flag/env and cli.rs
imports nothing from `domain`, so it gains its own estate-root discovery
before any daemon exists · E5 config discoverability · **cheap-now**:
`sgt daemon stop` (E4) plus the admission drain flag scoped to exactly
that use — one journaled event pair + submit refusal so `daemon stop`
refuses new admissions while in-flight work finishes, nothing broader
(CUT 10 is right that one-owner retires upstream's multi-actor drain,
which upstream retrofitted three times) · perf-harness pin (#50) +
harness gaps (#13).

## MVP-4 — STABILIZE (S-series pattern on the assembled product)

Perf re-baseline of the assembled core (R-N0-4 budgets) · coverage wave ·
**the real #19 soak at last** (multi-hour, real Claude, envelope-guarded,
the MVP-2 Docker verify stage exercised — the walk-away promise
measured). Its hang bound is MVP-1's per-turn wall-clock ceiling: a
stalled-but-alive turn cancels and journals, because `recovery.rs` covers
restart only, never a daemon-resident staleness detector (function-map
DELTA #2) · #45 flake killed · #22 workspace-edge tests (they carry the
estate-discovery fixtures too) · repeated-run gates. Exit = numbers worth
publishing to colleagues.

## MVP-5 — CONTENT (human-usable)

AGENTS.md rewritten (routing table + standard loop) — its invariant input
is the **126 `agents-invariant` candidates from N2 run 4, "listed, not
drafted" (`n2-run4/run-manifest.md:152`) under a re-triage "awaiting owner
spot-check"**, so drafting plus that spot-check are MVP-5 work, not an
assumed-done precondition · `CLAUDE.md → AGENTS.md` symlink; dev rulebook
stays repo content per clone-is-distro · operator skills ported against
real verbs: **1 measured today** (`sergeant-help`, the sole OPERATOR-SKILL
verdict in `icm/retriage-2026-08-11.md`) plus the grilling-class re-homes
R-NS-6 creates and the function map's two SKILL verdicts (`sgt-sync`,
config/path resolution) — the set is fixed by the reconciliation next, not
asserted here · **execute R-NS-6's reclassification**: retriage still
verdicts `grilling`/`grill-with-docs` WORKFLOW-IF-E3, a category the
ruling calls empty, and the plan is not closed until those verdicts are
superseded in writing · library re-homing executed (re-triage + sweep;
#53/#57 fixed in passing) · helper units (207 `helper` + 23
`shared-helper`, n2-run4 classifications) consumed where earned ·
worker-brief template folds into the intent-elaboration skill, with **any
worker/fleet-shaped upstream content checked against the fleet-as-object
NOT-EVER ruling before it reaches AGENTS.md prose** · README recentered on
clone-and-work · the payments-style walkthrough as demo script.
**Exit = send it to colleagues.**

## Post-MVP roadmap (enhancers, in order)

1. **P2-JOURNAL** (first post-MVP milestone; first self-hosted *milestone*; its 1M-event run double-pays retention Rule C) 
2. **T-series minimal slice** (composer, thread, respond, #11/#16) → dogfood round → T-full on evidence 
3. **Backlog/queued-intents type** (two-state, dedup keys, finding routing — schema slot already landed in MVP-1) 
4. Live-turn stall detection, daemon-resident (the hung-but-alive class; a *process that exits* without an envelope already settles to `blocked` — BS2 Outcome 2's pre-envelope-exit shape — and MVP-1's wall-clock ceiling is the coarse bound, so what remains here is detection + bounded one-shot recovery) · G3 callbacks (on a consumer) · G1 scheduler (on a policy) 
5. Release pipeline/prebuilt binaries + stranger onboarding (envelope exists by then) · clean-distro extraction 
6. H-series remainder (gated on R-H0-3 probe — run the token-free probe during MVP as background fact-finding) · N5 platforms (#18) 
7. Perf/backlog issues as budgets: ~~#6 #7~~ (closed 2026-08-13, grooming pass) #8 #10 #12; B2/#15 cookie handoff on non-loopback need; #21 dashboard tests; #26 pty window. 
8. Estate graph — the home NORTH-STAR already gates for retriage's `sgt project graph` NET-NEW-SURFACE (unblock: estate landed); its naming reconciliation against `sgt work show --graph` is settled there. `sgt project list/status/sync` does **not** get built: `sgt repo list` + `sgt doctor` answer status, and `sgt-sync` is a SKILL by owner pre-ruling — the source packages retire at MVP-5's re-homing.

## Explicitly closed by this plan

E1 (re-scoped: turn envelope core, dollars adapter) · E2 (env contract,
MVP-2/3 — **correction 2026-08-13:** not actually closed; only the
docker-side fix (TH-02) landed, the daemon-launched actor subprocess
still inherits the daemon's PATH unenriched, and #60 stays OPEN in
Wave 0) · E3 (the WORKFLOW-IF-E3 *category* dissolved by R-NS-6; its
cheap remainder, the submit-time preflight, ships in MVP-1 — the blanket
"dissolved" was absorbing a live item) · E4/E5/E6/E7 (MVP-3 + core) ·
**#4** (MVP-1) · **#23** (MVP-2) · "honest cost" as universal promise
(demoted to adapter capability) · fleet-as-object, PM semantics, intent
re-hash ceremony, tmux supervision (NOT-EVER, unchanged). **Left open on
purpose: #17** — Rule C is deferred-with-trigger (measured
rebuild-on-start over 30 s), which P2-JOURNAL's 1M-event run pays.

## Review dispositions (2026-08-11 pipeline)

31 findings from four reviewers; every id below, abbreviated to
`axis:Fn` (the reviewers' slugs — e.g.
`design-reality:F4-turn-envelope-misses-send-verb` — carry the same
numbering). "Applied" means the document changed; escalations are the
owner's.

**Applied (27).**
`dependency:F1` / `ponytail:F1` / `honesty-vision:F3` (N4 asserted
adjudicated — all three verified against `contracts/N4.md:1`; MVP-2 now
makes adjudication a gating step and names the three open asks; note the
*retention ruling* is genuinely adjudicated, so ponytail:F1's reading of
the MVP-1 row was half wrong — corrected to the contract, not the
ruling). `dependency:F3` (no real execute stage anywhere — new MVP-2
item). `dependency:F4` (group expansion stated as settled — marked
provisional in the design capture). `dependency:F5` (live-turn stall vs.
the soak — a per-turn wall-clock ceiling pulled into MVP-1 as the soak's
bound, per arbitration CUT 10's one-mechanism-two-problems argument; the
daemon-resident detector stays post-MVP). `dependency:F6` /
`honesty-vision:F5` (126 "adjudicated" → "listed, not drafted", spot-check
scheduled). `dependency:F7` / `ponytail:F4` (submit-time preflight was
absorbed by the blanket E3 closure — now its own MVP-1 item, with the
honest bound that it needs MVP-2's provenance to catch a *withdrawn*
capability). `dependency:F8` (`sgt project graph` / `list/status/sync`
unhomed — roadmap item 8). `dependency:F9` (#4/#23/#17 uncited).
`design-reality:F1` (discovery never reaches the estate root from inside
a member repo). `design-reality:F2` (`deny_unknown_fields` collision;
position stated, fork escalated). `design-reality:F3` (per-repo
`instructions` vs. one process — fail-closed conflict rule added, `local`
flagged unmeasured). `design-reality:F4` (envelope missed `send` — the
sharpest finding in the set; verified at `claude.rs:1382-1454`).
`design-reality:F5` (`origin`/`base` are new fields and new plumbing).
`design-reality:F6` (`workflow.bound` pins a name, not a policy).
`design-reality:F7` (estate-resolved data dir is new client-side
discovery). `design-reality:F8` (`[profile.*]` → `[[profile]]`).
`ponytail:F5` (schema slot in immutable events — additive-only rule and
the provenance-field question now stated). `ponytail:F6` (drain
undermotivated — scoped to `sgt daemon stop`, CUT 10's objection named).
`honesty-vision:F1` (`cargo install sergeant` is not shippable and
contradicts clone-is-distro; crate is `sergeant-rs`). `honesty-vision:F4`
(5 operator skills → 1 measured plus a named, traceable composition).
`honesty-vision:F6` (BS2 parenthetical — kept but made precise: BS2
Outcome 2 pins the pre-envelope-*exit* shape, which is why a dying
process settles; the hung-but-alive turn is what stays open).
`honesty-vision:F7` (17 worker-bundle dispositions — figure dropped as
untraceable; the fleet-as-object check it asked for is added).
`honesty-vision:F8` (E1 row overstated vs. the closing section).

**Applied in part (2).**
`ponytail:F3` — the argument that `--repo` already expands groups is
correct and now appears in MVP-3, but the full cut is **rejected**: the
owner's capture makes CLI verbs one of the three pens, and rule 2 covers
them. Both verbs are instead gated on MVP-1's ruling.
`honesty-vision:F2` — branch (b) applied: MVP-5 must actually execute
R-NS-6's reclassification (retriage's WORKFLOW-IF-E3 verdicts are still
standing), and the happy path now states the mid-run-ambiguity outcome
plainly. Branch (a), re-opening E3, is **rejected**: the dogfood
measurement (`grilling` completing 2/2 in 80 s with nowhere for the
human's answer to land, "negative value vs plain terminal Claude")
*supports* the ruling rather than undermining it. The sycophancy charge
is noted and does not survive its own evidence — but the unexecuted
reclassification was a real hole.

**Rejected (2 branches, none whole).** The two rejections are the
branches named inside the partials above — `ponytail:F3`'s full cut of
the group verbs, and `honesty-vision:F2`'s re-opening of E3. No finding
was dropped without a reason recorded here. Tally over the 31 ids: 27
applied + 2 applied-in-part + 2 escalated-only (`dependency:F2`,
`ponytail:F2`) = 31.

**Escalated (3) — owner decisions, not the writer's.** Two of the three
sit inside findings already applied; the fork is what escalates, not the
correction.
1. **Rule A's home** (`dependency:F2` / `ponytail:F2`). Both reviewers
   found the same real contradiction: a CORE row riding an ADAPTERS item.
   The plan takes a provisional position (land eviction standalone in
   MVP-1; N4's adjudication drops it from that Outcome) because nothing
   in the mechanism needs Docker — but this amends an adjudicated ruling,
   so the owner confirms or sends it to MVP-2 with Rule B.
2. **The manifest schema migration** (`design-reality:F2`).
   `[workspace]`→`[estate]` and `[[repository]]`→`[[repo]]` as
   renames-with-refusal, or coexisting vocabularies. Both defensible;
   MVP-1's serde work cannot start until it is chosen.
3. **The execute-stage subject** (`dependency:F3`). The item is now in
   MVP-2; which promoted workflow carries the first real
   `kind = "execute"` stage is a content call. `validate-and-ship`'s
   gate-drive step is the leading candidate and is named as such, not
   decided.

## v3 — escalations resolved + outside-review dispositions (owner rulings, 2026-08-11)

**The three escalations, ruled:** (1) Rule A lands standalone in MVP-1;
N4's adjudication drops it from that contract's outcome (owner amends
their own retention ruling). (2) Schema migration: **rename-with-refusal**
— `[workspace]`→`[estate]`, `[[repository]]`→`[[repo]]`, old vocabulary
refused with the migration named; no users, nothing to migrate; our own
fixtures update in the same commit. (3) Execute-stage subject: chosen at
N4 contract time under the owner's criterion — **the demonstration is
container-launch + output-passing through an ICM stage, so pick something
cheap, fast, and runnable constantly** (a validation/lint-class step, not
the no-mistakes gate-drive, which is too heavy for a soak subject).

**Outside review (Codex, `reference/review-northstar-outside-codex.md`),
dispositioned — adopted:**
- **A1 (its sharpest catch): the self-hosting surfaces contradiction.**
  In-estate data-dir puts self-hosted worktrees inside sergeant-rs's own
  checkout, which surface materialization refuses by design. Adopted as a
  named MVP-1 design ruling owed before the data-dir flip: **separate
  `data_dir` from `surfaces_root`** (its option 1 — journal/blobs stay
  in-estate; disposable worktrees land outside or sibling), exact shape
  ruled in the MVP-1 contract.
- A2: two install personas stated explicitly — colleague path is
  `cargo install --path . --bin sgt` (cold build honestly >5 min);
  the five-minute stranger path stays gated post-MVP. Happy path text
  corrected to stop borrowing the post-MVP surface.
- A3: the **instruction projection contract** elevated to a first-class
  MVP acceptance item (which files, composition order for multi-repo,
  conflict handling, pinned identities at bind, per-adapter translation)
  under the rule: manifest declares policy; core resolves and pins;
  adapters translate without redefining.
- A4: manifest ownership split — **tracked logical topology** (names,
  relative mounts, origins, groups, instruction policy) vs **gitignored
  local state** (resolved paths, entry kind, health observations).
  Three-pens stands as owner-ruled, amended: the harness SHOULD drive
  the verbs rather than hand-writing the file (AGENTS.md guidance, not
  a hard rule; sgt remains the validating writer on that path).
- A5: **minimum fake-backend deferred-finish fidelity pulled to the
  front of MVP-1** as a precondition for every turn-boundary contract
  test; the fuller R-H0-7 review stays MVP-2.
- A6: promote/finalize's four possible meanings acknowledged; the MVP-1
  contract must define declaration inputs, disposition semantics, owning
  component, timing vs teardown, and failure behavior before build.
- A7: **the assembled-product ship gate is MVP-5's exit**: fresh clone,
  documented install, two repos registered, fresh harness context,
  intent through AGENTS.md, actor + Docker verify, detach, restart,
  return via status/show/transcript, find branch + outputs — no hand
  edits, no journal decoding, no orchestrator rescue.
- A8: per-bucket self-hosting checkpoints (≥1 MVP-1 change through
  current sergeant; ≥1 MVP-3 integration through the assembled build;
  the soak driven via documented surfaces; the final gate from a fresh
  context). P2 stays "first fully self-hosted *milestone*".
- A9: P2-vs-T-minimal ordering demoted from frozen to a **post-MVP pilot
  ruling** decided by colleague-pilot evidence.

**Rejected from the review, with reasons:** its single-writer-only
manifest doctrine (conflicts with the owner's three-pens ruling;
resolved as SHOULD-guidance above), and nothing else — the rest either
restated our record or sharpened it.
