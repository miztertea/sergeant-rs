# ADR 0014: Product/workspace split — owner rulings on distro delivery, versioning, topology, and doctrine placement

**Status:** Accepted, 2026-08-17.

## Context

A live design session on 2026-08-17 (owner and Captain, in conversation)
opened on three linked complaints: the repository has outgrown its
organization; developing *sergeant* and developing *sergeant the product*
push and pull against each other with no boundary between them; and
`AGENTS.md` should be the captain persona rather than a document carrying
its own development corpus.

The session consumed four inputs. Two were named by the owner as research
to read before answering: **Open Knowledge Format v0.2**
(`GoogleCloudPlatform/knowledge-catalog`, trust signals, `sources` /
`generated` / `verified` / `status` / `stale_after`, reserved `index.md`
and `log.md`) and **Karpathy's LLM Wiki pattern** (immutable `raw/`,
LLM-owned `wiki/`, ingest/query/lint). The third was
`~/inbox/proposal-ci-cd-release-engineering.md` (2026-08-16), surfaced by
the owner mid-session via the inbox convention
(`docs/environments/cerberus.md`). The fourth was the owner's own Notion
research corpus, read on explicit owner instruction: the Six Memory
Failure Modes diagnostic, the Four-Layer Agent Instruction Model, Agent
Knowledge Operating System, Executable Project Memory (OKF),
Source-Grounded Concept Library, Five Context Failures, IdeaOS Capture and
Update Protocol and Multi-Session Reconstruction Protocol, Anti-Capture
Governance, Agent Behavior Framework, O-SMEAC, Work-Centered Intelligence,
Work Filesystem, WorkPacket, Bugle, and Skill Libraries as Simulated Work
Environments.

The rulings below were made by the owner in that session. They exist
nowhere else: the conversation is not a durable record, and every
downstream artifact — `reference/proposal-product-workspace-split.md`, the
NORTH-STAR amendments, and the SPLIT-1 grading contract — traces its
authority here. Recording them is itself an instance of the dark-corner
failure the session diagnosed: decisions reached in conversation had no
home until this file.

## Decisions

1. **Distro delivery — embedded.** The distro (always-on doctrine, skills,
   workflow templates) ships embedded in the `sgt` binary and is written to
   disk by `sgt init`. It is not a cloned directory. This is a deliberate
   departure from firstmate's distribution-unit claim — "there is no
   application to install, because the cloned repository *is* the distro" —
   which the owner's own research corpus names as the lineage's strongest
   available prior art.

2. **Co-versioning — one artifact identity.** `v0.x.y` names one thing: a
   binary together with the doctrine it knows how to use. Owner's rationale,
   recorded in his words: *"if we add/change a verb in the binary an old
   captain wouldn't be confused by it if they were decoupled."* Accepted
   consequence: every doctrine edit is a binary release. The mitigation is
   to make releasing boring, not rare.

3. **Templates, not published procedure.** The shipped workflow packages
   are examples, defaults, and starting points. Owner's framing: *"These
   aren't how you should work. These are ways you could work. The
   expectation is that you write your own locals for how you write code."*
   Local packages shadow stock by name. Stock is replaced wholesale on
   update.

4. **No `sgt workflow diff`.** Captain proposed a drift-diff verb, then
   withdrew it under R1 once templates were reframed as examples. The
   corpus's counter-argument — a fork has no invalidation mechanism, "no
   edition number, no survey date, no revision program" — is answered by an
   edition marker on each template, not by a verb.

5. **Two repositories.** `sergeant-rs` is the product. A new
   `sergeant-rs-workspace` (created by the owner, 2026-08-17, private) is a
   Sergeant estate that mounts `sergeant-rs` and owns the development
   knowledge library. The workspace opens PRs against `sergeant-rs` like any
   other registered repo, and runs its own process for accepting changes to
   itself.

   Captain initially proposed three repositories (product, estate, record)
   and an extraction of the distro *out of* the dev repo. The owner
   corrected both: two repositories, and the dev estate consumes the
   released distro rather than a build-tree approximation of it.

