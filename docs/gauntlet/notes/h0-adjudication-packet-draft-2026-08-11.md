# H0 — Harness-adapter kickoff adjudication packet (DRAFT for the owner)

Governing input: `reference/proposal-harness-adapter-research-v2.md`
(owner-delivered 2026-08-11, vendored same day). Shape mirrors N0:
adjudication only, zero code delta; the product is rulings the H-series
contracts cite. Nothing below is settled until the owner rules.

## What the report asks the engine to become (its 8-item contract v2)

provider+transport identity; executable runtime ownership; separate
session/turn handles; event-driven settlement/wakeup; typed interactions;
typed orchestration capabilities; capability provenance/stability;
explicit auth/config mode. The report is explicit that this is "a
foundational refinement of the existing contract, not a new engine," and
that tonight's Cerberus work (the settle driver; the a5 runtime
withdrawal) already proved two of its generic requirements live.

## Proposed rulings (each needs an owner verdict)

- **R-H0-1 (D2 successor row):** `claude -p` remains the admitted default
  Claude transport — measured, stable, and now live-proven (Run B2's two
  autonomous cascades). Agent View supervisor and Agent SDK sidecar
  become *candidate transports admitted only by measurement* through one
  admission suite shared with Codex App Server / OpenCode Server. No
  transport ships on documentation. (Report's own recommendation; L1/L8
  writ large.)
- **R-H0-2 (sequencing vs N4):** decide the order. Option A: N4 (Docker
  execute) first as contracted, contract-v2 refactor after — pro: N4 is
  fully gated and its prerequisites landed tonight; con: N4 bakes more
  code against contract v1's shapes. Option B: contract-v2 seam first —
  pro: the report says it lets remediation land cleanly and every
  adapter (including N4's Docker executor, which is itself "a runtime
  strategy") fits the new seam; con: delays the milestone the whole
  N-series aimed at. **Orchestrator's recommendation: Option B-lite —
  land contract v2's items 1–3+8 (identity/ownership/handles/auth mode,
  the type-level half) as H1 before N4, and defer items 4–7's deeper
  machinery (push settlement, typed interactions/orchestration,
  provenance) to H2 after N4 — the poll driver from BS2 satisfies
  "every turn causes a settlement attempt" today, so push/event wakeup
  is an optimization, not a correctness gap.** The panel should attack
  this recommendation (L9).
- **R-H0-3 (D6 Codex unblock):** D6 deferred Codex "until an environment
  exists where Codex can be measured." Cerberus may be that environment
  — but presence must be probed, not assumed. Token-free spike: which of
  codex / opencode / goose binaries exist or are installable here; their
  versions; App-Server/ACP surface scans (--help level). Outcome feeds
  whether the H-series scopes any non-Claude adapter at all.
- **R-H0-4 (`--bare`):** per the report, model as a config/auth profile
  of the direct adapter (a `PermissionMode`-style profile axis), never a
  separate transport. Cheap; can ride any H1 contract.
- **R-H0-5 (auth boundary):** the report flags that the Agent SDK
  changes the auth/product boundary ("terminal credentials the human
  already authenticated" vs SDK auth). Ruling needed on whether that
  boundary is a constraint (supervisor-first) or a config axis
  (SDK admissible with explicit profile opt-in — the #47 pattern).
- **R-H0-6 (admission evidence standard):** adopt the report's
  documented → implemented → measured → admitted ladder as the
  capability lifecycle vocabulary; it is L1/L8 formalized and matches
  the runtime-withdrawal machinery the adapter already has.

## The admission suite (skeleton for the first H contract)

Per candidate transport, in cost order: (1) token-free surface scan
(binary presence, version, --help, machine-protocol availability);
(2) token-free protocol probe where the transport has a server/socket
surface (session inventory, event stream shape, health); (3) one bounded
spend turn measuring: turn identity, terminal outcome, usage capture,
interrupt semantics, resume; (4) the Run-B-shaped live workflow run
(bounded intent, budget-guarded — noting tonight's measured limit that
per-turn usage granularity bounds any guard from below; guards must be
sized to whole-turn costs). Every claim lands in the transport's
capability row with provenance per R-H0-6.

## Immediate cheap spikes (queued behind owner's go, all token-free)

- codex/opencode/goose presence + version probes on Cerberus →
  environment fact rows.
- `claude --bare -p --help` / Agent View supervisor surface scan
  (`claude --bg`, session inventory JSON) — surface shape only, no
  turns.

## What this packet deliberately does not do

No engine edits, no adapter spikes with spend, no D-register rows
written (rows follow rulings), no N4 reordering without R-H0-2's verdict.
