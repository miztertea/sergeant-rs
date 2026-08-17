# 00-classify-dependencies: classify dependencies

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

A four-way classification determines whether a port is needed at all.

Trigger (workflow-level): A module's interface needs redesign, or a port/adapter decision needs to be made deliberately rather than by default.

## What must become true here (durable outcome)

A four-way classification determines whether a port is needed at all.

## Behavior contract

- **When a deepening candidate's dependencies are pure in-process computation with no I/O, always merge the modules and test the result directly through the new interface; no adapter is needed.**
  (trigger: a deepening candidate's dependencies are classified as in-process; outcome: the modules are merged and tested at the new interface with no adapter layer)
- **When a deepening candidate depends on something with a local test stand-in (e.g. an in-memory filesystem or an in-process database emulator), deepening is possible and the deepened module is tested against that stand-in inside the test suite, with the seam kept internal.**
  (trigger: a deepening candidate's dependencies are classified as local-substitutable; outcome: the deepened module is tested with the stand-in, without exposing a port at the external interface)
- **When a deepening candidate depends on the team's own remote services, define a port (interface) at the seam owned by the deep module, inject an in-memory adapter for tests and an HTTP/gRPC/queue adapter for production.**
  (trigger: a deepening candidate's dependencies are classified as remote-but-owned; outcome: logic lives in one deep module; transport is swappable via an injected adapter)
- **When a deepening candidate depends on a true third-party external service the team doesn't control, inject that dependency as a port and give tests a mock adapter.**
  (trigger: a deepening candidate's dependencies are classified as true-external; outcome: the module is testable without calling the real third-party service)

- **Seam discipline: one adapter is a hypothetical seam, not yet worth exposing; two adapters (typically production plus test) justify making the seam real.**
  (trigger: deciding whether a classification result actually justifies exposing a port; outcome: a port is only exposed once at least two adapters genuinely need it, not on the mere possibility of a future second one)
- **Internal seams (private, test-only) are not exposed through the public interface merely because tests happen to use them.**
  (trigger: deciding what belongs on the deepened module's public interface; outcome: test-only access points stay internal rather than leaking into the public contract)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Classifying a dependency into one of the four categories (in-process, local-substitutable, remote-but-owned, true-external) and applying the matching adapter strategy.
- Whether a classification result actually justifies exposing a port (two-adapter threshold) and what belongs on the public interface versus staying internal.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only when the dependency classification is made and the matching adapter strategy (or "no adapter needed") is determined.

### Decision evidence
The classification and its rationale are this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
