# MVP bucketing + post-MVP roadmap — 2026-08-11

Owner rules applied: (1) sequence by layer ladder — core → adapters →
CLI → stabilize → content — then ship to colleagues; (2) **never defer
something cheap just because it isn't direct-MVP** — small code now
that sets up later beats a bloated post-MVP backlog; (3) enhancements
(P2, TUI) gate on the MVP, never block it. Supersedes the North Star's
thematic waves; NORTH-STAR.md amended same day to cite this.

## The happy path this plan builds

`gh repo clone` → `cargo install sergeant` → `sgt init` → open Claude →
"I'd like to work on the payments api" → the harness (AGENTS.md-taught)
interviews, writes the manifest, `sgt repo add` populates the working
set → intent shaped, workflow picked, `sgt run` → actors work in
worktrees consuming each repo's own AGENTS.md (manifest policy) →
Docker execute stages verify the work honestly → you close the laptop →
you return: `sgt status`, the work finished or blocked-with-reason,
`sgt work show` names the branch and where every artifact lives,
`sgt work transcript` reads the conversation, turn budget enforced
throughout. No tangle.

## MVP-1 — CORE

| Item | Notes / provenance |
|---|---|
| Estate manifest parse (`sergeant.toml` `[estate]`/`[[repo]]`/`[group]`) + pin-at-bind policy snapshot | design settled in-session; #47-style fail-closed parse; atomic writes; advisory lock for sgt's own pens |
| Output pointer at terminal state | E6-as-corrected: retention already works (surface.rs), the pointer is the missing half |
| `promote`/finalize disposition executes | the other half of E6 |
| **Turn-count envelope at PREPARE/LAUNCH** (adapter-neutral) + cost cap where the adapter reports usage | resolves E1 honestly: core counts turns (always possible); dollars are an adapter capability. L16 arithmetic stated in the contract |
| Structured-intent schema slot (optional fields, progressive elaboration) | U-R6; **cheap-now**: sets up the post-MVP backlog type without building it |
| Blocked exit-door fault-injection invariant | upstream's 15-issue scar; every fail-closed state proves its exit |
| Multi-repo binding: **measure `--repo ...` today first**, then group-expansion design ruling | the cross-repo value claim; measurement before design |
| Terminal-work eviction (retention Rule A) | rides N4 per the adjudicated ruling |

## MVP-2 — ADAPTERS

| Item | Notes |
|---|---|
| Claude: `--setting-sources` de-leak → manifest `instructions` policy translation | the boundary fix; small |
| Claude: capability provenance (contract-v2 item 7) | the one cv2 item measured live (2.1.227 withdrawal) |
| Claude: partial-usage on interrupt — measure what the CLI can emit; document what it can't | fact-finding, not a promise |
| Docker executor (N4 per its adjudicated contract; Rule B disk measurement + #23 doctor disk check ride along) | the verification stage that earns walk-away; contract draft ready, prerequisites landed |
| Fake backend fidelity: deferred-finish + timing shapes (R-H0-7) | required for MVP-4's suite to be credible about time |

## MVP-3 — CLI

`sgt init` (scaffold estate, gitignore, doctor) · `sgt repo add/remove/list` ·
`sgt group add/remove/list` (member validation, mkdir-p semantics) ·
`sgt run --group` expansion · `sgt work transcript` (minimal blob decode) ·
output-pointer surfacing in `work show`/terminal output · doctor estate
checks with named remedies · data-dir default flipped estate-resolved
(U-R2) · E5 config discoverability · **cheap-now**: `sgt daemon stop`
(E4), admission drain flag (one journaled event pair + submit refusal),
perf-harness commit pin (#50) + harness gaps (#13).

## MVP-4 — STABILIZE (S-series pattern on the assembled product)

Perf re-baseline of the assembled core (R-N0-4 budgets) · coverage wave ·
**the real #19 soak at last** (multi-hour, real Claude, envelope-guarded,
Docker verify stages — the walk-away promise measured) · #45 flake killed ·
#22 workspace-edge tests · repeated-run gates. Exit = numbers worth
publishing to colleagues.

## MVP-5 — CONTENT (human-usable)

AGENTS.md rewritten (routing table + standard loop, consuming the **126
adjudicated invariant units**) · `CLAUDE.md → AGENTS.md` symlink; dev
rulebook stays as repo content per clone-is-distro · 5 operator skills
ported against real verbs · 17 worker-bundle dispositions
(port/superseded/both) · library re-homing executed (re-triage + sweep
verdicts; #53 and #57 fixed in passing) · helper units (207+23) consumed
where earned · worker-brief template folds into intent-elaboration skill ·
README recentered on clone-and-work · the payments-style walkthrough as
the demo script. **Exit = send it to colleagues.**

## Post-MVP roadmap (enhancers, in order)

1. **P2-JOURNAL** (first post-MVP milestone; first self-hosted *milestone*; its 1M-event run double-pays retention Rule C) 
2. **T-series minimal slice** (composer, thread, respond, #11/#16) → dogfood round → T-full on evidence 
3. **Backlog/queued-intents type** (two-state, dedup keys, finding routing — schema slot already landed in MVP-1) 
4. Live-turn stall detection (hang class; BS2 already covers process-death) · G3 callbacks (on a consumer) · G1 scheduler (on a policy) 
5. Release pipeline/prebuilt binaries + stranger onboarding (envelope exists by then) · clean-distro extraction 
6. H-series remainder (gated on R-H0-3 probe — run the token-free probe during MVP as background fact-finding) · N5 platforms (#18) 
7. Perf/backlog issues as budgets: #6 #7 #8 #10 #12; B2/#15 cookie handoff on non-loopback need; #21 dashboard tests; #26 pty window.

## Explicitly closed by this plan

E1 (re-scoped: turn envelope core, dollars adapter) · E2 (env contract, MVP-2/3) ·
E3 (dissolved by R-NS-6) · E4/E5/E6/E7 (MVP-3 + core) · "honest cost" as
universal promise (demoted to adapter capability) · fleet-as-object,
PM semantics, intent re-hash ceremony, tmux supervision (NOT-EVER, unchanged).
