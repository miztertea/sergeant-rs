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

*Sergeant is an AgentOS distro — a cloned directory of instructions,
skills and conventions that turns a general-purpose coding harness into
an operator of your estate — carried by `sgt`, a durable
intent-execution engine that runs those intents to completion in
isolated worktrees whether or not anyone is watching.*

The finished loop: clone → `sgt` on PATH → `sgt init` (trust, estate,
`repos/` working set) → open your harness → say *"let's work on the api
bug"* → the harness shapes a structured intent, picks a workflow, drives
the CLI on your behalf → walk away → return to a finished change on a
retained branch, a readable transcript, and an honest, bounded cost.
Acceptance: a stranger reaches that last step in under five minutes of
setup. (Stranger onboarding itself is gated — see Waves.)

## Ownership

- **Core (`sgt`)** owns durable execution: journal, blobs, projections,
  the Backend boundary, WorkState, the API, admission, holds and message
  delivery to running executions, promote/finalize, backlog identity and
  provenance, the intent schema, the spend envelope.
- **OS (AGENTS.md, skills, workflows, conventions)** owns judgment and
  dialogue: how to talk to sergeant, what deserves the Work apparatus,
  how to ask and guide. AGENTS.md is hand-authored canonical doctrine;
  generated catalogs are disposable.
- **Estate (`repos/`, manifest, per-repo instructions)** owns the working
  set. `repos/` is a mount, never a dev_root; nothing of ours lives at
  `~/`; machine-local truth is in-estate and gitignored.
- **Surfaces (CLI, TUI, dashboard, harnesses)** own presentation and
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

## Gaps the record must close (owned by the MVP plan's buckets)

Install path, Work-vs-inline routing judgment (OS-owned), per-repo
instruction contract, the recursion proof (self-hosting measured, not
pledged), cross-repo Work spanning estate entries (the central value
claim — currently unimplemented and uncontracted; MVP-1's
group-expansion ruling must produce its design), soak evidence (#19),
fake-backend timing fidelity, unsupervised-run safety envelope, backlog
dedup scheme, estate trust model.

## Rulings index

Draft §5's twenty rulings stand as written except: #14 upheld in its
inversion of R-H0-2; #20's prebuilt-binary urgency overturned (gated
with onboarding); E6 reframed per D1; E3 dissolved per R-NS-6. Owner
rulings U-R1..R6 stand as amended in
`docs/gauntlet/notes/u-series-scope-draft-2026-08-11.md`, which this
document supersedes as the citation root.
