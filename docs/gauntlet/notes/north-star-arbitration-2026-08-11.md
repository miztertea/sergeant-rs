# North Star arbitration — three adversarial challenges, verbatim


---

## Challenge 1

### Attack
**Thesis: the draft has the right ordering principle and applies it everywhere except its largest wave. Finish and harden the core; keep the distro layer at doc-minimum until the core is unimpeachable — on a finite, named list.**

**1. The identity clause inverts what the evidence measured, and the sequencing follows the inversion.**
§1 says "the distro is the product, the engine is a component of it." The one dogfood run that produced real value — `research`, $4.31, artifact "genuinely good and load-bearing" (`docs/gauntlet/runs/dogfood-2026-08-11/run-manifest.md:14`) — is recorded as: "the Work wrapper added audit trail and nothing else over bare `claude -p`." If the distro is the product, that line is an indictment. The honest reading is the reverse: the engine's differentiating value was unreachable because promote is a no-op (E6), the transcript is unreadable (E7), and the cost signal is missing (E1). The draft's own R-NS-1 defines the differentiator precisely as "durability, ordering, race-freedom that a script re-running from scratch cannot honestly provide" — i.e. the core. §1 clause 1 and §2's R-NS-1 disagree, and the roadmap follows §1. Correct identity: *the engine is the product; the distro is its packaging and its first user.*

