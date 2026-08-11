# 01-verify-bash-compat-both-passes

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** the toolchain/task runner run test:docker:drain runs

**Outcome:** compatibility is proven under both the host's ambient Bash and the minimum supported Bash 3.2, not just one

**Statement (the operative rule):** The Docker drain test suite runs the same test file in two passes — system Bash under Debian bookworm-slim, and Bash 3.2 under an official Alpine bash:3.2 image — and reports overall pass/fail only once both passes are accounted for.

## What must become true here (durable outcome)

Compatibility is proven under both the host's ambient Bash and the minimum supported Bash 3.2, not just one — per the Statement above, which is the operative rule this stage exists to enforce.

