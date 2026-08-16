# Proposed fold into `AGENTS.md` "When NOT to use `sgt`"

Draft only — not a live edit (`docs/adr/0013` decision 6). Target section:
`AGENTS.md` lines 164-177 ("When NOT to use `sgt`"), current live text:

```markdown
## When NOT to use `sgt`

Dispatch (`sgt run`) is for work that spans repositories, contains two or
more independent repository-owned tasks, needs an isolated
independent-review worker, or the user explicitly asks for workers.
<!-- BU-0005 --> Direct, in-session implementation is used instead only
when both hold: the user explicitly asks to work in-session (or says not
to dispatch), and one repository owns the complete outcome — a
single-turn ask, answering a question, reading a file, a small edit with
no need to survive a restart or run unattended. <!-- BU-0004, BU-0009 -->
The harness (this session) owns that routing judgment; sergeant's core
makes no claim about it (North Star ruling 4). Reach for `sgt run` when
the work should be durable, resumable, and reviewable independent of this
conversation continuing — not by default for everything.
```

Proposed addition (two sentences, appended to the same paragraph),
folding BU-P1-107 (currently only stated for sergeant-rs's own code) and
BU-P1-009/BU-P8-056 (worktree/worker reconciliation, currently unstated
anywhere in the live corpus):

```markdown
Direct, in-session implementation is never a lighter path: it still goes
through task tracking, TDD-first implementation, native validation,
independent review, and the shipping gate — the same gates dispatched
work goes through, just without leaving the session. <!-- BU-P1-107 -->
Before committing to direct mode over dispatch, reconcile running work
the same way step 2 above does for a dispatch decision (`sgt work list`,
`sgt repo list`) — direct mode does not exempt the session from checking
whether another Work item or preserved worktree is already touching the
same repository or task. <!-- BU-P1-009, BU-P8-056 -->
```

Rationale: both additions are PL-1 (stable invariant) / PL-2 (Captain
judgment) restatements of behavior the `direct-implementation` package
otherwise packaged as dispatched-workflow stages — see
`../adjudication-draft.md` rows BU-P1-107, BU-P1-009, BU-P8-056. No new
heading or subsection is proposed; both sentences extend the existing
paragraph and existing Standard-workflow-loop step 2 respectively, per
`docs/icm/convention.md` §5 (helpers/invariants are additions to an
existing surface, not new packages, when the existing surface already
owns the class of decision).
