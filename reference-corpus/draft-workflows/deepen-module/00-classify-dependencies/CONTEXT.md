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
  — `BU-P4-014`, `reference/sergeant-upstream/.agents/skills/codebase-design/DEEPENING.md` (Dependency categories / In-process, L11)
- **When a deepening candidate depends on something with a local test stand-in (e.g. an in-memory filesystem or an in-process database emulator), deepening is possible and the deepened module is tested against that stand-in inside the test suite, with the seam kept internal.**
  (trigger: a deepening candidate's dependencies are classified as local-substitutable; outcome: the deepened module is tested with the stand-in, without exposing a port at the external interface)
  — `BU-P4-015`, `reference/sergeant-upstream/.agents/skills/codebase-design/DEEPENING.md` (Dependency categories / Local-substitutable, L15)
- **When a deepening candidate depends on the team's own remote services, define a port (interface) at the seam owned by the deep module, inject an in-memory adapter for tests and an HTTP/gRPC/queue adapter for production.**
  (trigger: a deepening candidate's dependencies are classified as remote-but-owned; outcome: logic lives in one deep module; transport is swappable via an injected adapter)
  — `BU-P4-016`, `reference/sergeant-upstream/.agents/skills/codebase-design/DEEPENING.md` (Dependency categories / Remote but owned, L19)
- **When a deepening candidate depends on a true third-party external service the team doesn't control, inject that dependency as a port and give tests a mock adapter.**
  (trigger: a deepening candidate's dependencies are classified as true-external; outcome: the module is testable without calling the real third-party service)
  — `BU-P4-017`, `reference/sergeant-upstream/.agents/skills/codebase-design/DEEPENING.md` (Dependency categories / True external, L25)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
