# Hypotheses — #234 dirty-patch trailing-newline corruption

## Status note

As with `10-reproduce-and-minimize`, this Work branch's `git log` already
carries the fix (`56927a14`) and its evidence record (`f16ba9e4`), landed
by an earlier turn of this same Work, ahead of this stage's execution. No
edit to the subject was made here. This document records the ranked,
falsifiable hypothesis set this stage requires — reconstructed against
the reproduction evidence in
`../10-reproduce-and-minimize/output/reproduction.md` — and confirms
which one the already-committed fix and its regression test actually
discriminate in favor of, without re-touching the subject.

## Ranked hypotheses

### H1 (highest confidence) — `git()`'s `.trim()` strips the diff's mandatory trailing newline

If the general `git()` helper's
`String::from_utf8_lossy(&output.stdout).trim().to_string()` is the
cause, then capturing the same `git diff --cached --binary` stdout
**without** that trim (raw bytes, `git_bytes`) will produce a patch whose
last byte is `0x0a` and that `git apply --check` accepts against an
independent clean checkout; reverting the capture path back through
`git()`/`.trim()` will reintroduce the corrupt-patch failure on the same
input.

**Status: confirmed.** The shell-level harness in
`reproduction.md` captured the identical diff two ways (verbatim vs.
`.trim()`-transformed) and showed the verbatim bytes end in `0a` and
apply cleanly, while the trimmed bytes end in `7d` and fail with `error:
corrupt patch at line 15` — the same failure signature as #234. The
committed regression test (`retain_dirty_writes_a_patch_git_apply_accepts`)
re-discriminates this at the code level through the real capture path.

### H2 — `String::from_utf8_lossy`'s lossy re-encoding (not the trim) corrupts binary diff content

If lossy UTF-8 re-encoding is an independent cause (distinct from
trimming), then a diff containing non-UTF-8 bytes (e.g. a binary-file
hunk) would still show byte mismatches against `git apply` even when the
trailing newline is preserved by some other means; using `git_bytes`
(bypassing `String` entirely) should be the only capture path that
preserves such content exactly.

**Status: not exercised by the current repro/tests** — the reproduction
harness and the committed regression test both use text-only diffs
(a tracked-file edit plus an untracked file), so this hypothesis was not
independently falsified. It remains plausible as a *second*, narrower
defect in the same helper and is the reason `git_bytes` was specified to
skip the `String` round-trip entirely rather than only dropping `.trim()`
— but it is ranked below H1 because #234's own evidence (last-byte
non-newline, always at EOF) is fully explained by H1 alone, with no
reported symptom pointing at mid-patch byte corruption.

### H3 — corruption is introduced at the write path (`write_atomic`), not at capture

If the defect were in how the patch is written to disk rather than how
it is captured, then a correctly-captured, newline-terminated `Vec<u8>`
handed to `write_atomic` would still land on disk missing its trailing
byte (e.g. from a line-ending normalization or truncation in the writer).
Falsified by comparing the in-memory captured bytes' last byte and length
against the on-disk file's last byte and length after `write_atomic`
runs.

**Status: ruled out.** The shell-level harness reproduces the exact
symptom purely from the `git()` helper's in-memory transform, before any
write occurs, and the committed test asserts the *written* patch file's
raw last byte is `\n` once capture goes through `git_bytes` — confirming
the write path is a faithful passthrough and was never the cause.

### H4 — the `--binary` flag (or `--cached`) on the diff invocation is what triggers corruption, not the general capture helper

If the specific flag combination were the cause, then running the same
capture through `git()` on a `git diff --cached` (no `--binary`) would
not exhibit the missing-trailing-newline defect, since the flag — not the
helper's stdout post-processing — would be responsible. Falsified by
re-running the harness with `--binary` omitted: `.trim()` still strips
the terminal `\n` regardless of flags, because the transform operates
purely on the returned `String`, independent of how the diff content was
produced.

**Status: ruled out by construction** — `.trim()` in `git()` is
unconditional over all callers of that helper, not conditioned on the
invoking args, so no flag-specific behavior exists to test differentially;
noted here only because it was a plausible-sounding alternative worth
naming and discarding explicitly rather than silently assuming away.

### H5 (lowest confidence) — the bug requires a multi-file diff to manifest at all

If the defect depended on diff shape (specifically, more than one
changed file) rather than being a byte-level always-present
transformation, then a single tracked-file-only edit would still fail
`git apply --check` after trimming, since the bug would be structural
rather than incidental to which line ends up last.

**Status: falsified as originally framed, refined into a minimization
finding.** The reproduction/minimization work found the *opposite*: a
single-file diff's true last line is the `\ No newline at end of file`
marker, which `git apply` tolerates missing its own trailing newline —
so a single-file case is a **false negative**, not evidence the bug is
absent. `.trim()` still strips the real trailing `\n` in both cases; only
whether that strip is *observable* by `git apply --check` changes with
file count. This hypothesis is retained at lowest rank as a documented
near-miss: it is why the committed regression test deliberately pairs a
tracked-file edit with an untracked file rather than using either alone.

## Checkpoint: user visibility

This is a headless run; the ranked hypothesis list above is recorded for
async re-ranking rather than shown interactively before proceeding, per
this stage's named non-blocking exception (J2: "proceeding without the
user's re-ranking if the user is unavailable"). No J0 escalation applies:
every hypothesis above is stated as a falsifiable prediction, H1 has
already been discriminated in its favor by both the shell-level harness
and the code-level regression test, and none of H2–H5 are equally
supported alternatives requiring a human tie-break — each is either
ruled out by the existing evidence or (H2) narrower in scope than, and
not contradicted by, H1.

## Gate

3–5 ranked, falsifiable hypotheses recorded before any instrumentation,
per this stage's completion boundary. No further edit to the subject was
made in this stage; `56927a14` (fix) and `f16ba9e4` (evidence) remain the
correct artifacts, with H1 as the confirmed causal hypothesis carried
forward into `30-instrument`.
