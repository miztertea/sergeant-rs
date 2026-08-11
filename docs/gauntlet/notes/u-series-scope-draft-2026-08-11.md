# U-series scope draft — 2026-08-11

## Adjudicated during the grill (owner rulings, 2026-08-11)

- **U-R1 — the clone IS the distro.** The sergeant-rs development repo
  doubles as the AgentOS estate root for now (upstream doctrine: "the
  cloned repo is the distro"); clean-distro extraction becomes a named
  later milestone once the OS layer stabilizes. Once the core is built,
  most development IS skills/workflows/helpers — content, not code.
- **U-R2 — nothing of ours lives at `~/`.** The estate is self-contained:
  tracked (AGENTS.md, skills, workflows, estate/repos manifest, helpers);
  in-estate but gitignored machine-local truth (`.sergeant/data/` —
  journal, blobs, surfaces, daemon.lock — and the `repos/` entries). The
  current `~/.local/share/sergeant` XDG default (src/cli.rs:225) is an
  unexamined M-era habit: U flips the default to estate-root-resolved
  `.sergeant/data/` with `SGT_DATA_DIR`/`--data-dir` as overrides.
  **Amended after owner pushback (same grill): `repos/` is a WORKING
  SET, not a dev_root.** Upstream's `~/Dev` (the permanent home of all
  clones) is a different concept sergeant neither adopts nor replaces —
  where canonical clones live is not sergeant's business. `repos/` is
  the estate's mount point for repos actively being worked on; an entry
  may be a full clone, a worktree of a clone elsewhere, or a symlink —
  sgt requires only that `repos/<name>` is a git repo it can cut work
  surfaces from. The estate manifest declares the expected entries;
  `sgt init`/doctor verify or populate them.
