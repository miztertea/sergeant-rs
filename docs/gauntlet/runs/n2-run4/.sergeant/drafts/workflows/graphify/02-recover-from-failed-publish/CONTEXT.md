# 02-recover-from-failed-publish

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-run-graph-generation/output/outcome.md | L4 | upstream evidence produced by `run-graph-generation` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the graph-generation step exits (successfully or on failure) at any point after it may have started moving the old output aside

**Outcome:** a failure leaves the previous graph output intact and cleans up its own temporary artifacts rather than leaving a half-swapped or missing output

**Statement (the operative rule):** On any failure before publication completes, the exit trap restores the previously moved-aside old output, removes leftover temp symlinks and staging/backing directories, and only removes the old backup once the new output has actually been published.

## What must become true here (durable outcome)

A failure leaves the previous graph output intact and cleans up its own temporary artifacts rather than leaving a half-swapped or missing output — per the Statement above, which is the operative rule this stage exists to enforce.

