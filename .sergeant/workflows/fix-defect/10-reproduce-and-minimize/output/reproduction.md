# Reproduction — #234 dirty-patch trailing-newline corruption

## Status note

This Work branch's `git log` already carries commit `56927a14` ("Fix
#234: dirty-patch capture drops the trailing newline, corrupting every
retained patch"), landed by an earlier turn of this same Work, ahead of
this stage's execution. No edit to the subject (`src/runtime/git.rs`,
`src/runtime/surface.rs`) was made in this stage — this document records
independent reproduction of the pre-fix defect and confirms the already-
committed fix and its regression test are the correct, minimal artifact,
without re-touching the subject.

## Red: independent shell-level reproduction

A throwaway harness (no crate compile required — fast, deterministic,
agent-runnable) reproduced the user's exact symptom outside the codebase,
by replicating only the transform `src/runtime/git.rs`'s `git()` helper
applies to `git diff` stdout (`String::from_utf8_lossy(&stdout).trim()`):

1. Init a source repo, commit one tracked file.
2. Materialize a second copy ("worktree"), dirty the tracked file so its
   last diff line is content (not the `\ No newline at end of file`
   marker), stage with `git add -A`.
3. Capture `git diff --cached --binary` twice: once verbatim (`fixed`),
   once through the buggy `.trim()` transform (`buggy`).
4. Apply each against an independent clean checkout at the same base
   commit — never the dirtied worktree itself — via `git apply --check`.

Result: raw diff's last byte is `0a` (`\n`); the buggy/trimmed patch's
last byte is `7d` (`}`); `git apply --check` against the trimmed patch
fails with `error: corrupt patch at line 15` — the same failure mode
(corrupt patch, always at end-of-file, last byte non-newline) reported in
#234 and in the 22/22-rejection follow-up evidence. The verbatim-bytes
patch applies cleanly. This confirms the defect is genuinely the
`.trim()` in `git()`, not a look-alike.

## Minimization: what is load-bearing

Cutting elements one at a time and re-running the harness:

- **The untracked file (`new.rs`) is load-bearing.** Dropping it and
  keeping only the single tracked-file edit made the trimmed patch apply
  *cleanly* — a false negative. With only one changed file, the diff's
  last line is the `\ No newline at end of file` marker, and `git apply`
  tolerates that marker line lacking its own trailing newline. Adding a
  second (untracked) file makes the diff's true last line real `+`
  content, whose stripped newline is exactly what corrupts the patch.
  This is why the committed regression test
  (`retain_dirty_writes_a_patch_git_apply_accepts` in
  `src/runtime/surface.rs`) pairs a tracked-file edit with an untracked
  file — removing either changes what the last diff line is and can hide
  the bug.
- **Verifying against a second, independent clean checkout is
  load-bearing**, not incidental: `git apply --check` against the same
  worktree the diff came from cannot observe the defect (the worktree
  already has the changes on disk); only a checkout that has never seen
  the dirty state proves the patch is portable, which is the only way a
  retained patch is ever consumed in practice (`sgt work reap` hands it
  to an operator against a fresh checkout, not back to a worktree that
  teardown already removed).
- **The specific dirtied content (closing bracket, no trailing
  whitespace) is not load-bearing for reproduction** — any tracked-file
  edit reproduces the missing-trailing-newline defect, since the bug is
  in stripping the diff's own terminating `\n`, independent of the
  file's content. It was retained in the existing test only because it
  matches #234's own pinned evidence shape, not because a different edit
  would fail to reproduce.

## Existing code-level regression coverage (already committed)

`src/runtime/surface.rs::tests::retain_dirty_writes_a_patch_git_apply_accepts`
is the minimized repro at the code level: one tracked-file edit + one
untracked file, teardown to `RetainedDirty`, asserts the captured
patch's last byte is `\n`, and asserts `git apply --check`/`git apply`
succeed against an independent clean checkout, with applied content
compared byte-for-byte. Verified green at this Work's current `HEAD`:

```
running 1 test
test runtime::surface::tests::retain_dirty_writes_a_patch_git_apply_accepts ... ok
```

`tests/m7_docker_executor.rs`'s `walk_for_marker` e2e coverage
additionally runs `git apply --check`/`--stat` against every
`*.dirty.patch` found along the submit → execute → teardown path,
per the same commit.

## Gate

Reproduction confirmed (red, matching the user's exact symptom) and
minimized (every remaining element load-bearing, per above). This
satisfies stage `10-reproduce-and-minimize`'s completion boundary. No
further edit to the subject is required or was made in this stage; the
already-committed fix (`56927a14`) is the correct artifact.