6. **Workspace starts clean.** No history migration. Owner: *"If it needs
   git history, it can get that from the sergeant-rs repo."*

7. **`DEVELOPMENT.md` exists in both repositories** as two different
   documents — how to change the Rust product, and how to operate the
   workspace. Separate repos, no collision.

8. **NORTH-STAR is amended in three places** (see `NORTH-STAR.md`, dated
   entry 2026-08-17). Owner's reasoning applies the document's own rule to
   itself: *"what's in the repo is what was known at the moment it was
   written. North Star is no different."* The destination is restated
   stranger-first; the gated prebuilt-binary item is unblocked; and
   "whether or not anyone is watching" is struck as describing a
   consequence rather than the thesis — *"That's not what sergeant is
   about. It's about executing intentions."*

9. **The inbox CI/CD proposal is re-scoped, not adopted wholesale.** Owner:
   *"the proposal did its job, it led us to the proper solution."* Slices
   1–3 are adopted; Slice 4 is deferred until the release names two
   artifact classes. Its §12/§21 deferral of "AgentOS installation or
   update semantics" is precisely the half this ADR's decisions 1–3 move
   into scope. The source file remains in `~/inbox/` — it is re-scoped, not
   accepted, so the inbox convention's deletion-on-acceptance does not yet
   fire.

10. **Both ladders belong in `AGENTS.md`.** The Bounded-Judgment Ladder
    (J5–J0, ADR 0013) and the Ponytail Minimality Ladder (R1–R7) ship
    inline in the always-on file. This was the session's opening ask:
    *"agents should also get the basic decision ladder, ponytail ladder,
    etc. it's what builds its basic function to be able to answer within
    its purview and not exhaustively escalate everything to the human."*
    Measured gap that prompted it: `AGENTS.md` referenced neither ladder,
    and `@@bounded-judgment` resolves only inside an active stage's
    `CONTEXT.md` — so a direct in-session Captain, the mode most prone to
    over-escalation, had no ladder at all.

11. **Knowledge organization is co-equal in priority with the `AGENTS.md`
    rewrite.** Owner: *"if you can't find the lessons, find the gotchas,
    find how to deal with issues on the Cerberus host, find why we chose
    something, reference and adr, go back and see a decision log, without
    that we're developing half blind like we have been and developing more
    only compounds the problem."*

12. **Anti-capture balance accepted.** The session's push toward
    `needs_input` not implying human input is a deliberate move that matches
    a named capture signal — "human-review paths quietly become
    autonomous." The owner accepted the tripwire: any rung-lowering
    (J0 → J2) is an explicit, reviewed specification change, never emergent
    drift.

13. **Model policy for this workstream.** Dispatched Works run on the
    `sonnet` profile. Captain is Opus and holds adjudication. Fable takes no
    seat. This narrows R-S0-13's three-seat spread
    (`reference/notes/gauntlet-pattern.md`) for this workstream only; the
    general ruling is untouched.

## Alternatives considered

- **Three repositories** (product / estate / record). Rejected: the record
  and the estate share one audience and one access pattern — LESSONS L12
  requires re-reading governing text *at decision time*, and a split would
  make every decision need two clones.
- **Distro as a separate archive in the same release.** Rejected in favour
  of embedding: one artifact to install, and no way to hold a binary
  without its doctrine.
- **Decoupled versioning with a compatibility matrix.** Rejected on the
  skew argument in decision 2, and because the consumer is a language model
  that improvises around a missing flag rather than failing cleanly.
