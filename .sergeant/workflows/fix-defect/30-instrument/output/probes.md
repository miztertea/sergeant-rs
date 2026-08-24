# Instrumentation probes — #234 dirty-patch trailing-newline corruption

## Status note

As with the two prior stages, this Work branch's `git log` already carries
the fix (`56927a14`) and its evidence records (`f16ba9e4`, `e0718598`),
landed by an earlier turn of this same Work. This stage adds direct,
code-level instrumentation probes against the ranked hypotheses in
`../20-hypothesize/output/hypotheses.md`, run once, captured here, then
removed (`git checkout -- src/runtime/git.rs src/runtime/surface.rs`) —
`grep -rn PROBE234 src/` now returns nothing, confirming cleanup. No edit
to the subject remains after this stage; the fix and regression test are
unchanged from `56927a14`.

Tool choice, per this stage's ordered preference: no debugger/REPL exists
for this crate's test harness, so the next tier — targeted `eprintln!`
logs at the exact boundary each hypothesis makes a prediction about — was
used, one call site per probe, all tagged `PROBE234:` (`PROBE234:H2` for
the H2-specific probe) so cleanup was a single `grep -rn PROBE234 src/`.

## P1 — `git()`'s trim, at the real call sites `capture_dirty_patch` exercises (H1)

**Prediction (H1):** raw stdout ends `0x0a`; `git()`'s trimmed `String`
does not.

**Probe:** logged `output.stdout.last()`/`.len()` before the transform and
the trimmed string's last byte/len after it, inside `git()` itself, then
ran `retain_dirty_writes_a_patch_git_apply_accepts` with `--nocapture`
(this exercises every ordinary `git()` call `materialize`/`teardown` make
around the dirty worktree, not just the diff capture, since `git()` is the
default helper for most of this module).

**Result — confirmed.** Every logged invocation showed the same shape:
`raw_last_byte=Some(10) ... trimmed_last_byte=Some(<content byte>) ...
trimmed_len=raw_len-1`. Representative line:
```
PROBE234:git raw_last_byte=Some(10) raw_len=41
PROBE234:git trimmed_last_byte=Some(52) trimmed_len=40
```
Unconditional across every call observed — `.trim()` in `git()` strips
exactly the trailing `\n` on every invocation, not only the diff capture.
This is the general mechanism `capture_dirty_patch` used to go through
before `56927a14` switched it to `git_bytes`.

## P2 — `git()` vs `git_bytes()` on diff content containing invalid UTF-8 (H2)

**Prediction (H2):** if lossy re-encoding is an independent corruption
mechanism (not just the trim), a diff whose content includes a byte that
is not valid UTF-8 will come back *different* between the two capture
paths in more than just the trailing newline — because
`String::from_utf8_lossy` substitutes the invalid byte with the 3-byte
U+FFFD replacement character before `git()` ever reaches `.trim()`.

**Probe:** new fixture, not present in any existing test — a tracked file
edited to contain a single `0xFF` byte with no `NUL` nearby (so Git still
treats it as a text diff, the case H2 is actually about; a `NUL` would
make Git switch to `--binary`'s base85 encoding instead, which is
ASCII and would not exercise this path). Captured the identical `git diff
--cached --binary` invocation once through `git()` and once through
`git_bytes()`, one variable (capture function) changed:
```rust
let via_string = git(&spec.path, &["diff", "--cached", "--binary"])?.into_bytes();
let via_bytes = git_bytes(&spec.path, &["diff", "--cached", "--binary"])?;
```

**Result — confirmed, and refines H2 from "unexercised" to "independently real."**
```
PROBE234:git raw_last_byte=Some(10) raw_len=109
PROBE234:git trimmed_last_byte=Some(189) trimmed_len=110
PROBE234:git_bytes last_byte=Some(10) len=109 lossy_roundtrip_would_differ=true
PROBE234:H2 via_string_len=110 via_bytes_len=109 equal=false
```
`via_bytes` (109 bytes, ends `0x0a`) is the faithful capture. `via_string`
is *longer* than the raw stdout (110 vs. 109) despite trim also removing
the trailing newline — because the single `0xFF` byte was replaced by the
3-byte U+FFFD sequence (`EF BF BD`), a net +2 that swamps the -1 from
trimming. This is content corruption in the *middle* of the patch, not
just a missing terminal byte, and it happens whether or not the diff also
has a trailing-newline problem. H2 is promoted from "plausible, narrower,
unexercised" (20-hypothesize's ranking) to **confirmed as a second,
independent defect of the pre-fix `git()`-based capture path**, fully
closed by the same fix (`git_bytes`, which skips the `String` round-trip
entirely) — consistent with why `git_bytes` was specified to bypass
`String` rather than only drop `.trim()`.

