# 60-cleanup-and-postmortem: cleanup and postmortem

## Inputs

| File | Layer | Why |
|---|---|---|
| ../50-fix-with-regression-test/output/README.md | L4 | upstream artifact produced by `50-fix-with-regression-test` |

## Purpose

Repro gone, test passing, instrumentation removed, hypothesis recorded, architectural hand-off if warranted.

Trigger (workflow-level): "Diagnose"/"debug this", or something reported broken, throwing, failing, slow.

## What must become true here (durable outcome)

Repro gone, test passing, instrumentation removed, hypothesis recorded, architectural hand-off if warranted.

## Behavior contract

- **Before declaring the diagnosis done, the actor must confirm: the original repro no longer reproduces, the regression test passes (or the seam absence is documented), all `[DEBUG-...]` instrumentation is removed via a prefix grep, throwaway prototypes are deleted or clearly marked, and the correct hypothesis is stated in the commit/PR message for the next debugger.**
  (trigger: a fix has been applied; outcome: a fixed set of closing conditions is verified before the diagnosis is considered complete)
- **After the fix is in, the actor asks what would have prevented the bug; if the answer involves architectural change (no good test seam, tangled callers, hidden coupling), the actor records the architectural finding and recommendation in this stage's own output, making the recommendation only after the fix — not before, since more is known by then.**
  (trigger: the fix and cleanup checklist are complete; outcome: an architectural-improvement recommendation is durably recorded, timed deliberately after the fix)
  **Corrected 2026-08-16, ICM-R3:** the prior text named a hand-off to "the `/improve-codebase-architecture` skill" — no skill or workflow by that name, or functionally equivalent to it, exists anywhere in this repository (carried over verbatim from the upstream source at N1 extraction). The actual required behavior — flag the recommendation for the next debugger — is fully satisfiable by recording it in this stage's own `promote`-disposition output; it does not require a named downstream invocation target.

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Judging whether the fix implicates an architectural change worth flagging.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only when the closing checklist is verified: repro gone, test passing (or seam absence documented), all `[DEBUG-...]` instrumentation removed via prefix grep, throwaway prototypes deleted or marked, and the correct hypothesis stated in the commit/PR message.

### Decision evidence
The closing checklist and any architectural recommendation are this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