- **Notion as the project's knowledge system of record.** Proposed by
  Captain, rejected by the owner ("that's my personal idea store"), and
  independently already ruled against by the owner's own Notion decision
  page *Notion Is the Conversational Knowledge Layer* ("does not become the
  source repository") and by `reference/notes/ideaos-agent-contract.md`
  ("This repo never writes to the owner's Notion").
- **Full OKF reformatting of the corpus.** Narrowed to conformance rules
  the validator enforces, per R1/R6.

## Consequences

- Every doctrine edit becomes a binary release. Release cadence rises
  sharply; the plan absorbs this by automating qualification rather than
  batching doctrine changes.
- 91 template files reference `reference/sergeant-upstream/` and 611
  `BU-####` citations point into the behavior-unit corpus — all of which
  leave for the workspace. Template decontamination becomes a prerequisite
  for the split, not a follow-up.
- `AGENTS.md` loses its "Standard workflow loop" section: eight numbered
  steps of Layer-1 CLI mechanics occupying a Layer-0 file. The binary
  becomes the sole authority for its own surface, enforced by a cross-repo
  skew check that only the workspace can run.
- The workspace repo introduces a bootstrap hazard: a stock-template change
  cannot be exercised by the estate that ships it until released. Pointing
  the estate at a working tree is a legitimate testing posture and a
  corrosive default.
- Departing from firstmate's cloned-directory unit costs the lineage's
  strongest prior-art claim. Recorded rather than glossed.
- This ADR is the fidelity authority for the SPLIT-1 grading panel. Without
  it the panel would grade a proposal against nothing.

## Amendment, 2026-08-17 (same day, after SPLIT-1)

SPLIT-1's fidelity axis found that `reference/proposal-product-workspace-
split.md` §4.4 added two rungs below J0 — PACE and succession-of-authority —
with no traceable source, and carried them into its decision register
unqualified. The finding was correct: Captain drew both from the owner's
Notion governance corpus during the same session and asserted them as
doctrine.

The owner ruled after the panel closed, adding decisions 14–18:

14. **PACE and succession-of-authority are adopted.** Owner: *"Put them in.
    That military doctrine is sound. I didn't push back because I agreed and
    just didn't say so."* Recorded with the correction that silence is not
    assent and should have been asked about. PACE is a ladder of **routes to
    an authority**, never of decision latitude — degrading the route never
    degrades the rung (see the proposal's §9, corrected).
15. **The OKF type vocabulary is proposed by the first compile pass, not
    pre-ratified.** Captain proceeds where it agrees with the compiler's
    proposal; a materially different taxonomy escalates. A bounded J2 grant
    with a named escalation trigger.
16. **`sergeant-rs-workspace` gets its own `AGENTS.md`** — purpose-built for
    developing sergeant-rs, structurally modelled on the product's, Layer-3
    in content.
17. **Extract before moving, for every document.** Proposals go to the
    workspace, but anything binding present behavior is extracted to an ADR
    in the product repo first. Owner: *"That goes for every document.
    Learnings, potential adrs, etc. Consider what we have as evidence and
    its target is a well formulated project knowledge base."* This
    generalises decision 9's treatment of `GAUNTLET.md`/`LESSONS.md` to the
    whole corpus.
18. **`docs/environments/` splits by kind.** The capability stays with the
    product — `scripts/probe-env.sh` and the rule that a host is measured
    before it is trusted, because *"sgt should know to understand its
    environment."* The dated measurements become a workspace host wing.
    `sergeant-rs-workspace` stays private, so no secrets sweep gates the
    migration.

Decision 13's model policy is unchanged. ADR 0015 records the separate
general ruling on pull requests as proposals.

## Open questions

- Does `docs/environments/` stay with the product or move to the workspace?
  Host facts are workspace-shaped, but `scripts/probe-env.sh` ships.
  Currently assumed: stays.
- Edition-marker format — frontmatter field, or generated manifest?
- Will `sergeant-rs-workspace` become public? Determines whether the
  evidence tree needs a secrets sweep before the move.
- Which derived views ship in the first compile pass? Five Context Failures
  gates the set; the ordering is unresolved.
