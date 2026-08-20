# Backlog close-out sprint — plan (drafted 2026-08-20)

Mission: drive every actionable open issue in `miztertea/sergeant-rs` to
closed — fixed, measured-and-ruled, or explicitly won't-fixed by owner
ruling — using the house gauntlet-style dynamic workflows.

State at draft: **18 open issues** after the 2026-08-20 verified triage
(4 closed that day: #94, #144, #172, #120; 6 re-scoped with panel-verified
comments: #124, #80, #12, #8, #4, #10 — see each issue's 2026-08-20
comment for the receipts). **2 excluded as blocked markers**: #25 (Codex
adapter — blocked on an environment where Codex can be measured) and #176
(Gate F — blocked on the product/workspace-split Phase 5/6). **16 targets.**

## Protocol (carried over from the estate-root run, which it fits)

- Integration branch `integration/backlog-closeout` off `main` in the
  canonical clone `~/sergeant-rs`; a draft **head PR** to `main` carrying
  this plan is the single review surface. **Only the owner merges to
  `main`.** Waves merge freely into the integration branch.
- Wave branches `backlog/w<N>-<slug>`, developed in linked worktrees under
  `/var/tmp/backlog-impl/` (never `/tmp` — tmpfs; never estate mounts).
- Each wave: read-only **recon** (file:line anchors) → **spec** →
  **implement** → 4-axis blind panel (spec-fidelity, invariants,
  simplicity, test-honesty) with per-axis adversarial refuters defaulting
  to refuted → fixer on confirmed findings only → PR into integration.
- Models per standing policy: sonnet by default, opus when earned, never
  Fable for subagents/workflows. Implementer prompts forbid polling
  watchers; the orchestrator verifies from commits, not narration.
- fmt + clippy + full tests green per wave; issue bookkeeping (closures
  with evidence-quoting comments) rides the head PR, closures land when
  the owner merges.

## Waves

### Wave 1 — paper cuts (independent; parallel sonnet implementers)

| Issue | Work |
|---|---|
| #200 | release.yml: pass `make_latest: true` at publish + post-publish assertion that `/releases/latest` == released tag (acceptance criteria in the issue) |
| #143 | probe-env.sh: `bounded()` runs unbounded-with-note or detects `gtimeout` instead of substituting a sentinel; `sysctl -n hw.ncpu` fallback for Cores |
| #124 | `validate-and-ship` `60-close-out`: add the completion-boundary clause (drive the external run to a terminal disposition, or record that one was deliberately left open and why) + doctrine-skew test |
| #80 | doctor/status names the rung that won when an explicit `$XDG_DATA_HOME` is outranked inside an estate; fix ADR 0008's dangling GAUNTLET.md adjudication pointer; adjudicate manifest `data_dir` vs flag/env precedence |
| #88 | author the `record-decisions` (to-adr) workflow package: brief template with decision/alternative/rejection slots, log-gaps-don't-fill instruction, fidelity-first review axes |

### Wave 2 — perf + journal (shared code surface; sequenced within the wave)

| Issue | Work |
|---|---|
| #12 | share one `Journal::replay_data_dir` result across doctor's `journal_check`/`projection_check`/`git_surfaces_check`; measure before/after on the 19k-event estate (target: back near the ~500 ms probe floor) |
| #10 | index `events(kind, work_id)` or restructure `blocked_time_per_work` off the raw events scan; re-measure cold-call at 10k/25k/50k |
| #4 | bound/evict terminal Work structs from `WorkRegistry.works` (spec must state what `work show` on an evicted Work does — re-derive from journal); RSS churn run is the acceptance metric |
| #169 | answer the L6 question: is a torn tail a legitimate transient during append? Make the replay/test contract explicit either way (tolerate-or-retry in the harness, or fix the append atomicity and keep the test strict) |
| #8 | measurement only: 10+ reps of the S3 read-burst block against one long-lived daemon; close (asymptote) or escalate (leak) on the result |

### Wave 3 — estate verbs (features; opus earned for #159's implementer)

| Issue | Work |
|---|---|
| #159 | the §12.3 on-demand inspect/cleanup verb: git-walking, ancestry-classifying, explicitly invoked, deletion as a separate maintenance action; includes the mount-side orphan-ref count folded in from #172's closure |
| #112 | record the upstream GitHub URL estate-side (e.g. optional `github` field on `[[repo]]`) and add it as a named remote on mounts so `gh` resolves naturally in every surface |
| #167 | an on-demand fleet-reconciliation verb running the same identity-verified sync logic as the automatic pre-dispatch pass (not on `sgt watch`, which stays read-only per ADR 0009) |
| #166 | `sgt run --intent-file <path>`: validated (required sections, path-traversal/symlink/size guards) before any dispatch mutation |

### Wave 4 — design contracts (gauntlet evaluation first; implement only what converges)

| Issue | Work |
|---|---|
| #17 | Rule C: archival trigger design for journal segments/blobs/DuckDB — proposal doc + evaluation gauntlet (touches the journal-is-only-truth invariant; earns the full treatment per the issue's own framing) |
| #180 | the enforcement question: proposal evaluating boundary options (backend sandbox flags, path allowlists, teardown-time refusal, or ratifying observation-plus-drift-reporting as the contract) → owner ruling → implement only if the ruling converges small |

