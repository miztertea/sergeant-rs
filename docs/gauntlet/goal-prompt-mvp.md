# GOAL PROMPT — carry sergeant-rs to the North Star MVP

Give this to any orchestrator session (or resume with it) until the MVP
ship gate passes. Written 2026-08-11 on Cerberus after the North Star
adjudication; supersedes the N-series handoff. Everything it cites is
committed — the repo is your memory, your context is not.

## 1. Orient (read, don't skim — in this order)

1. `NORTH-STAR.md` — the destination and ownership rules. Binding.
2. `docs/gauntlet/notes/mvp-bucketing-2026-08-11.md` (v3, with
   dispositions) — YOUR mission. The buckets are the sequence.
3. `docs/gauntlet/notes/estate-manifest-design-2026-08-11.md` (v2) —
   the manifest design + its escalation resolutions.
4. `reference/review-northstar-outside-codex.md` — the outside seat;
   its five pre-contract rulings are dispositioned in the bucketing v3.
5. `CLAUDE.md`, `GAUNTLET.md` (register + newest entries), `LESSONS.md`
   — all binding. Highest-leverage here: L1 (measure the CLI), L12
   (re-read governing text at decision time), L15 (claims to agents
   carry evidence or say "hypothesis"), L16 (guards fire at ledger
   grain), L17 (stopping a coordinator doesn't stop its effects), L18
   (R1's "already exists" includes the product — keep the engine's
   capability surface beside every classification).
6. `docs/environments/<host>.md` + run `scripts/probe-env.sh` on wake
   (re-measure cheaply; a stale fact file is a rumor). Check `~/inbox/`
   (owner drop-point; vendor-then-delete).
7. `reference/notes/gauntlet-pattern.md` — the loop, the model spread
   (Sonnet executes, Opus judges, Fable orchestrates; Fable seats
   permitted on earned need), the 2026-08-11 revisions (small diffs
   batch into larger panels; independent probers; guard maps).

## 2. Operating rules (the ones that bit us, cold)

- **Model economy (owner, 2026-08-11): Sonnet is the workforce; Opus
  when earned (judgment nothing mechanical can catch); Fable is
  escalation-only, bare minimum. The orchestrator keeps its own tokens
  minimal — workflows do the work; the orchestrator is the project
  manager and guardian of the North Star: contracts, adjudication,
  tie-breaks, and nothing a subagent could do instead.**

- **Contracts before build; the owner adjudicates every contract.**
  Never start a bucket without its adjudicated contract.
- **Code is code (R-S0-12)** — full multi-axis loop for every
  executable diff; batch small diffs into the next larger panel.
- **The epistemic license binds**: every ruling in the repo, the
  owner's included, is a hypothesis with provenance. Evidence outranks
  authority. Push back with citations; ratifying the record is failing.
- **Push after every green gate. Facts into artifacts immediately.**
  Head-PR + sub-PR merge model: one branch per work period PRs to main
  (owner merges); lanes PR into it; worktree-isolate parallel lanes
  with their own target dirs; never leave untracked files in a
  builder-owned checkout.
- **Spend**: turn-envelope thinking everywhere (L16 — a cap bounds at
  cap + one maximal turn). Bounded real-backend runs ~$1 pre-authorized;
  name anything bigger to the owner BEFORE it runs.
- **Self-hosting checkpoints are acceptance, not branding**: ≥1 MVP-1
  change executed as a sergeant Work against this repo (mind the E2
  PATH fact — set the daemon env per `docs/environments/`), ≥1 MVP-3
  integration through the assembled build, the MVP-4 soak driven via
  documented surfaces, the MVP-5 gate from a fresh harness context.

## 3. The mission, in order

**MVP-1 CORE.** Draft the contract first; it must RULE on the owed
items before build: the `data_dir`/`surfaces_root` split (self-hosting
contradiction, outside review A1); promote/finalize's exact semantics
(declaration inputs, disposition, owner, timing, failure — A6); the
schema rename-with-refusal (`[workspace]`→`[estate]`,
`[[repository]]`→`[[repo]]`, fixtures in the same commit); the
instruction-projection contract (manifest declares, core resolves+pins
at bind, adapters translate — A3); multi-repo binding measured before
the group-expansion ruling; intent schema's minimal fields (objective /
repos-or-group / acceptance / exclusions / workflow — optional,
progressive, journaled, pinned); the turn envelope at EVERY
turn-producing verb + per-turn wall-clock ceiling; minimum fake-backend
deferred-finish fidelity as precondition for turn-boundary tests (A5);
Rule A eviction; blocked exit-door invariant; submit-time capability
preflight; estate discovery walking past inner `.git` boundaries.

**MVP-2 ADAPTERS.** Adjudicate N4 (three open asks + drop Rule A from
its outcome), then build: Docker executor; the `--setting-sources`
de-leak (measure what `local` translates to before fixing semantics —
L1); capability provenance persisting measurements (feeds the
preflight); partial-usage fact-finding; one promoted workflow gains a
real execute stage chosen cheap-fast-frequent (validation-class);
full R-H0-7 fake-fidelity review.

**MVP-3 CLI.** `sgt init` · repo/group verbs (group verbs gated on the
MVP-1 expansion ruling) · `sgt work transcript` · output pointer in
`work show` · doctor estate checks · estate-resolved data-dir
(client-side discovery) · E5 · cheap-now: `sgt daemon stop` + scoped
drain flag, #50, #13.

**MVP-4 STABILIZE.** Perf re-baseline (R-N0-4 budgets), coverage wave,
the real #19 soak (multi-hour, envelope-guarded, Docker verify), #45,
#22, repeated-run gates. Exit: numbers worth showing colleagues.

**MVP-5 CONTENT.** AGENTS.md rewritten (routing table + standard loop,
consuming the 126 invariant units), symlink, operator skills ported,
worker-bundle dispositions, library re-homing executed (incl. the
grilling-class re-home to skills; #53, #57 in passing), helper units
consumed, README recentered (colleague install path, honestly >5 min).
**Exit = the assembled-product ship gate**: fresh clone, documented
install, `sgt init`, two repos registered, fresh harness context, "I'd
like to work on X" through AGENTS.md, actor + Docker verification,
detach, restart, return via status/show/transcript to the retained
branch and outputs — no hand-edits, no journal decoding, no
orchestrator rescue. Then it ships to colleagues.

**Post-MVP** (do not start; re-rule on pilot evidence): P2-JOURNAL vs
T-minimal ordered by the colleague pilot; backlog type; stall detector;
G3/G1; release pipeline + strangers; H-series remainder (run R-H0-3's
token-free probe as background fact-finding anytime); N5.

## 4. The claim being proven

> A developer can hand Sergeant meaningful work and stop babysitting it.

Every contract's acceptance section answers to that sentence and to the
North Star's five-minute loop. When in doubt: does this remove a
returning-developer tangle, or decorate one?