- **U-R3 — self-hosting starts now.** Development work that fits a
  surviving workflow runs as a sergeant work from here on (research,
  diagnose-bug, code-review, validate-and-ship lanes); multi-agent
  gauntlet fan-outs stay harness-side until the composition question
  earns an answer (N2's adjudicated non-gap stands). Friction found
  while self-hosting feeds the E-list.

Draft for owner adjudication, not a contract. Consolidates today's two
Cerberus deliverables — the execution-surface re-triage
(`docs/icm/retriage-2026-08-11.md`) plus the absorbed-by-engine sweep run
against it — and the dogfood gauntlet's product-fitness findings
(`docs/gauntlet/runs/dogfood-2026-08-11/run-manifest.md`) into candidate
scope for the milestone after H-series lands. Every candidate below is
proposed, not committed; §4 lists what genuinely needs the owner's call.

## 1. Mission

`reference/sergeant-upstream/README.md`'s genesis paragraph: Sergeant is
"an agent distro: a cloned directory with an `AGENTS.md`, shell toolbelt,
and skills that turn a general-purpose agent into a project-aware first
mate." No install — the cloned repo *is* the distro. sergeant-rs kept the
name and the daemon/journal/API architecture but has not yet re-centered
its own docs on that identity: `README.md`/`AGENTS.md` describe a Rust
service, not an AgentOS a human clones and lives inside.

The owner's estate model completes the picture the old bash tool only
partially built: **clone repo → trust prompt → `repos/` mount dir → the
estate as a light monorepo** (frontend/backend/API coherence across
repos that are separately versioned but jointly reasoned about). Old
Sergeant's `~/.config/sergeant/<project>.yaml` + `~/Dev/<project>/`
mental model was the first cut at this; sergeant-rs's `--workspace`
flag and `sergeant.toml` profile discovery are a second, narrower cut
(one repo, not an estate). Neither is the destination. U-series' mission
candidate: make sergeant-rs the thing its own README already claims to
be — clone it, trust it once, and it owns an estate of repos as a single
coherent surface, with the engine (daemon/journal/API) as the load-bearing
core and everything else — docs, skills, CLI verbs — built to make that
core legible and operable.

## 2. Scope candidates

### (a) README + AGENTS.md recentered on the OS (rung: owner framing, source: this session)

The binary is the core; the repo is the product. Rewrite `README.md`'s
opening to lead with "clone this, trust it, and it runs your agent work"
rather than architecture-first prose; `AGENTS.md` (currently the
`.sergeant/` discovery stub, `docs/icm/convention.md` §3) gets a sibling
top-level orientation pass so a harness landing in a *fresh* clone (no
`.sergeant/` yet) still finds the deployment flow. Doc-only; no code.

### (b) The `repos/` estate mount + `sgt init` (rung: implied by deployment flow, source: mission + `sergeant-setup` sweep)

The deployment flow (clone → trust → launch) implies a verb that does not
exist: `src/cli.rs`'s `Command` enum has no `Init`. Today's sweep found
`sergeant-setup`'s `00-detect-prerequisites` (git presence, claude CLI +
version gate) is **already absorbed** — `sgt doctor`'s `git_check`/
`claude_check` do exactly this. What is missing is the bootstrap verb
itself: establish the `repos/` mount convention (sergeant-rs's analog of
old Sergeant's `dev_root`) and a `sgt init` that gets a fresh clone from
"nothing" to "`sgt doctor` reports healthy," reusing doctor's checks
rather than re-implementing them. This is the one clear NET-NEW-SURFACE
verb from the sweep with no engine collision.

### (c) E1–E7 from the dogfood run (rung: measured defect, source: `run-manifest.md`)

Ranked by what the three real runs actually hit:

1. **E1** — a canceled turn records zero `usage.updated`; budget guards
   have no signal on interrupted turns.
2. **E2** — PATH/toolchain parity for daemon-launched actor subprocesses
   (actor couldn't find `cargo`, self-misdiagnosed as a permissions fault).
3. **E3** — submission-time capability gating: a workflow submits cleanly
   on a host where its core affordance (the ask-grammar hold) is
   withdrawn after the first turn. See (d).
4. **E4** — daemon lifecycle verbs (`sgt daemon stop`); a read-only
   client silently respawned a just-killed daemon. Namespace with
   `drain-fleet`'s verbs (e) — same "operator controls the daemon" surface.
5. **E5** — config/profile discoverability (`sergeant.toml` shape
   undiscoverable; `--profile` not implied; doctor silent outside the
   workspace).
6. **E6** — Layer-4 finalize/promote unimplemented (§1a's disposition
   rule): every `promote` disposition is a no-op today; deliverables die
   with the worktree unless hand-rescued.
7. **E7** — transcript legibility: no `sgt work transcript`; reading an
   actor's turns means decoding blob hashes out of journal payloads by hand.

E1/E6/E7 are journal/engine-legibility gaps; E2/E4/E5 are operational
polish; E3 is a design item (d). All measured against a real backend,
post-BS2 binary — the strongest evidence in the corpus; should anchor U's
contract line items over anything below.

### (d) The E3 interactive-hold design item (rung: engine-gap, source: `docs/icm/convention.md` §2a + dogfood `grilling` run)

The convention already names the gap: interactive (grilling-class)
workflows remain workflows only where the engine can hold their
checkpoints open for a human. The dogfood run measured the cost of not
having it — `grilling` completed two stages *autonomously* in 80 seconds
with zero `needs_input`, "negative value vs plain terminal Claude." The
same gap is engine-gap **G5** in the `sergeant-setup` sweep
(`30-project-interview`'s multi-round interview, narrowed to "a
re-enterable `needs_input` stage" reading its own accumulated answers
back out of durable state) and is why `grilling`/`grill-with-docs` sit at
WORKFLOW-IF-E3 rather than plain WORKFLOW. One capability — a stage that
holds open, re-enters per answer, and reads its own response history —
unblocks three findings at once. Candidate design item, not a contract.

### (e) Surviving NET-NEW-SURFACE verbs, reconciled with T-series §6 (rung: per-package, source: today's absorbed-by-engine sweep + `reference/proposal-tui-t-series.md`)

T-series already owns Fleet as a top-level tab (§6.1, decision T-21):
"the complete Work browser." `monitor-fleet`'s read-only snapshot/
liveness reporting is **absorbed** into that plan plus `sgt status`/
`sgt work list` — no new verb needed there. What T-series does **not**
cover (it is a client, not a mutation surface) and what genuinely has no
engine home yet:

- `sgt fleet drain` / `sgt fleet force-stop` (`drain-fleet`) — engine-gap
  G4, a durable scope-qualified admission block. Namespace with E4's
  `sgt daemon stop`.
- `sgt fleet cleanup`'s multi-repo task slice (`reconcile-and-cleanup-
  fleet`) — blocked on a real domain question: sergeant-rs has no "fleet
  task" grouping multiple `Work` items the way old Sergeant's dispatch
  did; per-repo surface teardown is already absorbed by `recovery.rs`'s
  automatic reconciliation.
- `sgt project graph` (`project-graph`) — a whole-project, multi-repo
  architecture graph, a *different object* from the existing
  `sgt work show --graph` / `/v1/graph/work/{id}` (work-neighborhood,
  journal-seq provenance). Keep both names; do not collide them.
- `sgt project list/status/sync` (`load-project`'s command-shaped slice)
  — needs the estate/`repos/` registry concept from (b) first; today's
  `--workspace` flag is lighter and single-repo, and does not absorb this.
- `sgt review route-findings` / `sgt gate clear` (`route-review-
  findings`) — needs a "gate" domain concept the engine does not have.
- `deliver-external-callback`'s narrowed ack-gate (G3) and `wake-and-
  resume`'s periodic re-evaluation (G1) are engine primitives, not CLI
  verbs — lower priority, no dogfood evidence hit either.
- `wiki-digest` — questionable fit for `sgt` at all (digests external
  wiki sources, not sergeant's own state); recommend parking.

### (f) Library re-homing execution (rung: today's rulings, source: today's absorbed-by-engine sweep verdicts)

Execute the re-triage's adjudicated moves plus today's absorbed-by-engine
findings: retire `respond-to-worker` (fully absorbed into shipped
`sgt respond`); `sergeant-setup`'s prerequisite-detection stage retires
into `sgt doctor` documentation; remaining CLI-SURFACE packages become
either the NET-NEW backlog in (e) or park notes where no domain concept
exists yet; `sergeant-help` moves to the operator-skills layer (g).
`dispatch`'s two owner-deferred slices: `20-prepare-intent` absorbs (the
journal's single-canonical-intent-per-Work invariant gives this for free
— no multi-worktree duplication problem exists here); `05-classify-risk`
is a NET-NEW submission-time safety gate on `sgt run` (keyword-routes
risky intents to a fuller intent path), echoing E3's gating theme with a
different mechanism. `worker-mission` and `recover-stalled-worker` stand
as workflows (this session's sweep) — fleet-family naming is a smell inherited
from the old corpus, not evidence their judgment core is absorbed.

### (g) Operator-skills layer establishment (rung: `docs/icm/convention.md` §2a bucket 3, source: retriage + mission)

Per the deployment flow (clone → `sgt init` → launch harness), the
harness needs a layer it loads directly — never dispatched as Work —
that teaches it to operate sergeant well. `sergeant-help` is the first
tenant. This layer does not exist yet as a place in sergeant-rs's own
repo (it lived at `.agents/skills/` in old Sergeant); U-series should
establish where it lives here (a `skills/` sibling to `.sergeant/`, or
folded into `AGENTS.md`'s own reference set) before more OPERATOR-SKILL
packages need a home.

## 3. Explicit non-goals

- **N4 (Docker execute stages)** — queued behind U. Dogfood evidence is
  blunt: the product is "not human-usable unsupervised today" on its
  *existing* surface (one run produced nothing on an environment gap, one
  was structurally broken on this backend). A new execute-stage kind
  before E1/E2/E6/E7 and the AgentOS framing land would compound the gap,
  not close it. H0's R-H0-2 sequencing question is upstream and
  unresolved — this draft does not pre-empt it, only notes U's evidence
  argues the same direction.
- **H-series machinery beyond E3** — the harness-adapter kickoff packet
  (`docs/gauntlet/notes/h0-adjudication-packet-draft-2026-08-11.md`) is its
  own track; U-series takes only the E3 interactive-hold item because
  dogfood measured it against three promoted workflows directly.
- **Anything T-series already owns** — Fleet-tab navigation, the
  attention drawer, responsive composition, the read-only workflow
  catalog route. U-series proposes CLI/engine primitives T-series's own
  §5.2 non-goals leave open; it does not re-litigate T-series's UI
  decisions unless the owner explicitly folds a UI question in here.

## 4. Open questions for the owner

1. Does the estate/`repos/` model (b) get its own contract before or
   alongside `sgt init`, or is `sgt init` scoped narrow (bootstrap
   checks only, no project registry) with the registry deferred again?
2. Is "fleet" (multiple `Work` items grouped as one task) a domain
   concept sergeant-rs should adopt at all, or is per-repo `Work` plus
   `sgt run --repo` (repeatable) the permanent replacement — this
   decides whether `sgt fleet cleanup` and `route-review-findings`'s
   gate concept are buildable as scoped or need a bigger domain change
   first.
3. Does E3's interactive-hold (d) ship as its own U-series contract
   item, or does it wait on H-series's ask-grammar work landing first
   (R-H0-7 already touches fake-backend ask-grammar semantics)?
4. Priority order among (c)'s E1–E7 — this draft ranks by dogfood
   evidence, not by build cost; the owner may weigh E6 (finalize/
   promote) higher since it is the only one that can silently destroy a
   completed run's output today.
