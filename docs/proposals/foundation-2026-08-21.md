# Sprint plan — Foundation close-out: release pipeline + simple retention (2026-08-21)

> **PANEL-AMENDED.** The "Panel amendments (binding)" section at the end
> was produced by the 5-seat adversarial panel + refuters (6 confirmed of
> 17 raised; `plan-panel-results.json` beside this plan) and SUPERSEDES any
> conflicting sentence in the wave descriptions below. Wave specs must
> implement the amendments.

Owner-commissioned overnight sprint (commissioned in-session 2026-08-21, after
the v0.1.2 release): *"autonomously build end to end the completed release
pipeline that will work and the archive policy stuff. This should finish our
foundational work."* Ships as **0.1.3**. Owner merges main; nothing merges to
main during the sprint.

**Spec sources (J3, in authority order):**
1. The Q1–Q10 rulings record on issue #17
   (issue comment 2026-08-21) — **supersedes**
   `docs/proposals/journal-archival-rule-c.md` wherever they conflict,
   including the proposal's §7 non-goal "no retention surface in
   sergeant.toml" (owner explicitly overruled it: `retention = 1000` in
   sergeant.toml IS the ruling).
2. `docs/proposals/journal-archival-rule-c.md` for mechanism detail the
   rulings don't contradict (floor mechanics, I9 pinning test, error taxonomy).
3. release.yml's own settled-deviation header (dist tag-watcher topology
   REJECTED, human dispatch is the only release authority) — J5, not
   re-litigated.
4. This sprint's 5-agent live recon (`recon-results.json` beside this plan) —
   ground truth where docs disagreed with code.

**Protocol** (same as backlog close-out and cicd-hardening): integration
branch `integration/foundation`, draft head PR carrying this plan, wave
branches `foundation/w<N>-<slug>` in `/var/tmp/foundation-impl/` worktrees,
per wave: recon → spec → implement (TDD, DataDir guards, R-S0-12 full loop)
→ 4-axis blind panel (spec-fidelity / invariants / simplicity / test-honesty)
+ per-axis refuters defaulting to refuted → fixer on confirmed findings only
→ wave PR → merge to integration. Sonnet subagents by default, opus where
earned (named per wave below), Fable never in subagents. Rung citations
(R/J) in every wave PR body. W3 carries the full eight-dimension intent
brief (persistent-state + destructive territory, AGENTS.md INTENT).

## Recon corrections the specs MUST honor (code truth over doc claims)

1. **Startup is FOUR full-journal walks, not three** (daemon.rs:459-527):
   Journal::open's next_seq scan, registry.catch_up, Analytics::rebuild
   (graph folds inside it — not a separate pass), and
   `seed_capability_provenance`'s independent `replay_data_dir`. The
   proposal's §4.1 missed the fourth. Rung 0 must account for all four.
2. **`Replay::after` already implements floor-skip** (journal.rs:542-567,
   today only reached from api.rs tail catch-up) — W2 extends it (R2),
   never reinvents it.
3. **FloorState must cover capability provenance**: the Claude adapter's
   `ask_grammar` flag is rebuilt from its own full replay
   (claude.rs:824-868) and appears NOWHERE in the proposal's enumeration —
   a live instance of the I9 failure mode, caught pre-implementation.
   `admission_paused` is force-cleared at startup (daemon.rs:632-640) and
   needs no floor coverage — record that as an I9 allowlist entry with the
   reason.
4. **`command.accepted`/`command.rejected` are only SOMETIMES
   work-scoped** (api.rs:1065-1067 attaches work_id conditionally):
   eligibility/allowlist logic must classify per-EVENT (work_id presence),
   never per-kind, for these two. The "seven kinds" in the rulings record
   are five always-non-work kinds plus these two conditionals.
5. **Two terminal caches, not one**: terminal_runs capacity 512 (settled)
   and terminal_works capacity 1024 — the 1024 is still marked "Proposal,
   for owner ratification" in code (projection.rs:378). Ratify-at-review.