**2. Wave 1 (onboarding) is the load-bearing error, and the draft's own §4 refutes it.**
Ruling #7's logic — "inviting strangers into a loop that discards output converts a rough edge into a reputational defect" — generalizes past the defect it was written for. Wave 0 closes none of gaps 6, 7 or 8:
- Gap 6, soak: **#19 is open** and the ledger is explicit that it stays open — "adapter evidence, not a completed soak" (`GAUNTLET.md:344`, and again at `:79` after Run B2). "Walk away" is an hours-scale claim resting on 13 turns and two cascades.
- Gap 8, no brake: E1 delivers a *signal*, not a *bound*. L16 is arithmetic, not a bug: "a $2.50 budget guard could never have fired before $4.62, because usage lands once per turn and turn 2 alone cost $3.21 — the guard's floor is the largest single effect." Add L17 (a stopped coordinator's dispatched cancel still landed). Wave 0 + Wave 1 ships strangers a loop that is financially unbounded *by construction*, and strangers have no orchestrator — the draft says this itself in gap 8 and then schedules onboarding next anyway.
The fix is a core item the draft has nowhere: a per-Work spend/turn envelope enforced at the PREPARE/LAUNCH boundary (core by R-NS-1), sized per L16's whole-effect arithmetic, shipping with G4's admission block.

**3. Fake-backend fidelity is misfiled as onboarding — a category error under the draft's own boundary tests.**
Wave 1 lists "fake-backend fidelity review" between the README rewrite and the `CLAUDE.md`→`AGENTS.md` symlink, then calls it "the foundation every other claim rests on." Both cannot be true. BS2's root cause is the sharpest measurement in the corpus: "fake turns settle at launch — a 352-green suite coexisted with a 45-minute stall; one stub parameter, `stalls_for`, was the whole difference" (`GAUNTLET.md:59-63`). Coverage of 94.63% lines (`CLAUDE.md`, `docs/coverage/baseline-2026-08-10.md`) measures executed lines over a backend that could not represent elapsed time. This is instrument calibration and it must precede every item graded by the instrument — including E6, whose regression test will be written against that fake backend. Top of the core wave, before E6, not queued behind doc work.

**4. E6 is right in priority and wrong in ownership, and the draft doesn't notice.**
The governing text specifies finalize as workflow content: "a shared helper invoked by the closing actor stage; it is a canonical execute-stage workload once `kind = \"execute\"` exists" (`docs/icm/convention.md:162-178`). Implementing E6 that way means dispositions edited across 34 promoted packages plus a shell helper — an OS-layer change with no engine test that fails when reverted (L7), landing at the head of a wave the draft insists must precede all OS work. Core-first version: the engine owes the guarantee that a declared `promote` output is captured into the content-addressed blob store and journaled at stage end, surviving worktree teardown *whether or not any workflow's finalize step ran*. `src/runtime/blob.rs` already exists; this is a small engine delta with a clean revert-probe. Landing E6 as library content guarantees it is re-litigated when `kind="execute"` arrives with N4.

**5. Waves 2 and 3 are inverted: backlog is core, estate is mostly OS.**
By R-NS-1, backlog-as-durable-type (dedup identity, ordering, provenance, idempotent promotion) is core; the estate is a tracked manifest plus doctor checks plus per-repo instruction *content*, and ruling #11 already concedes `sgt project list/status/sync` is a skill. So Wave 2 is largely OS with two core items smuggled in (G4, stall detection) and Wave 3 is pure core. Order by irreversibility:
- Dedup identity has the highest correction cost in the document — upstream burned seven issues on colliding short keys (`docs/gauntlet/notes/upstream-lessons-mining-2026-08-11.md:16`, #146/#147/#162/#164/#166/#186/#197), and identity schemes are hardest to change once items exist under them.
- The draft's own ruling #18 makes self-hosting the acceptance test. Self-hosting the E-list requires a durable queue to *hold* the E-list; without it, U-R3's recursion is one demo run, not a loop. Backlog + structured intent (U-R5/U-R6) ships with the E-fixes.

**6. G4 drain is under-specified for the slot the draft gives it.**
Wave 2's one-liner ("genuinely core: a race-safe admission gate") is exactly what upstream shipped first, then retrofitted three times: #68 → #74 → #167, locks → cooperative checkpointing → `--wait`/`--timeout` plus owner/age lock diagnostics, because "the original lock either silently blocked or silently failed open depending on which path was hit" (mining lesson 11). If it is core, its first cut carries bounded wait and lock provenance, in the core wave, next to E4 — not after strangers are already on the daemon.

**7. G3's deferral ("unblock: an actual consumer") is falsified by the destination the draft wrote.**
Step 5 of §1 is "walk away." SSE is "pull-only for connected clients, not a durable retried delivery guarantee" (`docs/gauntlet/notes/upstream-core-function-map-2026-08-11.md:29`). The hosts we actually run on are measured hostile to connected clients — "three resets in one S-series day" (`CLAUDE.md`, Remote-container operations). A walk-away product whose only completion signal requires a live subscriber has no completion signal on its own environment. That buys the narrowed ack-gate the retriage already scoped (`docs/icm/retriage-2026-08-11.md:103`), not a retry subsystem. And reject the draft's implementation aside: modeling delivery "as a Work with a webhook backend" recurses (a delivery Work that fails needs its own delivery notification) and puts unbounded external retry *inside* the state machine that admission control and the spend envelope are supposed to govern. It is an outbox in the journal, not a Work.

**8. Ruling #20 (prebuilt binary in Wave 1) makes an unmeasured claim outrank measured defects.**
Gap 1 concedes "no measured install time exists anywhere in the record." So a Wave 1 *requirement* is derived from a slogan while the stall watchdog — measured at 45 minutes, live — is demoted to Wave 2. That directly violates the draft's own ruling #19 (dogfood outranks pre-dogfood proposals) and its stated ordering principle. Worse, the cost is recurring and external: the CLI version gate is pinned in `src/backend/claude.rs`, and `CLAUDE.md` requires re-measurement on every CLI bump; a stale pinned binary in a stranger's hands is L1 ("exit codes lie, `subtype` lies, model aliases silently substitute") delivered to someone with no orchestrator to catch it. Honest first cut: `cargo install --git` plus a *measured* install number, with the release matrix deferred until promote semantics, the backlog type and the intent schema stop moving.

**9. Ruling #4 (OS owns Work-vs-inline routing) encodes today's engine deficit as permanent doctrine.**
A rule living only in `AGENTS.md` has no test that fails when it is violated (L7), and every stranger's harness re-derives it — upstream's #52 is the precedent the draft itself rejects in ruling #17: a correctness problem closed as a documented convention, which then failed anyway. The `research` run felt like "inert ceremony" *because* promote was a no-op, the transcript unreadable, the cost invisible. Fix E1/E6/E7 first, re-measure, and only then decide whether a routing rule is still needed — otherwise you are writing doctrine against a deficit you are about to remove.

**10. The draft already proves my case twice and then exempts its biggest wave.**
Ruling #14 inverts R-H0-2 on exactly this reasoning — land #7's measured capability-provenance defect with the E-list, defer type-level refactoring for transports no host has probed. Ruling #13 cuts T-series for the same reason. That test, applied consistently, also defers the prebuilt binary, `sgt init`, the estate and the operator-skills layer behind the core list. One principle, one exception, and the exception is the largest wave in the document.

**Concrete counter-sequence (replacing Waves 0–2):**
CORE WAVE (single wave, finite list, no OS investment beyond `AGENTS.md` gaining a paragraph that says what sergeant is): (0) fake-backend timing fidelity + a stall watchdog with a revert-probed test; (1) E6 as an engine durability guarantee; (2) E2 env contract; (3) E1 + the L16-sized spend envelope + G4 admission with bounded wait and lock diagnostics; (4) E7 transcript; (5) blocked exit-door fault injection; (6) backlog-as-durable-type + structured intent with full-causal-context dedup; (7) G3 ack-gate as a journal outbox; (8) one real soak closing #19. EXIT TEST: ruling #18's recursion (the core wave's own fixes run as sergeant Works against this repo) plus a second dogfood round that reproduces the `research` run's value *with* the wrapper now earning its keep. Only then Wave 1 onboarding, then estate, then surfaces.

### Concessions
**Points the draft genuinely wins, and I adopt them:**

1. **E6 before E3 (ruling #5).** Correct, and for the correct reason — E6 falsifies every run, E3 falsifies one class (2 of 35 admitted packages). I dispute only E6's *ownership*, never its rank.
2. **E2 is a falsifier, not operational polish (ruling #6, overturning the scope draft's `:158` classification).** Right, and it is core: upstream's #138 (managed systemd service's PATH omitted `td`, so handoffs "silently weren't recorded" — mining lesson 7) is the identical bug in another codebase. The daemon owes its children a declared env contract.
3. **The Blocked exit-door fault-injection test is the best single addition in the document.** Upstream's #123/#199/#111/#170 produced "a record with zero supported transitions" from real command output (mining lesson 3), and the mining note flags sergeant-rs's fail-closed-into-`blocked` as a *live* risk. Testing "an operator can get it out," not just "it stops safely," is exactly right and unambiguously core.
4. **Backlog as a separate durable type, not a widened `WorkState` (ruling #9), with idempotent promotion keyed on dedup identity to survive L6.** Correct on all three counts. I move it earlier; I would not change a word of it.
5. **Merging delta #5 into the backlog (ruling #8).** Right — a separate `gate` object duplicating ordering, dedup and provenance is precisely the fragmentation upstream spread across #146–#197.
6. **Cutting T-series to the minimal slice (ruling #13) and correcting "primary" to "secondary" (#12).** ~1,900 lines of spec gated on a hold-state E3 has not built is premature by evidence. No core-first objection.
7. **Ruling #14's inversion of R-H0-2.** Correct, and it is my own argument applied to H-series; I cite it against the draft rather than disputing it.
8. **Ruling #18 — E6's fix must itself run as a sergeant Work.** The single best falsification test in the document. I strengthen it into the core wave's exit criterion rather than weakening it.
9. **Retiring "Depot," and the whole NOT EVER list** — fleet as a domain object, PM semantics, operator-specific integrations, reconstructed supervision machinery (upstream #152's pane-identity drift is structurally solved by daemon-owned handles), the external DAG engine (#131→#132 decoupled within one release cycle), the OTel bridge, snapshot loading. All sound.
10. **The ordering principle itself** — "what falsifies the destination loop beats what degrades it" — is the right rule. My whole attack is that the draft stopped applying it at Wave 1.
11. **The genuine risk in my own path, stated plainly.** "Unimpeachable core" is unbounded and can absorb unlimited time; the draft is right to fear a core wave with no exit. The honest example is upstream mining lesson 5 (SQL injection, "worth checking `src/runtime/analytics.rs`"): I checked, and it is already satisfied — `table_rows` allowlists against `TABLES` before interpolating (`src/runtime/analytics.rs:1307-1315`), queries are canned, inserts go through the appender. Generic hardening finds nothing here. That is why my path is a *finite enumerated list with a falsifiable exit test*, not a posture. If the owner will not bound it that way, the draft's sequencing is the safer choice.
12. **G1's deferral is right** (U-R5 defers it explicitly, no dogfood evidence hit it), and the draft's observation that it is "the missing half of `WorkState::Waiting`" is a better framing than the retriage's.

### Strongest case
1. The engine is the product and the distro is its packaging: the one dogfood run that delivered value did it while the Work wrapper "added audit trail and nothing else" — a verdict on missing core (E1/E6/E7), not on missing docs.
2. Wave 0 closes none of the draft's own gaps 6, 7 and 8: #19 soak stays open, the fake backend still cannot represent time (one `stalls_for` parameter hid a 45-minute stall behind 352 green tests), and L16's arithmetic proves E1's signal is not a brake.
3. Order by irreversibility, not by proximity to the user: dedup identity (7 upstream collision issues), promote semantics, the intent schema and admission/drain (retrofitted three times upstream) are the expensive-to-change items, and all four are core.
4. Onboarding is cheap to delay and expensive to un-ship — no external user is cited anywhere in the record, while a prebuilt binary freezes a pinned CLI version gate and four waves' worth of still-moving engine semantics into strangers' hands.
5. Bound the core wave so it cannot sprawl: nine named items, exit test = the core's own fixes run as sergeant Works against this repo (ruling #18) plus a second dogfood round — then, and only then, invite strangers.

### Costliest decision
**Whether the distribution surface — prebuilt binary plus stranger onboarding (Wave 1, ruling #20 and ruling #7's boundary) — opens before the engine's semantics freeze and a spend envelope exists.**

Every other sequencing call in the draft is reversible by reordering a later wave. This one is not: you cannot un-ship a binary or un-invite a user, and each stranger becomes a compatibility constraint on promote semantics, the backlog type and the intent schema that Waves 2–4 still intend to change.

The asymmetry is lopsided. Wrong-to-wait costs adoption nobody in the record has asked for — there is not one external user cited anywhere in the corpus, and gap 1 concedes no install time has ever been measured, so even the size of the prize is unknown. Wrong-to-ship costs a stranger an unbounded bill on a host we never measured, with L16 proving arithmetically that no guard sized below one maximal turn can stop it ($2.50 guard, $4.62 spent, turn 2 alone $3.21) and L17 proving that stopping the thing that started it does not stop its effects — and it does so with no orchestrator watching, which is the exact difference between us and them.

Decide it explicitly and record the arithmetic: if onboarding ships in Wave 1, a per-Work hard spend/turn envelope enforced at the PREPARE/LAUNCH boundary must ship with it, not after it. If that envelope is not in the wave, the wave is not ready for strangers.

---

## Challenge 2

### Attack
ATTACK — the draft's sequencing rests on a factual error, a layer misassignment, and a circular acceptance test.

**1. Wave 0's keystone claim is false against the code.** The draft calls E6 "the only defect that makes the journal report success while the deliverable evaporates," makes "the artifact of step 5 survives worktree teardown" its acceptance test, and hangs its "single most important sequencing call" on it. `/home/miztertea/sergeant-rs/src/runtime/surface.rs:17-24`: "Teardown retains the branch and removes the worktree… Teardown **fails closed**: a worktree with uncommitted or untracked changes… is *recorded* in the teardown report and left alone. Sergeant never destroys work it did not create." `:191` — "Branch that was retained (always: teardown never deletes a branch)." `teardown_binding_locked` (`:508-511`) returns `RetainedDirty { changes }` on any non-empty `git status --porcelain`, keeping the worktree on disk. The dogfood evidence agrees: the research run's own critic report records "teardown.disposition read **retained_dirty**, meaning the file sat as a plain untracked (`??`) file in a throwaway worktree"; the grilling worktree was removed only because grilling wrote nothing, and its branch survived at base_sha. Nothing evaporated. The draft's stated acceptance test **already passes today**. What actually failed is that no surface tells the operator where the thing is — and the same critic's POSITIVE note says `sgt run --json` "returns the full work/stage/surface/workflow state, **including the worktree path and branch name**." The information is already served by the API (`src/api.rs:1166` puts the teardown report on work-show). The gap between "the engine hands you the path" and "the operator knows to look" is a documentation gap. **The draft's #1 reason to block onboarding is itself an onboarding defect.**

**2. The one run that actually missed its deliverable missed it for a content reason engine work cannot fix.** The critic verified `.sergeant/workflows/research/00-investigate/output/README.md` WAS materialized, that the actor wrongly claimed otherwise, and that "the ICM convention's `output/` scaffold and the workflow's own behavior contract (BU-P3-044, 'place per the repo's note-keeping convention') point to two different 'correct' locations, and nothing forces or even flags the conflict." A promote/finalize engine would have promoted an empty `output/`. Moreover the convention already rules this to be non-engine work: `/home/miztertea/sergeant-rs/docs/icm/convention.md:143-176` — "declared locations, **no engine collection, no artifact manifest machinery**… the finalize step is a shared helper invoked by the closing actor stage." By the draft's own R-NS-1 durability test, E6 is OS-layer. The draft applied its taxonomy to twelve tensions and not to its own Wave 0.

**3. The draft ignores the measurement's own top-ranked finding.** The critic labels one item "biggest issue for a first-time user": diagnose-bug's name promises diagnosis while stages 40/50/60 write and apply a fix, with nothing in `sgt run --help`, `index.md` or `CONTEXT.md` warning pre-submit. That is a workflow package edit. It appears nowhere in Wave 0, Wave 0.5 or the E-list; it lands in Wave 1, behind four engine items. The draft ranked by its own falsification theory and dropped the finding the measurement itself ranked first.

**4. E2 is already answered in this repo, in a file the actor structurally cannot read.** `/home/miztertea/sergeant-rs/docs/environments/cerberus.md` records, dated the same day: "cargo/rustc 1.97.1 in `~/.cargo/bin` — **not on non-interactive shells' default PATH**; prefix `PATH="$HOME/.cargo/bin:$PATH"`." The actor burned ~15 of ~29 tool calls rediscovering it and concluded wrong ("cargo is being blocked by the permission layer"). Why: `src/backend/claude.rs:881` launches every turn with `--setting-sources user`, deliberately, per L2's capture hazard — "the target repo's project memory must not be able to install a different identity on the execution agent." So CLAUDE.md and AGENTS.md are invisible to the actor **by design**, and the only channel into it is stage `CONTEXT.md` plus `@@` common contexts — the OS layer, exclusively. A `@@environment` common context is a one-file, zero-code fix available this afternoon that retires the largest single waste in the measured corpus. The draft classifies E2 as a Wave-0 engine falsifier and gets both the layer and the schedule wrong.

**5. Wave 0.5 puts the product's front door behind a vendor we do not control — and does it by relabeling a small item as a large one.** E3 as recorded in `/home/miztertea/sergeant-rs/docs/gauntlet/runs/dogfood-2026-08-11/run-manifest.md` is "**submission-time capability gating**" — precisely what the critic asked for: "a pre-submit capability check (e.g. `sgt run` warning 'ask capability not available on this backend') would have saved real confusion." That is a preflight string. The draft renames it "E3 interactive hold," asserts "the conversational half of the loop is E3-shaped," and claims four consumers. That larger item requires re-creating an affordance the transport withdrew: cerberus.md records "**a5 red — `post_turn_summary` absent on this host**… GP-2's ask affordance gone, runtime withdrawal path operative" on claude 2.1.227. The draft's own Ruling 14 names this (#7, protocol-derived capability withdrawal) as the one live defect and notes R-H0-3's second-transport probe is still open. So Wave 0.5 gates all onboarding on an unbounded upstream dependency on the only transport that exists. Two items with a 100× cost spread are sharing a label, and the expensive one is being used to justify the delay. Ship the preflight warning with the OS layer in week one; let the hold wait for the transport question it actually depends on.

**6. The draft's own recursion test is circular.** Ruling 18 requires that E6 "must itself be run as a sergeant Work against this repo." Driving a Work against this repo requires a `sergeant.toml` whose shape the critic found "not discoverable from `sgt --help` or `sgt run --help` at all… only be found by reading prior run manifests or GAUNTLET history" (E5, Wave 2), knowing `--profile` is not implied (Wave 2), a daemon you can stop (E4, Wave 2), and a harness that knows the drill (AGENTS.md, Wave 1). The dogfood substituted an 83-line bespoke JS driver (`/home/miztertea/sergeant-rs/resources/n-series/dogfood-gauntlet.js`) whose prompt literally instructs the operator to "copy the mechanism from `docs/gauntlet/runs/runB2/run-manifest.md`'s setup section." That is the OS layer, hand-simulated, re-paid per run. The draft's Wave-0 acceptance test depends on the Waves it deferred. Under my path AGENTS.md + `sgt init` + estate ARE the driver, and the recursion becomes executable in week one instead of month two.

**7. It spends the expensive currency before the cheap one has told it where to aim.** R-S0-12 (GAUNTLET.md:519-521): "any executable diff takes the full multi-axis loop; the P1-PERF single-builder exemption covers only phases that write no code." Ledger cost of an engine milestone under that rule: 7–38 agents, 0.99M–3.4M subagent tokens each (GAUNTLET.md:425, 477, 732, 1200). Cost of the measurement that produced the entire E-list: **$5.29**. Four Wave-0 engine loops before the next measurement means ~4–8M tokens of full-loop review committed against N=3 runs, on one host, with a driver that no longer exists, on a ranking whose top item is factually mis-stated (point 1). OS content is largely non-code — cheap loop, revisable in place per the `reference/notes/` convention. Buy the instrument, then buy the engine work the instrument names.

**8. The prebuilt binary is misplaced as "onboarding," and the draft half-knows it.** Ruling 20 and gap #1 correctly call the ~10-min cold DuckDB compile fatal to the five-minute promise, then park it in Wave 1. But `docs/environments/claude-code-cloud.md` records container resets that "wipe installed tools and `target/` (~10 min DuckDB rebuild)" — three in one S-series day. `.github/workflows/` contains only `ci.yml` and `coverage.yml`: no release job exists. A release artifact is not stranger-polish; it is the largest measured throughput fix available to the people doing the engine work, it changes no engine semantics, and it costs one CI file. Day one, unconditionally.

**9. The question that decides whether sergeant has value at all is scheduled last.** The draft's own gap #2 calls Work-vs-inline routing "unowned"; Ruling 4 assigns it to the OS layer; and the dogfood's grading verdict on the *successful* run is "mostly a wash… the Work wrapper added audit trail and nothing else," "inert ceremony for a single-turn shape." If the routing judgment is wrong, E6/E1/E7 make a ceremony nobody should have invoked marginally more legible. The correct ordering rule is not "what falsifies the loop beats what degrades it" — it is **what decides whether the loop should run at all beats what decides how well it runs.** Routing lives in AGENTS.md. AGENTS.md is Wave 1.

**10. The reputational argument smuggles a false premise and the wrong audience.** "Onboarding a stranger into a loop that discards its output is worse than no onboarding" fails twice: the loop does not discard output (1), and nobody is proposing to invite strangers. U-R1 rules the clone IS the distro; U-R3 rules that "development work that fits a surviving workflow runs as a sergeant work **from here on**" and that "friction found while self-hosting feeds the E-list." Writing AGENTS.md, `sgt init` and the estate onboards *us* — the only users who exist. The draft conflates building the day-one experience with publishing it. A one-line README banner ("pre-1.0, see KNOWN-GAPS.md") dissolves the entire reputational objection at zero cost, and the draft's Ruling 7 — its self-declared most important call — has nothing left underneath it.

**The inversion, concretely.** Week 1 (no full-loop cost): AGENTS.md rewritten as canonical front door incl. the routing judgment and "where your deliverable is"; `@@environment` context wiring `docs/environments/` into every stage (kills E2's measured waste); workflow-package honesty pass (diagnose-bug's name/scope, research's output/ conflict, the finalize helper the convention already specifies for E6); estate manifest + `sgt init` (thin — `doctor` already has git/claude/data_dir/permission_mode checks at `src/cli.rs:964-1034`); release-binary CI job; `--data-dir` default flipped to estate-root per U-R2. Week 1 also, in parallel and as code: E1 (see concessions) and the two ~5-line surface honesty fixes the critic named — `src/cli.rs:945`'s `NotARepository => Check::ok` ("silently reports profile-less state… no hint") and the pre-submit capability warning. Then dogfood round 2, driven by AGENTS.md instead of a JS file, against this repo's own defects, at roughly $5 and no gauntlet loops — and let *that* rank the engine list.

### Concessions
Points the draft genuinely wins, conceded without hedging:

1. **E1 is core, it is first among engine items, and my path must not defer it.** No AGENTS.md edit can synthesize a `usage.updated` record for a killed turn. The critic's finding is sharper than the draft states: "not just 'no live signal,' it's 'no signal ever, even after the fact'" — and the successful run finished 7.8% over cap with `sgt cancel` correctly 409'ing. Combined with L16/L17 this is the one open defect that can cost a real person real money with no brake. I take E1 into week one as code, in parallel with the OS layer, and I concede this weakens my "engine work strictly on demand" framing: E1 is on demand *already*, demanded by the measurement.

2. **The blocked exit-door fault-injection test is core, urgent, and has no content substitute.** Upstream bled fifteen issues on this shape; ambiguous-recovery parking is exactly where a walk-away user gets stranded. Keep it in the first engine batch.

3. **The §2 ownership taxonomy (R-NS-1..R-NS-5) is the best thing in the document** and I adopt it wholesale rather than proposing an alternative. My quarrel is that the draft did not apply its own durability test to Wave 0 — doing so reclassifies E6 and most of E2 out of core, which is my argument, made with the draft's instrument.

4. **Ruling 14 (inverting R-H0-2) is the strongest single ruling in the paper** and survives my attack intact: land #7's capability-provenance fix now, defer 1–3+8 behind R-H0-3's probe. It is measured evidence beating an orchestrator recommendation, exactly as the epistemic license asks.

5. **Ruling 9 (backlog as its own type, not a widened `WorkState`) is correct and well-derived from L6**; so is Ruling 8 (a review finding is a queued intent, not a second `gate` object), Ruling 2 (canonical vs generated split — which my path depends on more than the draft's does), Ruling 3 (retire "Depot"), Rulings 12/13 (T-series secondary, cut to the minimal slice), and the entire NOT-EVER list including the reconstructed-supervision entry.

6. **Gap #6 (no soak evidence, #19 open) is real and my path does not fix it either.** Thirteen turns and two cascades do not support "walk away for hours" from anyone's sequencing. Neither paper has an answer; it should be named as a shared open item rather than scored against the draft.

7. **The draft is right that E6 is unfinished work.** My correction is about layer and blocking power, not existence: the finalize helper must be written and the disposition lint must run, and until they do, workflows will keep leaving deliverables as untracked files. I am arguing that this is a `.sergeant/common/scripts/` commit plus a lint, available now, not a Wave-0 engine gate on the whole product.

8. **My path's honest weakness, stated before it is used against me:** prioritizing by dogfood overfits to one host and three runs, and Cerberus's missing `post_turn_summary` may be an auth-mode artifact rather than a general fact (cerberus.md flags it as "plausible gating variable, not isolated"). The mitigation is the argument itself — the OS layer is what makes round 2, 3 and 4 cheap enough to raise N. But if the owner's intent is to ship to strangers within the quarter rather than to self-host, the draft's ordering gets stronger and mine gets weaker, and that is the axis on which I would expect to lose.

### Strongest case
1. The draft blocks the day-one experience on E6, whose premise is false: `surface.rs:17-24/:191/:508` retains the branch always and retains the worktree whenever anything was written, and the dogfood teardown record reads `retained_dirty` — nothing evaporates, the operator is just never told where it is, which is a doc defect guarding a doc fix.
2. Every remaining Wave-0 item is either already answered in a file the actor cannot read (E2: cerberus.md's PATH fact, invisible because `claude.rs:881` sends `--setting-sources user`, so stage context is the only channel) or one honest string away (E3's real recorded form is a pre-submit capability warning) — the OS layer is where the measured failures actually live.
3. R-S0-12 prices engine work at 7–38 agents and 1–3.4M tokens per milestone; the measurement that produced the whole E-list cost $5.29 — so the cheapest way to be right about the engine is to build the instrument (AGENTS.md, skills, estate, `sgt init`, prebuilt binary) that makes measurement repeatable, then spend loops on what round 2 names.
4. The draft's own recursion test (Ruling 18: run E6 as a sergeant Work) cannot execute until the Waves it deferred exist — today that gap is filled by an 83-line JS driver quoting a run manifest, re-paid every round; U-R1 and U-R3 already ruled the clone is the distro and self-hosting starts now.
5. The reputational objection dissolves against the actual audience: we are the only users, nobody is publishing a 1.0, and a one-line pre-1.0 banner costs less than the four full-loop gauntlets the draft would spend before anyone can test the five-minute claim once.

### Costliest decision
**Is the day-one experience a deliverable or an instrument?**

Everything else follows from this one framing, and both papers answer it implicitly rather than arguing it.

If AGENTS.md + skills + estate + `sgt init` is a **deliverable** — a front door onto a room — then the draft's Ruling 7 is right by construction: you do not hang a sign on a broken room, and my path invites judgment onto an unfinished product.

If it is an **instrument** — the apparatus that makes each subsequent product-fitness measurement cheap, repeatable, and driven by the product itself rather than by a bespoke script — then deferring it behind four full R-S0-12 loops means every engine decision between now and Wave 1 is made on N=3 runs, one host, one driver that no longer exists, and a ranking whose top item is factually mis-stated against `surface.rs`.

Asymmetry of being wrong:

- Wrong in the draft's direction: ~4 multi-axis engine loops (call it 4–8M subagent tokens plus the calendar) spent building promote machinery the convention says should not exist (`docs/icm/convention.md:146-148`: "no engine collection, no artifact manifest machinery"), an env contract that a `@@environment` context would have covered, and an interactive hold gated on an affordance claude 2.1.227 withdrew — while the routing question that determines whether any of it is worth invoking ("mostly a wash," "inert ceremony") stays unowned in Wave 1. Unrecoverable, because those loops are the scarce resource here.
- Wrong in my direction: a week of doc and content work that a later engine change partially invalidates. Cheap, and revisable in place — `reference/notes/` are living docs by their own convention, and OS content carries no R-S0-12 loop.

That asymmetry is the whole argument. But it inverts if the owner's real horizon is strangers-this-quarter rather than self-hosting-this-month — so the decision the owner should actually make first is not the wave order, it is **who the next ten runs are for**. Answer that and the sequencing falls out; leave it implicit and both papers are optimizing for audiences they never named.

---

## Challenge 3

### Attack
## THE CUT PATH — "the engine is done; what's left is content and ~200 lines of legibility"

The draft's own ruling 19 is the strongest sentence in it: dogfood outranks proposals drafted before dogfood. It then declines to apply that ruling with force. It re-sequences the pre-dogfood corpus into five waves plus eight deferred programs, each with an unblocking condition — which is *keeping* the scope, on layaway. A cut seat applies ruling 19 as a delete key. Eleven cuts follow, ordered by months saved. Each names the file or measurement it rests on.

---

### CUT 1 — E6 is not an engine subsystem. The code says nothing evaporates.

The draft makes E6 the flagship Wave 0 falsifier: "the only defect that makes the journal report success while the deliverable evaporates," and §1 says "step 5 is a lie until this lands." Check that against the code it is a claim about:

- `src/runtime/surface.rs:17` — **"Teardown retains the branch and removes the worktree — the branch *is* the durable surface."**
- `src/runtime/surface.rs:522` — teardown runs `git worktree remove` with **no `--force`**. Git refuses a dirty worktree, so the failure path is `BindingDisposition::RetainedError` — the worktree is *retained on disk with the error recorded*, not destroyed.
- `sgt work show` already merges the `surface` block into its output (`src/cli.rs:364-382`), and that block's bindings carry `work_branch` — `src/tui.rs:544` reads exactly that field off the same API payload.
- `docs/icm/convention.md:162-178` places finalize in **content**: "the finalize step is a shared helper invoked by the closing actor stage."

So the measured shape is: committed output → on `sergeant/<work-id>` in the user's own repo, branch name already printed by the CLI. Uncommitted output → worktree retained. What actually failed in dogfood is that nobody *told the operator* either of those things, so they hand-rescued files into `dogfood/<name>/deliverables/`.

E6 re-scoped: (a) one measurement — run one workflow to completion, then `git branch --list 'sergeant/*'`; (b) print "your change is on branch `sergeant/<id>` in `<repo>`" at terminal state; (c) put a commit step in the closing stage of the ~5 packages we keep. R6/R2, one afternoon, zero new event kinds, zero Layer-4 disposition engine, zero execute-stage dependency. If the measurement contradicts `surface.rs:17`, you have a *bug*, and it is still a bug fix and not a subsystem.

The draft inherited "deliverables die with the worktree unless hand-rescued" verbatim from `run-manifest.md:48-50` and promoted it to the top of the roadmap without reading `surface.rs`. That is precisely L15 ("a claim transmitted to another agent must carry its evidence or be labeled hypothesis") and L12 ("re-read the governing text at decision time") firing on the North Star itself.

### CUT 2 — E3's interactive hold, and every consumer the draft says it unblocks.

`docs/gauntlet/notes/cerberus-ask-grammar-remeasurement-2026-08-11.md`: on CLI **2.1.227**, `post_turn_summary` is emitted **zero times** across five probes, two models, three permission modes; `task_summary` also absent; `claude --help` documents none of it. It was present on 2.1.226. A vendor removed the entire affordance in a patch bump, silently. `src/backend/claude.rs:194` is the withdrawal predicate, and it fired correctly.

The draft's Wave 0.5 builds a re-enterable `needs_input` hold (G5) that reads its own accumulated answer history — R7 machinery on a foundation that vanished without a changelog entry between two patch releases. That is L1's lesson pointed at a roadmap instead of a test.

Replace it with a **doctrine cut: sergeant owns unattended execution; the harness owns the conversation.** The dogfood verdict on `grilling` — "negative value vs plain terminal Claude" — is not a missing engine feature. It is the correct verdict on wrapping an interactive interview in a durable Work while the human is already sitting inside an interactive harness (R4: use the native platform feature). This deletes: G5, the WORKFLOW-IF-E3 category, `grilling` + `grill-with-docs` (retire, don't gate), the T-series attention model, and R-H0-7's ask-scripting program.

What survives of E3 is its measured cost: **a submit-time preflight** — if `Capabilities::ask` is low, `sgt run` refuses a workflow that declares an ask stage. Two lines and a test.

Direct hit on ruling 5: the draft justifies E3 with "one capability, four consumers." All four consumers are on this cut list. A load-bearing wall in a building you are not putting up is not leverage.

### CUT 3 — the entire T-series, not the "minimal slice." Delete the web dashboard.

Ruling 13 cuts 1,943 lines of spec to "composer, thread, respond, #11/#16." But T-series §5.1 item 1 licenses "replace the TUI's screen hierarchy, view models, keymap, focus, scrolling, and rendering" — and a persistent state-aware composer plus a semantic thread **is** that rewrite. `src/tui.rs` is 1,909 lines; `tests/m6_surfaces.rs` is 3,394. The "minimal slice" is a rewrite wearing a smaller number.

R1 against U-R4's own first clause: the product is a distro that turns *the harness you are already in* into the operator. That is the at-your-seat surface. Three hand-maintained renderings of one API (CLI, TUI, dashboard) for one human is R1 failure three ways.

Cut: freeze `tui.rs` at its P0 proof (it works; the M6 contract met). **Delete** `src/web.rs` (779) + `web/` (224) + the `sgt web` verb — T-series §5.1 item 14 already proposes disabling the route and leaving a stub with two live reactivation issues (#15, #21); deletion is the lower rung than a disabled stub that still owns issues. #11 (width) dies with the freeze. Keep #16 (SSE reconnect) only if it also affects a CLI follow path — otherwise it is a bug in a frozen client.

Concede: ruling 12 (demote T-series from "primary interactive surface" to secondary) is right. But "secondary" and "we will rewrite its screen hierarchy" cannot both be true.

### CUT 4 — the backlog domain type (U-R5 / delta #8) and structured intent (U-R6 / delta #7).

Wave 3 buys a new durable type, a new event vocabulary (`backlog.captured`/`backlog.promoted`), dedup identity, provenance links, and idempotent promotion hardened against L6 adjacent-append windows. Apply R1 with the draft's own destination text: read §1's five steps. The word *backlog* never appears. This is not on the path to the loop.

Apply R2: **this repo already runs its backlog on GitHub issues** (#11 #15 #16 #19 #21 #23 #26 #45 #46 #47 #50 #53 #57) plus `GAUNTLET.md`'s backlog table with named triggers. Those already have dedup (issue numbers), ordering, human triage, and provenance — CLAUDE.md mandates the `Fixes #NN` trailer as the causal link. The harness can read and write all of it with `gh` today, and `gh` is R5 (installed dependency). Building a second issue tracker inside the journal, then teaching every projection, the graph fold, recovery, and every client to special-case a state that never runs, is a textbook R2 failure. Ruling 9 correctly identifies the poisoning risk and then answers it with a new type; the cheaper answer is no type.

U-R6: the one measured success in dogfood was graded "**inert ceremony for a single-turn shape**." The response to measured ceremony overhead is not an 8-section intent schema. Keep `intent: String`; a workflow needing structure asks for it in its CONTEXT.md — content, per R-NS-1's own durability test, which the draft states and then rules against.

Bonus: this deletes gap #9 (no dedup identity scheme) outright rather than filing it.

### CUT 5 — the estate machinery (manifest, registry, `sgt init` beyond `mkdir`, per-repo instruction contract, `sgt project *`).

The draft's own §4 gap 5 concedes the central value claim — cross-repo coherence, the "light monorepo" — "has no implementation and no contract," and no Work can span two `repos/` entries. Wave 2 therefore ships *declaration machinery for a capability that does not exist*. R1.

What the loop needs from an estate is one thing: point `sgt run` at a repo, which `--workspace`/workspace discovery already does. Make `repos/` a **documented convention** — a directory holding clones, worktrees, or symlinks — with zero tracked manifest, zero verify/populate code, zero new verbs. If doctor should say "`repos/foo` is not a git repo," that is five lines inside a shipped verb.

Keep exactly one piece of U-R2: flip the data-dir default off XDG to estate-root `.sergeant/data/` (`src/cli.rs:225`). That is R6, it makes the clone genuinely self-contained, and it is the only part of the estate ruling the destination sentence actually requires. Concede ruling 11 (`sgt project list/status/sync` are skills) — correct, and I extend it to `sgt project graph` and delta #6.

### CUT 6 — N4 Docker: delete, do not defer.

`docs/gauntlet/contracts/N4.md` scopes §16.1–16.12 (twelve subsections of Docker executor), §22.7–22.10 (contract matrix, 1 GiB-capture RSS budget, image/cache pressure, doctor rows), §22.5 crash-injection across every create/start/exit/cancel window, exact-cleanup sweeps, digest pinning, ownership semantics for container-written files, three-environment probe-gating — **plus** two bracketed retention rules (terminal-work projection eviction with journal re-derivation; a doctor disk-pressure check). That is multiple milestones with a perf-regression surface attached.

Against §1's five steps it delivers nothing. And the lower rung already exists: an actor stage can run `docker run` in its worktree today (R6). "Deferred behind Wave 0 and the seam question" (ruling 15) is not free — it keeps §16, the N4 contract, the retention ruling draft, and their perf budgets alive as tracked scope that every future contract must cite around.

Note the cost already paid: the Cerberus close-out records #44 (journal group commit) landing with "**no throughput delta on this host; the win is O(lock-holds) journal cost for N4's volume**." We have already shipped one optimization whose sole justification is a milestone that should be deleted. Deferral keeps that meter running.

### CUT 7 — the harness-family program (H1 contract v2, admission suite, non-Claude adapters, D6).

Ruling 14 inverts R-H0-2 and lands "#7's capability-provenance fix" with the E-list. Go further: **the provenance machinery already shipped and already worked.** On 2.1.227, `Capabilities::ask` lowered on the first completed turn exactly as INV-N3-06 designed, and a5 went red as intended (Cerberus close-out, item 2). That is not a defect in the one transport; that is the design succeeding under a live vendor withdrawal. Name a concrete failing behavior or drop the item.

D6's own text is the argument: adapter code that cannot be validated against the real harness is "prose with a compiler." Keep `claude -p` + fake. **Delete `src/backend/codex.rs`** (22 lines of stub) and close D6 as *not pursued* — the §15 trait stays backend-neutral, so any future adapter is additive, which was D6's own reasoning. Cut R-H0-3's probe, R-H0-4, R-H0-5, the admission-suite skeleton, and H1's items 1–3+8 entirely (the draft already defers them; deferral still costs a register row and a re-litigation each time a fresh seat reads it, per L3).

Fake-backend fidelity: keep only what is measured. The fake settling at launch hid #46 for 352 green tests, and **one stub parameter (`stalls_for`) was the whole difference** — that landed at BS2. A general program to model "deferred turn end, persistent sessions, event-driven arrival, typed interactions" is R7 for transports we just cut.

### CUT 8 — DuckDB. This dissolves gap #1 and ruling 20 without building a release pipeline.

Ruling 20 answers the ten-minute install with "a prebuilt binary is a Wave 1 requirement." That buys release CI, cross-compile targets, artifact hosting, checksums, an upgrade story, and a version-skew surface — permanent new machinery to work around a dependency. The lower rung is to remove the dependency.

`Cargo.toml:17` — `duckdb = { version = "1", features = ["bundled"] }`. CLAUDE.md's own cost record: ~500 C++ translation units, ~10 min cold, and the `[profile.dev.package.libduckdb-sys]` pin standing between a 5 GB and a 15 GB `target/`. What it buys:

- **Three** canned queries (`blocked_time_per_work`, `backend_retries`, `execution_touched` — `src/runtime/analytics.rs:252-289`).
- Storage for `graph_nodes`/`graph_edges` — and `src/runtime/graph.rs:1-11` says the graph is "a pure fold with no I/O" that analytics merely *materializes*; the fold is already a `BTreeMap` in memory.
- `analytics.rs:16` states the coupling is structurally contained: "`duckdb` is imported by this module and nowhere else."

Cut `analytics.rs` (1,748 lines), `sgt analytics`, `/v1/analytics*`; keep the graph fold in memory behind the existing `/v1/graph/work/{id}`. `cargo install --git` then becomes a real one-command install with **no release pipeline at all**, and the destination's five-minute promise is met by subtraction. Honest costs: an M5 deviation row, a coverage-baseline shift, and `tests/m6_surfaces.rs` t5's pinned `ApiViews` method set moves — all ledger work, not a program.

No seat proposed this. It is the largest single lever in the document.

### CUT 9 — the workflow library: 35 packages, 372 files, ~8.9k lines of prose, 3 ever run.

The measurement: 3 runs, 1 valuable. The valuable one (`research`) is the **smallest package in the library** — 5 files, 108 lines — and its critic said the engine added nothing to it. The draft never touches library size; the catalog it hands a stranger in Wave 1 is 35 packages, 2 of which measured negative.

The retriage plans to convert 9 CLI-SURFACE packages into verbs. Retire them instead — and deltas #1, #3, #4, #5 and half of #6 leave with them: `drain-fleet`, `monitor-fleet`, `project-graph`, `reconcile-and-cleanup-fleet`, `respond-to-worker` (already collides with shipped `sgt respond`), `route-review-findings`, `wake-and-resume`, `wiki-digest`, `deliver-external-callback`.

Keep an admitted set of ~5 with measured or self-evident value (`research`, `code-review`, `diagnose-bug`, `validate-and-ship`, `direct-implementation`). Move the other ~20 to `.sergeant/drafts/workflows/` — `docs/icm/convention.md` §2 already provides that boundary and says "the boundary is the directory itself," so the cut costs one `git mv` and zero code. Re-promote on evidence, one package per dogfood round.

L8's spirit — a capability flag is a claim requiring a contract test — applies to a 35-package catalog advertised to a stranger. 32 of them are untested claims.

### CUT 10 — delta #1 (fleet drain) and delta #2 (stall detection), which the draft *promotes* into Wave 2.

**Drain** is called "genuinely core: a race-safe admission gate." For a single-human estate the admission gate is *not typing `sgt run`*. Upstream needed drain because tmux panes were spawned by many scripts with no owner; CLAUDE.md's one-owner invariant is precisely what retires that problem. R1.

**Stall detection** is promoted "because #46/#47 measured a 45-minute real stall that 352 green tests could not see." But the Cerberus close-out records #46/#47 as **closed at BS2** — root cause OBSERVE starvation after turn end, fixed by the completion driver. The draft cites the symptom of an already-fixed bug as the trigger for a new daemon-resident subsystem with kill/relaunch policy. If a stall recurs, the R6 answer is a per-turn wall-clock budget in the profile that cancels and journals — **the same knob** that answers gap #8 (no unsupervised safety envelope) and E1's financial exposure. One mechanism, two problems, ~50 lines.

### CUT 11 — the governance layer the draft adds.

Five waves, eight NOT-YET entries with unblocking conditions, 20 rulings, and five new numbered R-NS rules — set beside the existing D-register, the R-S0/R-N0/R-H0 rulings, the B-backlog table, and the E-list. L3 requires pointing every fresh reviewer at the register first; a sixth numbering scheme makes that briefing longer, and Ponytail binds governance too. R-NS-4 is a restatement of what `m6_surfaces.rs` t5 already enforces mechanically (R2). R-NS-2, R-NS-3 and R-NS-5 are one sentence each in AGENTS.md. R-NS-1 is a genuinely good test — and applied honestly it decides CUT 4 against the draft's own ruling 9.

---

### WHAT THE CUT PATH SHIPS INSTEAD — one milestone, then measure

1. `sgt work transcript` (E7) + terminal-state output pointer ("branch `sergeant/<id>` in `<repo>`") — the legibility fix that answers both "what happened" and "where is my change."
2. Per-work wall-clock + spend cap that cancels and journals (E1 + gap #8 + delta #2, one knob).
3. PATH/env contract surfaced in `sgt doctor` and inherited by daemon-launched subprocesses (E2 — concede fully, ruling 6 is right).
4. `sgt run` submit-time capability preflight (E3, the two-line version).
5. `sergeant.toml` discoverability (E5) + estate-root data-dir default.
6. Delete: DuckDB/analytics, `src/web.rs` + `web/` + `sgt web`, `src/backend/codex.rs`; freeze `tui.rs`; demote 30 workflow packages to drafts.
7. AGENTS.md rewritten as canonical front door; `CLAUDE.md` → symlink; "Depot" retired.
8. **Then run the draft's ruling 18 as the whole acceptance test**: the first fix runs as a sergeant Work against this repo, and a stranger-clone install is timed.

Everything else waits for a second dogfood round to *ask* for it.

---

### CONCESSIONS — where the draft genuinely wins

- **Ruling 18 is the best item in the document.** Making the first Wave 0 fix run as a sergeant Work against this repo is the cheapest possible falsifier of the whole thesis, and U-R3 is unmeasured today (gap #4 is correctly stated). I would not merely keep it — I would make it the entire milestone gate.
- **Ruling 19 is correct and is the engine of my own argument.** Dogfood outranks pre-dogfood proposals. My complaint is under-application, not error.
- **Ruling 6 (E2 is a falsifier, not polish) is right**, and ruling 17's principle — a defect resolvable only by operator discipline is a product defect once strangers are the audience — is the correct generalization. E2 stays in my Wave 1 unchanged.
- **Ruling 7 (onboarding strictly after the loop works) is right**, and it is the draft's own best sequencing call. My cuts exist to make that wait short rather than to reorder it.
- **Rulings 1, 2, 3, 4 (AGENTS.md rewrite; canonical/generated split; retire "Depot"; routing judgment lives in the OS layer) are all correct, all content, all cheap.** Ruling 4 in particular is the same principle that kills `05-classify-risk`, which the draft only defers.
- **The "NOT EVER" list is the strongest section of the draft** — fleet as a domain object, PM semantics, operator-specific integrations, upstream's re-hash ceremony, tmux supervision, external DAG, the otel bridge, snapshot loading. I add to it; I subtract nothing.
- **Ruling 12** (T-series demoted to secondary) is right and I am extending it, not contradicting it.
- **The two-clause identity sentence is right** and I am not touching it. Distro-as-product with the engine as component is the correct frame; my cuts make it reachable in one milestone rather than six.
- **E7 is real and the draft under-ranks it at fourth.** It is the single highest-value unbuilt thing in the corpus, and CUT 1 makes it carry E6's job too.
- **Gap #7 (test-suite credibility is unbounded for timing) is a genuine finding** and the `stalls_for` precedent proves the cheap version works. I keep the narrow fidelity fix and cut only the general program.
- I have not run the E6 measurement myself. CUT 1 rests on `surface.rs:17`, the missing `--force` at `surface.rs:522`, and the CLI's existing `surface` output. If a live run contradicts the module doc, E6 is a leak to plug — and the draft's Wave-0 placement is then vindicated on severity, though still not on scope.

### THE STRONGEST 5-LINE CASE

1. Three measured runs say the loop fails on **legibility and cost visibility**, not on missing capability — and the one artifact of real value came from a 108-line prompt that the engine's ceremony added nothing to.
2. The engine is finished: 27.6k lines, 379 tests, a journal that retains the branch (`surface.rs:17`), a hold, a respond verb, a recovery path, and a live-proven autonomous cascade — the draft's Wave 0 is ~200 lines of print statements and one cap knob away from closing.
3. Every remaining wave adds a domain concept the destination sentence never asks for — a second issue tracker, an estate registry with no cross-repo capability behind it, a Docker executor, an intent schema, a third client — while `duckdb` alone costs the ten-minute install the draft proposes to fix by building a release pipeline.
4. Two of the biggest programs rest on foundations that already moved: E3/G5 on a vendor JSON line that vanished between 2.1.226 and 2.1.227 with no changelog, and stall detection on a bug closed at BS2.
5. Cut to one milestone — transcript, output pointer, spend cap, env contract, preflight, delete DuckDB/web/codex/30 packages — then run the first fix as a sergeant Work against this repo; if the recursion carries it, everything cut can be re-argued from evidence instead of from proposals written before anyone tried the product.

### THE COSTLIEST DECISION

**Does sergeant own the interactive conversation, or only unattended execution?**

Everything else on both paths is one afternoon to one week. This one decision controls the T-series (1,943 lines of spec plus a `tui.rs` rewrite), E3's re-enterable hold (G5), the WORKFLOW-IF-E3 packages, the attention model, R-H0-7's fake-backend program, and roughly half of H-series — call it months.

The asymmetry decides it. Choosing "sergeant owns interactive" costs those months building surfaces that compete with the harness the user is already sitting in, on top of an ask affordance a vendor deleted in a patch release without documenting it (five probes, two models, three permission modes, zero `post_turn_summary` lines). Choosing "unattended only" and being wrong costs one dogfood round: users say "I need to answer mid-run," and the recovery is the two-line submit preflight plus the `sgt respond` and `needs_input` hold that already ship.

The draft answers this implicitly and in the expensive direction — ruling 5 puts E3 at Wave 0.5 and ruling 13 keeps a TUI slice. It should be lifted out as the single explicit fork the owner rules on, because it is the only decision on this page where being wrong costs more than a week.

### Concessions
The draft genuinely wins on: ruling 18 (running the first fix as a sergeant Work against this repo is the cheapest falsifier of the entire thesis, and I would promote it from an added acceptance test to the whole milestone gate); ruling 19 (dogfood outranks pre-dogfood proposals — this is the engine of my own argument, and my only complaint is under-application); ruling 6 and ruling 17 (E2 is a falsifier, and a defect resolvable only by operator discipline is a product defect once strangers are the audience); ruling 7 (onboarding strictly after the loop works — my cuts exist to shorten that wait, not to reorder it); rulings 1-4 (AGENTS.md rewritten not amended, canonical/generated split, retire "Depot", routing judgment in the OS layer not an engine heuristic — all correct, all content, all cheap, and ruling 4 is the same principle that should kill 05-classify-risk outright rather than defer it); ruling 12 (T-series demoted to secondary — correct, and I extend rather than contradict it); the NOT EVER list, which is the strongest section of the document and to which I add nothing but subtract nothing; the two-clause identity sentence, which I leave untouched; E7, which the draft under-ranks at fourth when it is the highest-value unbuilt thing in the corpus; and gap #7 on test-suite timing credibility, where the `stalls_for` precedent proves the narrow fix works. I also concede a limit on my own strongest point: I did not execute the E6 measurement. CUT 1 rests on reading src/runtime/surface.rs:17, the absent --force at surface.rs:522, and the surface block already merged into `sgt work show` output. If a live run contradicts the module doc, E6 is a real leak to plug and the draft's severity call is vindicated — though its scope call (a Layer-4 finalize/promote subsystem gated on execute stages) still is not.

### Strongest case
1. Three measured runs say the loop fails on legibility and cost visibility, not on missing capability — and the one artifact of real value came from a 108-line prompt that the engine's ceremony added nothing to.
2. The engine is finished: 27.6k lines, 379 tests, a journal that retains the branch (surface.rs:17), a hold, a respond verb, a recovery path, and a live-proven autonomous cascade — the draft's Wave 0 is ~200 lines of print statements and one cap knob away from closing.
3. Every remaining wave adds a domain concept the destination sentence never asks for — a second issue tracker, an estate registry with no cross-repo capability behind it, a Docker executor, an intent schema, a third client — while duckdb alone causes the ten-minute install the draft proposes to fix by building a release pipeline.
4. Two of the biggest programs rest on foundations that already moved: E3/G5 on a vendor JSON line that vanished between CLI 2.1.226 and 2.1.227 with no changelog, and stall detection on a bug closed at BS2.
5. Cut to one milestone — transcript, output pointer, spend cap, env contract, preflight, delete DuckDB/web/codex/30 packages — then run the first fix as a sergeant Work against this repo; if the recursion carries it, everything cut can be re-argued from evidence instead of from proposals written before anyone tried the product.

### Costliest decision
Does sergeant own the interactive conversation, or only unattended execution? Everything else on either path is an afternoon to a week; this one fork controls the T-series (1,943 lines of spec plus a tui.rs screen-hierarchy rewrite), E3's re-enterable hold (G5), the WORKFLOW-IF-E3 packages, the attention model, R-H0-7's fake-backend program, and roughly half of H-series — months. The asymmetry decides it: choosing "sergeant owns interactive" spends those months building surfaces that compete with the harness the user is already sitting inside, on top of an ask affordance a vendor deleted in a patch release without documenting it (five probes, two models, three permission modes, zero post_turn_summary lines on 2.1.227). Choosing "unattended only" and being wrong costs exactly one dogfood round — users say "I need to answer mid-run," and the recovery is the two-line submit-time preflight plus the `sgt respond` verb and needs_input hold that already ship. The draft answers this implicitly and in the expensive direction (ruling 5 puts E3 at Wave 0.5; ruling 13 keeps a TUI slice); it should be lifted out as the single explicit fork the owner rules on, because it is the only decision in the document where being wrong costs more than a week.
