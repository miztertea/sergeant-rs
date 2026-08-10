# 10-extract-per-repo: extract per repo

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-resolve-output-path/output/README.md | L4 | upstream artifact produced by `00-resolve-output-path` |

## Purpose

Per-repo extraction completed, with in-source output staged out of the way and code-only fallback when no LLM key exists.

Trigger (workflow-level): Architecture work needs whole-project structure, or the operator asks for a graph/refresh.

## What must become true here (durable outcome)

Per-repo extraction completed, with in-source output staged out of the way and code-only fallback when no LLM key exists.

## Behavior contract

- **Extracting a repo's graph data stages the repo into a filtered, non-source-repo scratch location before scanning whenever any output-exclusion pattern applies, so the extraction tool can never re-ingest an already-published graph nested inside the source repo it is scanning.**
  (trigger: a repo's graph output lives nested inside the repo itself; outcome: a graph build never accidentally indexes its own previous output as source material)
  — `BU-P6-089`, `reference/sergeant-upstream/bin/sgt-graphify` (L349-355)
- **When no supported LLM API key is configured, graph extraction is run in code-only mode rather than being allowed to abort the entire multi-repo run on encountering a non-code file it cannot semantically process without a key.**
  (trigger: no supported LLM API key is available in the environment; outcome: graph extraction degrades gracefully to code-only indexing rather than aborting the whole project graph build)
  — `BU-P6-090`, `reference/sergeant-upstream/bin/sgt-graphify` (L356-359)
- **sgt-graphify must fall back to a `--code-only` extraction mode when no LLM API key is configured, rather than failing outright, so graph extraction remains available without an LLM dependency.**
  (trigger: sgt-graphify runs in an environment with no LLM API key configured; outcome: graph extraction degrades gracefully to a code-only mode instead of being entirely unavailable without an LLM credential)
  — `BU-P7-088`, `reference/sergeant-upstream/tests/sgt-graphify-test.sh` (line 660)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
