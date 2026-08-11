# N-series session retrospective — 2026-08-11

Owner-requested before PR #43 merges: what in the instructions, the record, or
the way things played out would have helped this session go better. Written by
the orchestrator from lived experience, not reconstructed. Companion to the
S-series retro (PR #48); overlapping findings are marked rather than repeated.

## What would have helped most, ranked

### 1. An environment-facts probe, run once at session start

Nearly every hard stall this session was a *collision with an unmeasured
container fact*: the disk allowance (hit ENOSPC twice mid-verification), the
release-installer 403 through the proxy, `IS_SANDBOX=1` being required for
real claude turns (documented only inside `claude.rs` module docs and the
gate.sh note — the Run B operator found it; the orchestrator did not), the
root refusal of `--dangerously-skip-permissions` (undocumented anywhere,
discovered by a dead turn), and GitHub runners lacking `CAP_LINUX_IMMUTABLE`
(bit two sessions independently — the convergent-evolution probe-gate).
Every one of these was measurable in seconds *before* it cost an hour.
**Recommendation:** a `scripts/probe-env.sh` that measures and prints the
facts table (writable allowance remaining, proxy posture, uid + skip-flag
viability, CAP_LINUX_IMMUTABLE, O_DIRECT behavior of TMPDIR), run at session
start and pasted into the session's first ledger touch. CLAUDE.md's ops
section describes *procedures*; this is the missing *measurement* half.
[Partially converges with PR #48's ops section — this adds the probe.]

### 2. Disk budget arithmetic in CLAUDE.md, not just the DuckDB warning

The existing note says a debug target is ~5 GB. What it doesn't say: the
session allowance (~40 GB writable here), that a main checkout + one merge
worktree + the pipeline's private cache = three target dirs that do NOT fit
alongside a 2 GB scratchpad of run evidence, and the safe deletion order
(other checkouts' `target/` entirely > both `incremental/` dirs > pipeline
cache > never run evidence). Both ENOSPC events were survivable only because
deletes still work at quota. One paragraph would have prevented both.

### 3. A rule for factual claims transmitted to other agents (new lesson, L15)

The session's sharpest process failure: the orchestrator told the Run B
operator that the workflow's CONTEXT.md instructed `sgt respond` — inferred
from memory of *authoring the run-3 actor prompts*, not from reading the
file. The operator read the file, found the claim false, reported the real
discrepancy instead. L12 (re-read governing text at decision time) covers
decisions; this is its transmission corollary: **a factual claim inside an
instruction to another agent must either carry its evidence (file read
in-session, journal line quoted) or be explicitly labeled hypothesis** —
the receiving agent cannot distinguish your verified facts from your
confident guesses, and a wrong "fact" from the coordinator arrives wearing
authority it didn't earn. Landed as LESSONS L15. The same wake produced the
mirror-image win: the operator *declining* the orchestrator's
credential-copy instruction in favor of the documented safer fix — evidence
that instructing agents to verify their coordinator is not theoretical.

### 4. Cross-session coordination through the repo, not through merges

Two sessions worked blind on sibling branches. Cost: the same probe-gate
fix derived twice (validating, but ~an hour of duplicated CI round-trips),
a reconciliation that silently dropped the S-branch's post-merge lane fixes
(restored by PR #48), and a PR-#28 merge race window. The ledger is
append-only history, not presence. **Recommendation:** a lightweight
`SESSIONS.md` (or a sergeant Work item, once self-hosting) declaring per
active session: branch, current lane, last push SHA. Check it before fixing
anything that might be another session's lane; re-check the source branch
tip before declaring any reconciliation done (the drop happened because a
merge is a snapshot and the sibling kept moving).

### 5. Notification economics for long-running background phases

The orchestrator's context is the scarcest resource in a 12-hour session.
Idle re-arm notifications from watch agents ("still waiting, re-armed")
each cost a wake plus context to dismiss, and the one time a wake carried
real signal ("31 minutes, plausible for a genuine turn") the plausibility
judgment was wrong — a 10-second `pgrep` disproved it. Two rules that would
have helped from the start: watch agents surface only *transitions* (state
change, artifact appearance, deadline), never reassurance; and every
"still running, seems fine" claim about a native process must be
accompanied by process-table evidence, because a dead turn and a long turn
are indistinguishable from the work projection alone (that distinction is
literally this architecture's thesis — work state ≠ process state — and
the session still got caught assuming the friendly direction).

### 6. What the record got right — keep, and lean harder

- **Push-after-green + facts-into-artifacts-immediately** made three
  container resets and one context compaction nearly free. The repo is the
  orchestrator's real memory; everything that survived, survived because it
  was committed. This is now instinct; it should stay written.
- **The DataDir guard, the bracketed pgrep, resources/ over the zip, the
  deviation register** all did their jobs silently this session.
- **Evidence-first debugging**: every diagnosis that started with reading
  the journal (root refusal, envelope-less seam) was right; the one that
  started from memory (CONTEXT.md) was wrong. The journal being the only
  truth applies to the humans and orchestrators too.
- **Owner checkpoints at irreversible boundaries** (merge order, Run B
  scope, ending the timebox early) each changed the plan for the better.
  The cost of asking was minutes; two of the three answers redirected work.

## Disposition

L15 appended to LESSONS.md. Items 1–2 are Cerberus-appropriate (the probe
script belongs in the first Cerberus session where the facts differ);
item 4's SESSIONS.md is deferred until a second concurrent session next
exists. No CLAUDE.md edits here beyond what PR #48 already landed — the
ops section is the right home and this note is its evidence trail.