Wave 4 may legitimately end at owner-ratified design docs plus follow-on
implementation issues rather than merged code — that outcome still closes
or re-scopes both issues honestly.

## Decisions needed from the owner at kickoff

1. **#166 / #167** — build the verbs (planned above) vs strip the doctrine
   mandates instead. Recommendation: build; #166 carries safety weight.
2. **#80** — is the silent-outrank case worth doctor's breath (the issue's
   ask-item 4, never ruled on)? Recommendation: yes, one detail line.
3. **#112** — estate-side remote (planned) vs environment-side docs only.
   Recommendation: estate-side, optional field, consistent with the
   proposal's "preserve declared origins" ruling.
4. **#4** — eviction policy shape (count-bound? age?): spec proposes,
   owner ratifies via the head PR.
5. **Wave 4 exit**: confirm design-docs-plus-follow-ons is an acceptable
   close for #17/#180 if the gauntlets say the builds are large.

## Exit criteria

Every in-scope issue closed or carrying an owner-ratified disposition;
head PR merged by the owner; CI green; residue sweep + retrospective filed
in this library; estate mounts, canonical clone, and installed binary
resynced to merged main. Reference budget: the estate-root run spent
~5.6M subagent tokens over 7 phases; this sprint is broader but shallower —
expect the same order of magnitude.

---

## Kickoff rulings (owner, 2026-08-20 grilling — supersede the wave table above where they differ)

Grilled per `skills/grilling`, one decision at a time, each researched to
ground truth first (the interview twice found issue framing untrustworthy:
#166's "eight required sections" was unported upstream heritage, #167's
reconciliation need dissolved with the shell tool — see the
`issues-are-leads-not-specs` lesson).

1. **#166**: `sgt run --intent-file` ships as pure-content transport with
   mechanical guards only (size, path-traversal, symlink, encoding). No
   core schema, no validator. The eight-dimension risk brief (Objective,
   Required Invariants, Approved Tradeoffs, Out Of Scope, State
   Transitions, Failure Windows, Negative Test Matrix, Validation
   Evidence) becomes Captain pre-dispatch discipline in AGENTS.md; the
   classify-risk stage keeps its keyword trigger and points at that
   discipline, shedding the mandate/validator ghosts. Sgt executes; it
   does not validate intent.
2. **New wave — workflow authoring**: #201 (`validate-intent`, filed at
   kickoff), #88 (`record-decisions`), and the doctrine edits (#166's
   classify-risk pointer, #167's monitor-stage rewrite, #180's wording)
   ride together.
3. **#167**: closes with no verb — reconciliation is engine-owned
   (`runtime::recovery`, the `reconcile_*` family, `execution.reconciled`
   journaled; parked work preserved by rule). The upstream `--sync-all`
   need was file-drift repair for a daemon-less shell tool. Doctrine
   rewrite describes the real mechanism; pre-dispatch obligation is
   observation via read verbs.
4. **#112**: optional forge-neutral `upstream = "<url>"` on `[[repo]]`;
   admission/populate ensures a matching `upstream` git remote on the
   mount (manifest authority; `origin` untouched; worktrees inherit).
   Local-only repos: nothing changes. The assumption is git, never a
   specific forge CLI — no gh/glab/tea logic anywhere. Refines the
   estate-root disposition row: forge-*specific* convenience stays out of
   core; a declared forge-neutral remote is just git.
5. **#180**: ratified — observation-plus-honest-reporting IS the
   isolation contract (mutation surface declared, drift observed and
   journaled, violations charged as dirty evidence; prevention/sandboxing
   remains a non-goal). Closes in-sprint by truing up NORTH-STAR/doctrine
   wording, naming shared-mount collision as accepted risk. Wave 4 is
   #17 alone, design-may-close bar confirmed.
6. **#4**: journal-backed bounded cache on Rule A's `terminal_runs`
   pattern — count-bound, no clocks, evicted state re-derived from the
   journal; `work list` keeps full history via a slim index. Spec
   proposes capacity and list-view treatment; owner ratifies at the
   wave-2 PR.
7. **Standing constraint (from #4, applies to #17 and all defaults):
   settings are argued for the platform targets — Linux, macOS, WSL —
   never tuned to Cerberus.** This host is a measurement point, not the
   design target.
8. **#80**: both halves ruled. (a) One doctor detail line when an
   explicitly-set `$XDG_DATA_HOME` is outranked by the estate rung —
   kept; edge case, rightfully included. (b) Precedence ratified:
   `--data-dir` > `SGT_DATA_DIR` > manifest `data_dir` (invocation-
   explicit beats declared); replace ADR 0008's dangling GAUNTLET.md
   pointer with the real ruling and pin the order with a test. #80
   closes in wave 1.
9. **Release-ready head PR**: the finalize phase delivers the version
   bump (number proposed in the PR for owner ratification), the CHANGELOG
   section (Gate A), and a synced `Cargo.lock`, so the owner's path is
   review → merge → tag → release. #200's `make_latest` fix lands in
   wave 1, so that release is the first through the repaired pipeline —
   its post-publish assertion proves the Latest badge moved.
10. **Launch**: same overnight autonomous protocol as the estate-root
    run, confirmed. Fires on the owner's go.
