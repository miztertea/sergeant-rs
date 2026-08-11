# Upstream Sergeant (callmeradical/sergeant) Decision-History Mining

Surveyed all 100 listed issues (85 closed/open) and 100 PRs by title; deep-read 22 decision-bearing items via `gh issue view`/`gh pr view --json body,comments`. Wrote nothing to any repo.

## 1. The 12 sharpest lessons

**1. Drain killed the watcher, not the worker (#152) — process/pane identity divergence.** `_do_drain` ran `kill -TERM "$BASHPID"` inside a background subshell, killing only the watcher; the OpenCode process, notification loop, and progress loop survived. 11 workers marked `drained` and 3 marked `done` still had live processes, ~9 GiB RSS aggregate stranded, some with pane IDs that didn't even match the fleet record's pane ID anymore.
**Status: structurally solved.** sergeant-rs's daemon owns process handles directly (no tmux-pane introspection as a proxy for liveness) — this entire bug class requires the daemon to lose its own handle, a different and narrower failure mode.

**2. The cleanup-ownership saga — trusting fleet state instead of proving it (#39–#46, plus ~15 more issues/PRs through #129).** Roughly a dozen sequential PRs progressively hardened `sgt-cleanup` to validate exact persisted owner/repo/worktree identity before acting, after repeated incidents of cleanup replay trusting unverified `status`/`result` fields. #39: "Cleanup must prove exact persisted project, repository, worktree, remover, and evidence ownership before reconciling an absent worktree. Publication failures must fail closed."
**Status: structurally solved, and validates the design** — sergeant-rs's "ambiguity fails closed... never a guess" (CLAUDE.md) is exactly the end-state this saga arrived at reactively over months. Worth citing as evidence the principle is correct, not just aesthetically nice.

**3. Fail-closed states with no exit door (#123, #199, #111, #170).** After an acknowledgement timeout, a worker landed in `orphaned` — and recovery rejected it ("worker status is 'orphaned' (must be in_progress)") while cleanup also rejected it ("not terminal: orphaned"). The record was permanently stuck.
**Status: live risk.** sergeant-rs's fail-closed-into-`blocked` design must pair every fail-closed transition with a defined *unblocking* path, or it reproduces this exact class. Recommend fault-injection tests assert not just "stops safely" but "an operator/reconciler can get it out."

**4. Generic dedup keys collided across unrelated contexts (#146, #147, #162, #164, #166, #186, #197 — 7 issues/PRs).** `sgt-review-findings` matched findings by `repo+axis+source+finding-ID`; a short human ID like `SPEC-001` reused across unrelated reviews silently overwrote a different task's title, description, evidence, and parent lineage.
**Status: open design input.** Any future finding/task-router or journal correlation key must scope identity to full causal context, never a short reused label alone.

**5. SQL injection + a command that exits 0 while dropping a table (#205 review comment).** The new SQLite-backed inter-agent message bus interpolated every value directly. Ordinary prose (`"don't merge this"`) broke it; crafted input (`x', '2020-01-01', NULL); DROP TABLE messages; --`) executed arbitrary SQL, and the failure was silent at the exit-code level.
**Status: live risk worth a targeted audit.** Echoes sergeant-rs's own L1 ("exit codes lie"). Worth checking `src/runtime/analytics.rs` for any place agent-authored text reaches DuckDB via string interpolation rather than parameters.

**6. Bash 3.2 associative arrays crashed macOS outright, and the regression test couldn't fail (#191).** `declare -A` under `set -euo pipefail` aborted before any wake condition was read; separately, `tests/sgt-wake-test.sh`'s `_assert` called `exit 1` inside a subshell, which the parent's `set -e` never observed — false green for an unknown period.
**Status: structurally solved (Rust, no bash portability)**, but the meta-pattern — a test that structurally cannot fail — is precisely what LESSONS L7 and the DataDir/pgrep-bracket discipline in this repo's CLAUDE.md already guard against. Good corroboration, not a new risk.

**7. External `td` CLI integration was fragile on every axis it touched (#12, #115, #138, #173).** Name-collided with an unrelated Homebrew `Swatto/td`; broke on JSON shape drift in `td list --json`; the managed systemd service's PATH omitted `td` entirely so handoffs silently weren't recorded; git metadata was captured from the invoking process's CWD instead of the explicitly-supplied worktree.
**Status: open design input for U-series' td question.** Any external CLI dependency needs capability/version probing (not presence-only), PATH-independent resolution for daemonized contexts, and worktree-scoped invocation that never relies on CWD.

**8. Compaction silently substituted the wrong task and corrupted credentials (#52).** After conversation compaction, the coordinator dropped a live non-td-tracked task, picked an older task instead, and "corrected" `root@10.0.0.5` from the actual `user@10.0.0.5` — a quiet wrong answer, not a visible failure. Closed as "a workflow/process concern rather than a code defect... no code change needed," mitigated only by a documented convention (`td usage --new-session`).
**Status: live risk despite stronger architecture.** sergeant-rs's durable session identity (journal-sourced, not conversation-memory-sourced) structurally reduces this, but it's exactly the shape of bug ("confidently wrong" rather than "visibly failed") that deserves an explicit restart/reconnect fault-injection test rather than trust in the architecture alone.

**9. Deliberately honest "unknown" over false "idle" (#154).** The fleet-snapshot design returns `busy:true` only from verified live evidence, and **never** `busy:false` in v1 — every other case is `busy:null`, because a trustworthy negative requires a quiescence certificate that didn't exist yet.
**Status: good pattern to affirm**, not a risk — worth carrying into any sergeant-rs status/monitoring API: prefer "unknown" to a confident guess.

**10. A prototype shipped with `# DELETE when prototype question is answered.` as line 3 for the entire project's life, with no recorded decision anywhere (#179).** Forced closure eventually produced `docs/adr-oc-inject-deletion.md`.
**Status: validates existing practice** — this is exactly why sergeant-rs's GAUNTLET.md deviation register and LESSONS.md exist instead of leaving decisions as dangling comments.

**11. Drain admission was retrofitted three times (#68, #74, #167).** Two-tier global+project persistent drain locks, then cooperative worker checkpointing, then still needed `--wait`/`--timeout` support and honest lock-diagnostics (owner, age, purpose) added later because the original lock either silently blocked or silently failed open depending on which path was hit.
**Status: open design input for U-series drain semantics** — build bounded-wait and owner/age-visible lock diagnostics in from the start rather than retrofitting under incident pressure.

**12. An external DAG engine (dagr) was integrated then immediately made optional (#131 → #132).** Coupling the core dispatch loop to an external workflow-graph tool created enough friction that it had to be decoupled again within one release cycle.
**Status: corroborates** sergeant-rs's choice to own its stage/workflow engine internally (N-series proposal) rather than shell out to an external orchestrator.

## 2. Bearing on open U-series questions

- **td's role:** #207 (open, unresolved upstream) — no bounded interface resolves a td epic/task identifier to its fleet descendants; `sgt-context`, `sgt-status`, and `sgt-watch --snapshot` each reimplement partial namespace resolution independently. Combined with #12/#115/#138/#173's integration fragility, the upstream evidence argues for deciding early and explicitly whether td-like identifiers are first-class journal correlation keys owned centrally (matching sergeant-rs's own "if a client needs something the API lacks, extend the API" rule) — never duplicated ad hoc per surface.
- **context/sync as skills:** #52's fix was a *documented convention* (`td usage --new-session`), not a protocol guarantee — and it still didn't stop the corruption in the reported case. Structured handoff/sync needs to be a protocol-level, journal-durable artifact, not a habit callers are trusted to follow.
- **estate layout:** #207's duplicated resolution logic across three+ CLI surfaces is a direct argument for a single canonical estate/status resolver behind the API, not reproduced per client.
- **worker monitoring:** #152 (pane/process identity drift), #199/#123/#111 (orphan recovery deadlocks), and #154 (fail-closed tri-state snapshot design) are the most transferable evidence — together they say liveness must be proven from a source the daemon itself controls, and every terminal-looking state needs a recovery transition designed in, not bolted on after users get stuck.
- **drain semantics:** #68/#74/#167's three-pass retrofit (locks → checkpointing → wait/diagnostics) is a checklist for what U-series drain design should specify up front.

## 3. Verbatim, worth seeing directly

- #152: *"roughly 9 GiB RSS in aggregate... not OS orphans: their tmux supervisors and controlling terminals were still alive."*
- #123: *"ERROR: Recovery is unavailable: worker status is 'orphaned' (must be in_progress)"* / *"ERROR: <repo> is not terminal: orphaned"* — a record with zero supported transitions, from real command output.
- #205 review: *"Ordinary prose broke the bus. `\"don't merge this\"` produced a SQL syntax error... A crafted body executed arbitrary SQL... dropped the table while the command exited [0]."*
- #52 resolution comment: *"This is a workflow/process concern rather than a code defect... no code change needed."* — closing a wrong-host/wrong-credential corruption bug as a documentation gap is a notably different philosophy than sergeant-rs's stated "fail closed, never a guess"; worth the owner weighing whether that upstream precedent is one to explicitly reject.
- #179: *"`# DELETE when prototype question is answered.` ... has had [this] as its third line since it was written"* — 105 lines of load-bearing polling logic shipped for the project's entire life under a comment nobody ever resolved.