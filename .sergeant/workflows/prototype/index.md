---
kind: workflow
name: prototype
status: published
version: 1
description: >-
  Build a throwaway prototype to answer a design question, branching between logic and UI questions.
tags:
  - prototype
  - design
  - throwaway
---

# Prototype

Six-stage actor-only workflow (N1 candidate **W21**, `docs/gauntlet/contracts/N1.md`) that builds a throwaway prototype to answer one design question, branching at `00-select-branch` between a logic/state investigation (`20L-build-logic`) and a UI-variants investigation (`20U-build-variants`) before handing off and capturing the validated result into production code. Use when: the user wants to sanity-check whether a state model or logic feels right, or explore what a UI should look like.

See `CONTEXT.md` for workflow orientation (including the reviewer note on the `20L`/`20U` branch shape) and `workflow.toml` for the pinned stage order. The full behavior-unit citation trail is archived at `docs/gauntlet/promoted-provenance/prototype.md`, per the promotion procedure in `docs/icm/promotion-spec-2026-08-11.md`.
