# Consequence-class sweep checklist

Layer 3 (stable across runs), local to `20-harvest`. `40-classify` and
`80-adversarial-review` also point back to this file — see
`../../_config/icm-ladder.md` and
`../../80-adversarial-review/references/challenge-checklist.md`.

## Why this exists

N2 run 2 extracted 108 behavior units from 16 files, cited every one of them
correctly, and still let 11 reference behaviors carrying safety, identity,
recovery, delivery, or human-decision consequence go silently absent —
not because the files weren't read (all 16 were), but because nothing was
specifically hunting for these five classes
(`docs/gauntlet/runs/n2-run2/comparison-scorecard.md` §3, the §22.2 finding).
Two of the sharpest misses were one-sentence `AGENTS.md` guardrails inside a
file this run's actors extracted 23 other units from — proof that ordinary
"read the file, extract what stands out" attention is not enough on its
own. A behavior that gates a destructive or irreversible action, or that
draws a boundary around what an authorization does *not* cover, must never
be silently absent from this stage's output the way it was here.

## This is mandatory, in addition to ordinary extraction

For **every** `decompose` file, once its ordinary one-behavior-per-unit
extraction (`../CONTEXT.md`) is otherwise complete, deliberately re-scan the
same text — it is already open, this is not a second file read, it is a
second *lens* on the same read — against the five hunt questions below, and
record one row per file in `output/consequence-class-sweep.md`
(`references/partition-checkpoint-protocol.md` step 3). A file's absence
from the sweep file is a coverage gap; a row that is blank in any column is
indistinguishable from "forgot to check" and is itself a defect in this
artifact — every cell is either a `behavior_id` (or list) that already
covers that class for that file, or the literal text `swept, none found`.

```text
| File | Safety | Identity | Recovery | Delivery | Human-decision |
|---|---|---|---|---|---|
| bin/sgt-drain-force | BU-0071 (start-time/PID-reuse check before signal) | swept, none found | swept, none found | swept, none found | swept, none found |
| AGENTS.md | swept, none found | swept, none found | swept, none found | swept, none found | BU-0014 (standing-authorization scope) |
```

## The five hunt questions

Ask all five of every `decompose` file, regardless of how "obviously
behavioral" or "obviously mechanical" the file otherwise looks — the
sharpest misses in N2 run 2 were single sentences inside files whose other
content looked unrelated.

1. **Safety** — Does anything here gate a destructive, irreversible, or
   force-level action on a precondition — a verification check, a
   confirmation, an identity match — before it is allowed to proceed? Hunt
   specifically for: process-kill/signal code that re-checks the target is
   still the *same* thing it thinks it is (not merely "a process currently
   exists at this PID" — has anything changed underneath, e.g. the PID was
   reused since it was recorded?); anything phrased as force/drain-force/
   cleanup/retire that removes or discards state; any place a standing or
   blanket authorization is scoped with a "never authorizes X, Y, Z"
   sentence — the boundary matters exactly as much as the grant it limits.
2. **Identity** — Does anything here pin an identity — a model/variant, an
   execution or process handle, a worker/session identity, a repository or
   branch identity — that must be honored exactly, not silently substituted
   with an ambient default, on a *later* re-entry (resume, retry, recovery,
   response)? A check that fires once at creation and is never re-verified
   at resume time is a distinct behavior from the creation-time check
   itself — both are real if both exist; do not let finding the
   creation-time check satisfy the hunt for the resume-time one.
3. **Recovery** — Is there a durable, resumable record published *before* a
   destructive or interruptible action starts, so an interruption mid-action
   is safely retryable rather than leaving ambiguous half-done state? Is
   there a rule that reclassifies an ambiguous terminal condition (an empty
   result, missing substantiating evidence, an unexpected exit) as
   `orphaned`/`unknown` rather than silently accepting it as a clean `done`?
4. **Delivery** — Is there a guarantee that some action happens *exactly
   once* — not zero times, not twice — even across retries or restarts: a
   lease, a convergence point, a single shared finalizer that multiple paths
   must funnel through before acting? Is there a readiness gate that must be
   satisfied before delivering or notifying, with a stated behavior on
   timeout (fail closed, never fabricate success)?
5. **Human-decision** — Is there a fail-closed distinction between
   "temporarily blocked, may still resolve on its own" and "permanently
   unsatisfiable, escalate to a human" wherever a retry-vs-give-up decision
   is made? Is there a scope boundary stating what an authorization, mode,
   or standing permission does *not* extend to — a "never" rule sitting next
   to the grant it limits?

## What this does not change

The sweep does not relax `../_config/evidence-policy.md`'s one-behavior-per-
unit rule, and it does not license inventing a behavior the source does not
actually state — a hunt question that turns up nothing real is recorded as
`swept, none found`, not stretched into a unit the file doesn't support.
The point is attention, not fabrication: read the same text a second time
with these five questions specifically in mind, because ordinary extraction
attention already proved (N2 run 2) that it can miss a guardrail phrased as
one quiet sentence inside a much longer, mostly-mechanical file.