6. **Blobs are content-addressed and DEDUPLICATED** (write-once by hash):
   two Works can share one blob file. Prune must never delete a blob still
   referenced by any RETAINED Work's events — blob deletion is a
   mark-and-sweep over retained events' `b3:` references (payload fields
   `raw`/`raw_error`/`stdout`/`stderr` — docker.rs:1008-1017,
   claude.rs:1391-1421,1551), not a per-Work delete list.
7. **Lock-free readers race deletion**: `replay_data_dir` takes no lock and
   runs from separate processes (doctor, cli.rs:3685) while the daemon
   could prune. POSIX unlink keeps open FDs valid; the exposure is
   list-then-open (NotFound mid-iteration) and floor-coherence. W3 must
   specify reader behavior under concurrent prune; W4's doctor path must
   tolerate it.
8. **No compound-event precedent exists** (recon: every L6 case uses
   group-commit-with-tolerant-replay or re-derivation). The prune protocol
   is new-pattern territory; the spec chooses the L6 answer explicitly
   (compound event vs intent/completion pair + startup completion per Q9)
   and defends it — deletion is not re-derivable after a partial crash,
   which is why teardown's re-derivation answer doesn't transfer.
9. **sergeant.toml is `deny_unknown_fields`** (estate.rs:709-723): adding
   `retention` touches the serde schema AND the toml_edit pen
   (manifest.rs); an 0.1.2 binary refuses a manifest carrying the new key —
   acceptable (one developer per installation, AGENTS.md), stated in the
   ADR.
10. **`work_transcript` does a full seq-0 replay under the exclusive
    CoreGuard on every call** (api.rs:2154-2206, disclosed §22.6 tradeoff)
    — W4's read-path work bounds it to the Work's own segments via the
    index (`last_seq`/first-seq range), or explicitly re-discloses why not.
11. **Gate A rejects ALL non-main dispatches** (release.yml:112-122), both
    modes — today the packaging chain is unprovable from a branch. W1
    changes exactly this, for dry-run only.
12. **The four remaining `gh api` steps in release.yml are cited R7s**
    (porcelain has no by-ID verbs; drafts are unreachable by tag —
    verified against installed `gh`). W1 does NOT churn them; it documents
    the citation inline where missing.

## Waves

### W1 — Release pipeline completion (small, first, independent)
The pipeline is already mostly Lego (recon): `gh release create --draft` is
porcelain, dist is build-only per the settled §7 deviation, #215 moved
Latest-marking to porcelain. Remaining work:
- **Absorb PR #215** (merge `fix/publish-latest-porcelain` into the wave
  branch; #215 closes as superseded at finalize with a comment).
