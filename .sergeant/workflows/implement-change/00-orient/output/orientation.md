# Orientation — sprint split-hardening W5 (#259, #262)

## Pinned revision

Work branch `sergeant/01M0T0R613MCJ7BSCPMQETK7VW`, cut from `main` at
`a126dbd2e961eacd93ec1867c6ac436424f608a4` ("Merge pull request #257 from
miztertea/fix/opencode-question-replied-race"). Verified resolvable via
`git rev-parse --verify a126dbd2e961eacd93ec1867c6ac436424f608a4` and
`git log -1` in this worktree; this worktree's `HEAD` is currently at the
same commit. All later stages (panel, close) are judged against this
fixed point, not re-derived.

Note: the worktree already carries unstaged modifications to
`src/backend/codex.rs`, `src/cli.rs`, `tests/codex_backend.rs`, and
`tests/m6_surfaces.rs` at session start — pre-existing state to build on,
not part of the pinned baseline.

## Spec / acceptance source

Located directly, no inference needed: GitHub issues `gh issue view 259`
and `gh issue view 262` in this repo, both filed by the Work-branch owner
and cross-referenced by the dispatching prompt.

- **#259** — "[codex] Linked-worktree Git metadata is read-only, so actors
  cannot commit." Acceptance criteria (verbatim from the issue): a real
  Codex contract test edits/stages/commits in an assigned linked
  worktree; the commit advances the assigned `sergeant/<work-id>` branch;
  the actor cannot write outside its authorized Work/linked-Git scope;
  `sgt doctor`/submit preflight fails closed when the configured
  permission mode cannot provide that capability.
- **#262** — "[codex] bypassPermissions actor cannot bind loopback while
  doctor reports the profile healthy." Acceptance criteria (verbatim): a
  measured Codex adapter contract test can bind `127.0.0.1:0` when the
  selected permission mode claims to allow native repository validation;
  loopback access does not imply or grant external network access;
  doctor/preflight distinguishes the configured permission-mode name from
  effective actor capabilities and gives a specific remedy when they
  differ.

The dispatching prompt supplies additional, more specific engineering
guidance (root causes with line references, a five-part build plan, an
explicit out-of-scope list, and gate commands) that narrows *how* to meet
these two issues' acceptance criteria. That prompt is treated as the
authoritative task-level spec for implementation choices; the GitHub
issues remain the source of truth for *what durable outcome* counts as
done.

## Boundary

**In scope for this change:**

- Codex-adapter-local resolution of the Work's own linked-worktree Git
  admin dir (`.git/worktrees/<name>`) and its addition as a scoped
  `--add-dir` writable root for that Work's codex launches only — never
  the whole `.git`, never `repository.path`.
- A fail-closed preflight check in the codex adapter's `prepare()`: if
  that grant can't be resolved/applied for a mutation-shaped launch,
  refuse admission with a named, actionable error instead of letting a
  Work run to `completed_dirty`.
- Making `sgt doctor`'s `permission_mode` row state, structurally, which
  backends actually consume `permission_mode` — and for codex (which
  ignores it), say so plainly instead of implying effect.
- A new per-profile/codex-config opt-in knob that composes codex-cli's
  documented `-c sandbox_workspace_write.network_access=true` override,
  scoped to sandbox loopback binding, never daemon-global, never
  default-on, never `danger-full-access`.
- Unit tests for the composed argv (git-dir grant, network override),
  a preflight-refusal test, a doctor-wording test, a `common_dir_finding`
  regression check, and a gated live end-to-end contract test proving a
  real Codex commit lands on the Work branch.

**Explicitly out of scope:**

- Any change to the claude/opencode/agy adapters beyond what a
  `BindingSummary` widening — if that route is taken over an
  adapter-local resolution — mechanically requires.
- Making `permission_mode` do anything functional for codex; the fix is
  honest reporting, not new codex behavior driven by that field.
- Any grant broader than the single `.git/worktrees/<name>` directory, or
  any network capability beyond scoped sandbox loopback (no external
  egress, no daemon-wide default).
- Anything not needed to close #259/#262 as specified — this change does
  not attempt a general commit-capability or capability-reporting
  overhaul beyond the codex adapter and doctor's `permission_mode` row.
