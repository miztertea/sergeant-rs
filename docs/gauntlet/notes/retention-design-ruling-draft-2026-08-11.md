# Retention design ruling — ADJUDICATED 2026-08-11 (R-N0-3's gate satisfied)

Status: **adjudicated.** Owner rulings 2026-08-11 (same-day session,
recorded verbatim in intent): Rule A accepted; Rule B accepted with the
measurement-vehicle clarification and the fake-backend fidelity review
queued to the H-series (H0 packet); Rule C accepted AS AMENDED below —
the rebuild-time trigger is the binding one, the design ladder when it
fires is compress-cold-segments-first (R2) before any snapshot machinery
(R7), and the 20 GiB disk figure is demoted to a blob-store backstop
(measured arithmetic: at ~549 B/event the journal needs ~39M events to
reach 20 GiB; run 4 — the largest run ever — wrote 186 kB; the realistic
disk driver is blobs, not journal text). R-N0-3's precondition on the N4
contract is hereby met. Original draft text below, amended in place where
the rulings changed it.

The shape R-N0-3 licenses: a ruling may be "measured budget X, GC deferred
again with trigger Y" — silence is the only illegal state. Each rule below
names its rung and its trigger.

## Rule A — #4: terminal-work projection eviction, lands IN N4 (R2)

The only measured unbounded **memory** growth is the in-memory projection
retaining every terminal work forever (~21–25 kB/work, zero reclaim,
monotonic — s2-churn, two independent runs). Rule: terminal works
(`completed`/`canceled`/`failed` with no live execution) evict their full
projection state after settle, keeping a light terminal index entry (id,
terminal state, timestamps, workflow name — the list-view fields).
`work show` on an evicted work re-derives the full view from the journal
on demand (rebuild machinery already exists and is the architecture's
whole thesis — R2: reuse, no new truth). Not a cache with invalidation:
terminal works are immutable by definition, so re-derivation is always
correct.

Acceptance shape for the N4 contract: s2-churn's RSS slope goes ~flat
post-settle; evicted `work show`/API/TUI views byte-identical to
pre-eviction; restart recovery indifferent to eviction state; the §22.5
window between terminal settle and evict needs no new event (eviction is
in-memory only — crash loses nothing, rebuild just doesn't re-retain).

## Rule B — blob-store disk cost: measured budget + doctor visibility in N4; GC deferred with a hard trigger (R1 + measurement)

Execute stages (N4's addition) stream arbitrary logs into the blob store —
§16.9 caps the journal cost at O(1) events, not the disk cost. Rule:

- N4's own contract tests **measure** per-execute-stage blob cost under a
  1 GiB log capture (the §22.8 budget already binds peak RSS; this adds
  the disk-cost measurement beside it). **Adjudication clarification:**
  the measurement vehicle is the real local Docker engine with a
  Sergeant-built probe image (deterministic, token-free), NOT the fake
  backend — the fake stands in for actor stages only. The same exchange
  surfaced that the fake backend's fidelity gap (it settles turns at
  launch — the blindness that hid #46) needs its own review; queued in
  the H0 packet.
- `sgt doctor` grows the disk-pressure check #17 itself asks for (#23
  folded in here, its trigger fired): data-dir size, blob-store share,
  free-space headroom, growth since last daemon start. Visibility, not
  policy (R1 — doctor already exists; this is one more probe).
- **Blob GC stays deferred** with the trigger made hard: a blob-GC design
  contract fires when any measured data dir crosses 20 GiB blob share, or
  when the first real deployment wants to delete a work's evidence (a
  *policy* question — what "retire" means — that engineering must not
  answer unilaterally). Content-addressing makes eventual GC a
  mark-sweep from journal refs; nothing in N4 may make that harder
  (acceptance: no blob written without a journal ref that names it).

## Rule C — journal segment archival: deferred (AMENDED at adjudication: rebuild-time trigger binding; compress-first ladder; 20 GiB → blob backstop)

**Owner amendment 2026-08-11:** the binding trigger is measured
rebuild-on-start exceeding 30 s (at Cerberus's 54.6k events/s that is
~1.6M events — reachable; the 20 GiB journal figure is not, at ~39M
events, and is demoted to a blob-store backstop under Rule B). When the
trigger fires, the design contract's ladder is pre-committed: (1)
compress cold segments in place (R2 — segments already rotate at
~8.4 MB; text compresses ~10×; replay decompresses; no identity-binding
machinery), and only if compression cannot hold the line, (2) the
snapshot/truncate design (R7, B1's dormant machinery, which stays
dormant until then). Original draft rationale follows.

### (original draft text)

Segments interleave works, so archiving "old" segments while rebuild-on-
start replays the full journal is a contradiction — segment retirement
requires a load-bearing snapshot/checkpoint, which is exactly the B1
machinery ruled dormant (identity binding and all). Reintroducing it for
disk economics the measurements don't yet justify would be R7 with no
failed lower rungs. Rule: deferred. Triggers (either fires the design
contract): measured rebuild-on-start exceeding 30 s on real history, or
journal share of the data dir exceeding 20 GiB. N4 adds the missing
measurement the issue names: **rebuild at 1M events** (the 50k-event
point measured 1.7 s / 178 MB; the 1M point tells us whether the trigger
is years away or quarters away). DuckDB rides the same disposition — it
is a projection, rebuilt from truth, so its growth is bounded by the same
horizon and needs no policy of its own yet.

## What this ruling deliberately does not do

No retention configuration surface (no `sergeant.toml` knobs) — policy
knobs without a GC to obey them are dead weight (R1). No age-based
expiry — "durable trajectory" is the invariant, and nothing measured
yet justifies weakening it. No projection snapshotting — B1 stays
dormant on its registered trigger.

## Adjudication asks (owner)

1. Accept/amend Rule A's landing in N4 (it is the one code change).
2. Accept/amend Rule B's 20 GiB trigger and the #23 fold-in.
3. Accept/amend Rule C's 30 s / 20 GiB triggers and the 1M-event
   measurement as an N4 gate item.
