# Dogfood gauntlet — 2026-08-11, first product-fitness measurement

Owner-authorized ($25 cap; spent **$5.29**). Three promoted workflows run
end-to-end on real tasks, real claude backend, post-BS2 binary, operated
as a first-time human user would be. Full operator + critic reports:
`agent-reports.json` (this dir); driver:
`resources/n-series/dogfood-gauntlet.js`; raw journals per run under the
session scratchpad (`dogfood/<name>/deliverables/`, journals copied here
where small).

| Run | Result | Cost | Product verdict (critic, condensed) |
|---|---|---|---|
| `diagnose-bug` on issue #45 | canceled at stage 1/6 (~4.5 min; operator session force-terminated early — confound recorded) | ~$? (zero `usage.updated` recorded — itself finding E1) | Actor's work-in-progress was on-task, but PATH archaeology consumed half its calls; zero deliverable. Also: the package's name promises "diagnose" while its stages 40–60 implement a fix — a checkpoint-honesty defect independent of execution. |
| `research` on Rule C's compression question | **completed 1/1** | $4.31 | **Artifact genuinely good and load-bearing**: measured zstd-3 at 6.5–7.7× on a real captured journal (correcting the ruling draft's ~10× estimate), 9 primary sources, supply-chain vetting. But the Work wrapper added audit trail and nothing else over bare `claude -p` — "inert ceremony for a single-turn shape." Finished 7.8% over its cap with no intervention window (L16 live again). |
| `grilling` on R-H0-2 | completed 2/2 **autonomously in 80 s** | $0.98 | Worst case: the pause-for-answer checkpoint the workflow is named for cannot occur on this host (ask grammar absent) — both stages completed with zero needs_input; the human's answer has nowhere to land and the worktree tears down. Negative value vs plain terminal Claude for this use case. Predicted by the a5 re-measurement; now measured at product level. |

## Critic's library-level verdicts

- §6-ladder re-adjudication of the 8 single-stage packages: **5 stand as
  workflows** (research, drain-fleet, wiki-digest, route-review-findings,
  reconcile-and-cleanup-fleet — each gates real judgment or an
  irreversible action), **1 demotes to helper** (monitor-fleet), **2 need
  the owner** (wake-and-resume, deliver-external-callback — their own
  package text concedes the judgment case is thin).
- Answer to the owner's assignment worry, condensed: *the idea is not
  broken and the engine core held (settle driver ran two workflows
  end-to-end unattended); the research artifact proves real value is
  reachable. But the product is not human-usable unsupervised today —
  one run produced nothing on an environment gap, one is structurally
  broken on this backend, and the success case barely beat bare
  `claude -p` on value-per-ceremony.*

## Engine/product needs, ranked by what the runs actually hit (E1–E7)

1. **E1 — cost visibility on interrupted turns**: a canceled turn records
   zero `usage.updated` despite real billable calls; budget guards have
   no signal (compounds L16).
2. **E2 — PATH/toolchain parity** for daemon-launched actor subprocesses
   (actor couldn't find cargo; self-misdiagnosed it as permissions).
3. **E3 — submission-time capability gating**: grilling submitted cleanly
   on a host where its core affordance is withdrawn *after the first
   turn*; preflight was honest at submit and the run was doomed anyway.
4. **E4 — daemon lifecycle verbs** (`sgt daemon stop`; and a read-only
   client command silently respawned a just-killed daemon).
5. **E5 — config/profile discoverability** (sergeant.toml shape
   undiscoverable; `--profile` not implied; doctor silent outside the
   workspace).
6. **E6 — Layer-4 finalize/promote unimplemented**: every `promote`
   disposition is a no-op; deliverables die with the worktree unless
   hand-rescued.
7. **E7 — transcript legibility**: no `sgt work transcript`; reading what
   an actor said requires decoding blob hashes from journal payloads.

## Confounds, stated

diagnose-bug's operator session was force-terminated at ~4.5 min (harness
artifact, not product); its PATH findings stand, its 0/6 stage count is
not a fair grade of the package. The daemon env for that run was started
without the cargo PATH prefix this host needs (cerberus.md fact) — E2 is
real (the product should surface the env contract) but the operator
setup shares blame.