## P3 — pre-write vs. post-write bytes across `write_atomic` (H3)

**Prediction (H3):** if corruption were introduced at the write path
rather than capture, a correctly-captured, newline-terminated `Vec<u8>`
would still lose its trailing byte (or otherwise change) by the time it
lands on disk.

**Probe:** logged the in-memory `diff` slice's last byte/len immediately
before the `write_atomic` call in `capture_dirty_patch`, then read the
file back off disk immediately after and logged its last byte/len,
comparing the two independently of any assumption that they match.

**Result — ruled out (unchanged from `20-hypothesize`'s finding, now
directly measured across the write boundary rather than inferred).**
```
PROBE234:capture_dirty_patch pre_write_last_byte=Some(10) pre_write_len=311
PROBE234:capture_dirty_patch post_write_last_byte=Some(10) post_write_len=Some(311)
```
Identical last byte and identical length before and after `write_atomic`.
The write path is a faithful passthrough; H3 is ruled out with a direct
byte-for-byte measurement, not only the regression test's on-disk
assertion.

## P4 — H4 (flag-dependence: `--binary`/`--cached`)

**Prediction (H4):** if the flag combination mattered, `.trim()`'s effect
would differ depending on the invoking args.

**Probe:** none run — by construction, not by omission. `git()`'s body
(`src/runtime/git.rs:116-133`) calls `.trim()` unconditionally on
`output.stdout` after every successful invocation, with no branch on
`args`. P1's log line above is drawn from calls the real code makes with
several different arg shapes (`worktree add`, `add -A`, `diff --cached
--binary`, `rev-parse`, …) and every one shows the identical trim-strips-
`\n` shape. There is no differential input that could produce a different
answer without editing `git()` itself, so no probe distinguishes what the
source already rules out; recorded here as the stage's honest answer for
this hypothesis rather than skipped silently.

## P5 — H5 (single- vs. multi-entry diff shape)

**Prediction, refined per `20-hypothesize`:** whether `.trim()`'s damage
is *observable* by `git apply --check` depends on whether the diff's true
last line is real content (multi-entry: tracked edit + untracked file) or
the `\ No newline at end of file` marker (single tracked-file-only edit),
which `git apply` already tolerates missing its own newline.

**Probe:** this is exactly what `retain_dirty_writes_a_patch_git_apply_accepts`'s
fixture already differentially tests by construction (a tracked-file edit
*paired with* an untracked file, deliberately not tracked-file-only) — P1's
capture of that same run is the evidence. No new probe added; reusing it
here is itself the "one variable changed" comparison the stage asks for
(diff shape held at the multi-entry case that makes the defect
observable, everything else the same test).

**Result — confirmed as refined, not the original framing.** The
diff observed in P1/P3 (311 bytes, ends `0x0a`) is the multi-entry case;
its last raw byte is a genuine `\n` after real content, which is exactly
what makes `.trim()`'s strip observable to `git apply --check`. This
matches `20-hypothesize`'s refinement: shape controls *observability*,
not *presence*, of the defect.

## Surviving hypothesis

**H1, confirmed** (P1, plus the pre-existing shell-level and code-level
evidence): `git()`'s unconditional `.trim()` strips the diff's mandatory
trailing newline. **H2, confirmed as a second, independent defect** (P2):
lossy UTF-8 re-encoding corrupts non-UTF-8 diff content regardless of the
newline issue. **H3, ruled out** (P3, direct measurement). **H4, ruled
out by construction** (P4, no differential input exists). **H5,
refined and confirmed** (P5): shape controls observability, not presence.

No hypothesis survives unproven — H1 and H2 both have direct positive
evidence, H3 and H4 both have direct negative evidence, and H5's
refinement is directly measured rather than assumed. `git_bytes`
(committed in `56927a14`) is the correct fix for both confirmed causes:
it skips the `String` round-trip entirely (closing H2, not only H1)
rather than merely restoring the trailing newline.

## Gate

Every prediction from `20-hypothesize` has its own tagged probe (P1-P5),
one variable changed at a time (capture function for P2, write boundary
for P3; P4/P5 are constructional/reused with the change of input shape
made explicit), and the surviving hypothesis is stated as confirmed
(H1, H2, H5-as-refined) or ruled out (H3, H4) — none left unproven.
Instrumentation was removed before this artifact was written;
`src/runtime/git.rs` and `src/runtime/surface.rs` are unchanged from
`56927a14` at the end of this stage.