- **Gate A dry-run-from-branch** (the owner's "every CI change takes an
  hour" pain, and this sprint's own proof mechanism): when
  `mode == 'dry-run'`, permit dispatch from any ref and skip the
  SHA==origin/main assertion; `publish` keeps the full main-only +
  HEAD-current check unchanged (J5 release authority intact — dry-run
  creates no tag, no publish, draft deleted on `always()`; recon verified
  drafts create no git ref). Cite J4 (owner commission: "the completed
  release pipeline that will work" requires proving it pre-merge) — and
  flag as ratify-at-review item 1.
- Version-literal steps in Gate A already accept 0.1.3 once the bump lands
  (checks are input==Cargo.toml, tag-absent, CHANGELOG-contains).
- Sweep release.yml/ci.yml comments that describe the pre-#215 publish
  shape; add the missing R7 citations at the four `gh api` sites (comment
  only, no behavior).
- **Proof**: at finalize (after the 0.1.3 bump lands on integration),
  dispatch `release.yml` `mode=dry-run` FROM `integration/foundation` —
  the full gates + packaging + draft + delete chain must go green. That
  run is the head PR's evidence that "it's all going to work."
  (The publish-mode split PATCH → `gh release edit --latest` step remains
  provable only at the real 0.1.3 publish; its mechanism was proven live
  on v0.1.2 by hand — stated honestly in the PR body.)

### W2 — Startup: single-pass replay + FloorState cache (opus-earned spec)
Rulings: Q1 (rung 0), Q2 (cache permitted, pure-cache semantics, rebuild
flag), Q4 (fixed window 16 segments / 128 MiB).
- **Rung 0**: one shared replay feeds registry catch-up, Analytics rebuild,
  and capability-provenance seeding; Journal::open's next_seq scan bounded
  to the last segment or folded into the same pass (spec's call — R2:
  `Replay::after` + `first_seq` already exist). Measured before/after on
  the dogfood journal recorded in the wave PR.
- **FloorState cache** (name per spec; "pure cache" is the contract):
  per-Work rows (WorkIndexRow fields + new `last_seq`), command-ledger
  keys (command_id → status-class + work_id for submits), capability-
  provenance watermark (recon correction 3), each bound to the exact
  segment set + per-segment BLAKE3 it summarizes. Absent/mismatched ⇒
  full replay (never an error). Written after a successful full rebuild
  and updated at prune (W3 hook — W2 lands the format + read/write +
  fallback; the floor stays 0 until W3 exists, so W2 alone changes no
  read semantics below the window).
- **Startup window**: replay only the newest 16 segments / 128 MiB via the
  cache + `Replay::after`; cache supplies everything older.
- **`--rebuild-cache` flag** on foreground daemon start (cli.rs Daemon
  variant, threaded through run_until_signal → start_with): ignore and
  rewrite the cache. (No cache existed before this wave — the flag arrives
  with the thing it toggles.)
- **I9 pinning test**: fold full replay vs fold windowed replay + cache;
  assert registry equality field-by-field against an enumerated allowlist
  (admission_paused enters the allowlist with the force-clear reason).
  This test is the wave's acceptance gate — it fails the build when anyone
  adds registry state the cache doesn't carry.
- §26 below-window semantics (Q8): retried command_id older than the
  window → refuse-by-name from cache keys (submit refusals name the Work).
  Byte-identical replay preserved for everything in the window.

### W3 — Prune engine (eight-dimension intent; destructive territory)
Rulings: A2/Q3 (retention model, per-Work atom, journaled, declared policy
is the authorization), Q5 (per-event allowlist, proven in-milestone), Q6
(blobs die with Works), Q7 (stall loudly, no force flag), Q8 (ledger
exemption), Q9 (crash-completion at startup).
- **Knob**: `retention = 1000` under `[estate]` in sergeant.toml (serde +
  toml_edit pen + validation ≥ some floor, spec proposes; absent ⇒ default
  1000). Per-event eligibility: a segment is prunable iff every work-scoped
  event's Work is terminal AND past the newest-1000 cap AND every
  non-work-scoped event in it is allowlisted (recon correction 4: the two
  command kinds classify by work_id presence; command-ledger entries are
  EXEMPT — their keys live in the cache forever regardless of segment
  fate).
- **Allowlist proofs**: replay-equivalence test per kind for
  daemon.started, daemon.stopped, backend.probed, admission.paused,
  admission.resumed (+ the two conditional command kinds' non-work
  instances) — each proves a windowed replay reaches the same registry
  state as full. All land in this wave; pruning ships strict AND effective.
- **Protocol** (spec chooses and defends the L6 answer, recon correction
  8): journaled intent → blob sweep decision → segment unlink(s) → journaled
  completion, with startup completion of an interrupted prune (Q9 — mirrors
  recovery.rs's evidence-based completion, never suspicion-based deletion;
  recovery.rs:118-123's refusal stays the boundary). Floor/cache updated
  atomically with the completion record. Lock-free-reader coherence
  specified (recon correction 7).
- **Blob sweep**: mark = all `b3:` refs in RETAINED works' events (the
  window + cache index tell which segments hold retained works); sweep =
  blobs referenced only by pruned Works. Dedup-safe by construction
  (correction 6). Journaled counts in the completion event.
- **Stall visibility**: prune returns/records the blocking Work (oldest
  non-terminal or in-cap Work pinning the oldest segment) — surfaced by
  W4's doctor check. No override exists.
- **Trigger**: automatic at daemon start after cache load + on rotation
  crossing the cap (spec refines; policy-in-manifest is the authorization
  per A1 — no `--yes` ceremony for policy-driven prune; a manual
  `sgt journal prune --dry-run/--yes` verb is in scope ONLY if the spec
  shows it near-free atop the engine, else deferred with a line in the ADR).

### W4 — Surfaces, doctrine, finalize
Rulings: Q10 (show what we have, three-zone honesty), A1 (ADR 0003
amendment), plus release-readiness.
- **Read paths**: `GET /v1/events` + SSE catch-up serve from the floor with
  an explicit floor marker in the response body (`truncated_below` or spec's
  name); SeqDiscontinuity keeps its corruption-only meaning; SSE's silent
  close on journal error gets the marker treatment only if near-free
  (pre-existing gap, recon-noted — else documented, not fixed, this
  sprint). `work show` on a pruned Work: named answer ("pruned on <date>
  under policy", derived from the prune event's record) — never a blank
  404, distinguished from never-existed. `work transcript` reads bounded
  to the Work's own segment range via the index (correction 10) — or the
  §22.6 disclosure updated with why not.
- **Doctor**: `journal_growth` check beside disk_pressure — live-segment
  count/bytes, floor, retained-Work count vs cap, blocking Work + age when
  pruning is stalled, last rebuild duration (recorded on `daemon.started`
  payload by W2's rung-0 pass) with warn ≥10s / fail ≥30s.
- **Doctrine**: ADR 0003 amendment (per-Work integrity while retained +
  bounded retention + silent-deletion-is-the-crime, Work as atom); new
  retention ADR (the knob, the default 1000 with the measured 1.8 MB/Work
  basis, eventually-exact segment granularity, ledger exemption, crash
  completion); `docs/proposals/journal-archival-rule-c.md` gets a
  disposition header pointing at the rulings record + ADRs (superseded-by,
  not deleted); version-policy.md untouched unless a claim broke.
- **Finalize** (same wave, after panel): version 0.1.3 in Cargo.toml +
  Cargo.lock sync, CHANGELOG 0.1.3 section (release pipeline + retention),
  README consistency re-check, `docs/proposals/foundation-2026-08-21.md`
  (this plan) on the integration branch, retro in the workspace evidence
  dir, machine cleanup (worktrees, /var/tmp build dirs, stray daemons,
  scratch estates), **the W1 proof dry-run dispatched from
  `integration/foundation`**, head PR un-drafted with wave checklist,
  rung citations, ratify-at-review items, and the dry-run link.

## Wave ordering & conflict control
W1 → W2 → W3 → W4 strictly serial. W1 is independent (workflows only) but
runs first so the dry-run-from-branch relaxation exists before finalize
needs it. W2→W3→W4 all touch daemon.rs/journal.rs/projection.rs —
conflict-forced serial. Each wave rebases on integration head before its PR.

## Ratify-at-review items (owner, at head PR)
1. Gate A dry-run-from-any-branch relaxation (publish unchanged) — the
   authority-posture change W1 makes; revert is one `if:` guard if refused.
2. TERMINAL_WORK_CACHE_CAPACITY = 1024 — code's own comment awaits this
   ratification (recon correction 5).
3. `retention = 1000` default + validation floor the W3 spec proposes.
4. Manual `sgt journal prune` verb: shipped or deferred (whichever the W3
   spec chose) — confirm the choice.
5. The L6 protocol shape W3's spec chose for the prune record.

## Risks
- **Deletion is the sprint's irreversible surface** — mitigated by: W3's
  eight-dimension brief, the compound/intent-completion protocol with
  crash tests, blob mark-and-sweep dedup safety, the I9 pinning gate from
  W2 landing BEFORE any deletion code exists, and the panel's invariants
  axis + refuters on every wave.
- **Dogfood estate**: the workspace's live estate runs the current binary;
  nothing in this sprint touches its data dir. Tests use DataDir guards
  exclusively; the perf measurement reads the dogfood journal READ-ONLY.
- **Dry-run proof depends on W1's Gate A change being correct** — if the
  relaxation itself is buggy, finalize's dispatch fails loudly and the head
  PR says so rather than papering over it.
- **tmpfs**: all builds in `/var/tmp/foundation-impl/`; nothing builds in
  the session scratchpad (standing rule after the 08-20 quota outage).

## Panel amendments (binding — 2026-08-21 panel, 6 confirmed / 11 refuted)

**A1 (BLOCKER, invariants) — every full-replay consumer becomes
floor-aware in the SAME wave PR as the first deletion code path.**
`Replay::new` hardcodes `expected = 1`; after the first prune no segment
starts at seq 1, so every `Journal::replay()` / `replay_data_dir()` caller
would fail `SeqDiscontinuity` — misclassifying ruled retention as
corruption, which A2 forbids. W3's PR must audit and convert every caller:
`rederive_registry_for` (below-floor Works route to the pruned-by-name
answer, retained-but-evicted Works rederive from the floor), doctor's
`journal_check`, the other CLI `replay_data_dir` sites, and W2's own
cache-miss fallback. "Full replay" is REDEFINED everywhere as "replay from
the oldest surviving segment's `first_seq`" (`Replay::after` already
computes this — R2).

**A2 (BLOCKER, deletion-safety) — horizon predicate; a prune may never
bisect a Work.** The per-segment content predicate in the W3 sketch can
delete an early segment of a Work whose later events survive. Replace with:
compute horizon **H** = max seq such that every Work having ANY event with
seq ≤ H is terminal AND past the cap AND has `last_seq ≤ H`; prune only
segments whose last seq ≤ H. A Work with `last_seq > H` pins everything —
that Work is what the Q7 stall report names.

**A3 (BLOCKER, deletion-safety) — the prune completion event carries the
residue; the journal stays self-describing and the cache stays pure.** The
completion event records, for every pruned Work: the slim index row
(WorkIndexRow fields + prune date) and its command-ledger keys
(command_id → status-class, work_id for submits). Consequences: (a) a
replay from the floor reconstructs the §26 ledger and the pruned-by-name
answers with NO cache — Q2's "absent/mismatched ⇒ replay, never an error"
stays literally true after any number of prunes; (b) `--rebuild-cache`
after pruning is safe; (c) Q8's exemption is journal-durable, not
cache-durable. This resolves the rulings-fidelity MAJOR (cache-loss
reopening the duplicate-Work hole) by construction. The retention ADR
states it.

**A4 (BLOCKER, deletion-safety) — blob mark scan is a recursive payload
walk + a pinning test, landed in W2.** Blob refs hide inside nested JSON
strings in event payloads (docker detail JSON), so a flat field scan marks
nothing there and would sweep live blobs. The mark scan recursively walks
payload values, matches `^b3:[0-9a-f]{64}$`, and recurses into strings
that parse as JSON. A blob-ref pinning test (same shape as I9) enumerates
every `BlobStore::put`/`put_stream` call site and asserts the extractor
recovers the ref from a real emitted event — lands in W2, BEFORE any
deletion code exists, so a future capture site cannot create unmarkable
blobs.

**A5 (MAJOR, deletion-safety) — mark/sweep vs live dedup adoption needs
stated mutual exclusion.** Between mark and sweep, a live Work can adopt
(dedup-hit) a condemned blob. W3's spec chooses and defends one mechanism;
default: two-phase quarantine — sweep renames condemned blobs to
`blobs/b3/.pruned/<hex>` (recorded in the completion event);
`BlobStore::get` falls back to `.pruned/` and a hit rescues the blob back;
the NEXT prune cycle deletes `.pruned/` entries untouched since the prior
completion. If the spec picks core-guard exclusion instead, it must also
make the dedup hit re-materialize a vanished file — the guard alone is
insufficient.

**A6 (MAJOR, feasibility/simplicity fold-in) — recon correction 6's field
list was wrong** (`stdout`/`stderr` are inside nested detail JSON, not
top-level payload fields); corrected by A4's recursive walk. The cut lines
if the night runs short, in order: the manual `sgt journal prune` verb
(defer), transcript read-bounding (disclose instead), SSE marker
(document as pre-existing gap) — never the I9 gate, the floor-awareness
audit (A1), or the crash tests.
