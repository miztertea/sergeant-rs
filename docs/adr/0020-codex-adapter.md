# ADR 0020: The Codex adapter — harness/backend axes, version provenance,
and the app-server transport

**Status:** Accepted, 2026-08-21. Implemented across four waves
(W1 `codex/exec`, W2 registration + R1 strike, W3 `codex/app-server`, W4
this record), merged to `integration/codex` (#225, #226, #227), closing
deviation D6.

## Context

Owner commission, same-day (2026-08-21): *"Claude works, now make codex
work."* Two prior decisions frame every choice below: ADR 0006 (`sgt
<harness>` compose-and-exec, already naming `codex` as a sibling of
`claude`/`opencode`/`goose` since 2026-08-14) and D6 (2026-08-08: Codex
descoped to a doc-stub because it could not be measured yet —
`knowledge/rulings/deviations/d6-codex-descoped.md`, a workspace-repo
record this ADR does not edit; see this ADR's own "D6" heading under
Consequences for what closes it).

Four owner rulings bind this whole sprint and are cited by name throughout
this record rather than restated each time:

- **R1** — a measured version floor is provenance, not a gate.
- **R2** — harness and backend are separate, user-composable axes.
- **R3** — core stays; every wave's work is the adapter, not a contract-v2
  seam.
- **R4** — parity is the floor, not the ceiling: where codex measurably
  works better than claude, use it, and amend the record with a dated
  entry (`[[repo-is-a-snapshot]]`).

## Decision

### Harness and backend are composable, user-selectable axes (R2);
default is the harness you launched with

`sgt claude`/`codex`/`opencode`/`goose` (ADR 0006) each set
`SGT_ORIGIN_CLIENT` on exec (`src/harness.rs`); `sgt run --backend <name>`
picks explicitly. Both already existed before this sprint touched
anything — the sprint's panel (amendment A1, 2026-08-21, converged ×3)
found the whole selection/precedence machinery already shipped:
`router.rs`'s four-tier ladder, `RouteSource::{Explicit, OriginAffinity,
WorkspaceDefault, GlobalDefault}`, tested end to end with `"codex"` as a
fixture backend name (`tests/codex_routing.rs`) months before the codex
adapter itself existed. **This sprint's registration work was therefore
verification, not invention**: W2 registered the W1 adapter under
`"codex"` in `BackendRegistry` (`daemon.rs`) and added tests proving
`--backend codex` and origin-affinity from `sgt codex` resolve through the
*existing* chain — no new precedence code, no new env var. An earlier
draft of this sprint proposed a second env var, `SGT_HARNESS`; the panel
struck it as undefined duplication of `SGT_ORIGIN_CLIENT`, which already
carries exactly that signal.

**Default resolution, as shipped and test-ratified (not newly designed
here):** `Explicit > OriginAffinity > WorkspaceDefault > GlobalDefault`.
Session-launched-harness affinity outranks the estate's own
`default_backend` — which is exactly R2's "default to the harness you
launched with," reached by a ladder this sprint verified rather than
built.

### Version floors are provenance, not gates — and this supersedes the
refusal language repo-wide (R1)

Before this sprint, `claude.rs` refused to launch below its own measured
floor (`MIN_TRUSTED_VERSION`), with a structured error. R1 struck exactly
that branch — **and only that branch** (panel amendment A2): the refusal
site bundled three conditions (`claude.rs:1030-1086`), and the other two —
a required launch flag absent from `--help`, an unparseable version
string — are not version-policy questions at all (one is launching
ungrammar sergeant never measured; the other is an unmeasurable CLI) and
were **not** struck. `claude.rs:1044-1068` now reports a below-floor CLI as
`available: true` with an honest unmeasured-provenance detail, and the
m4-era contract test that used to assert the refusal now asserts the
report. CHANGELOG.md's 0.2.0 entry names this a usability fix, not a
behavior regression: an operator on an old CLI now gets a working (if
unmeasured) backend and a truthful `sgt doctor` line instead of an outage
they cannot act on.

The codex adapter is built against this posture from the start: `codex.rs`
never had a refusal branch to strike. `MEASURED_FLOOR` (codex-cli 0.149.0)
is recorded, checked, and reported as provenance only — a build below it
is `available`, with the gap named, never blocked. **R1 is now this
repo's version-policy stance for every adapter, current and future, not a
one-time exception carved out for codex.**

### The transport story: `codex exec` and an adapter-owned `app-server`
child, both per execution; the daemon-and-WebSocket path is refused, with
its own roadmap lead recorded (A4)

**`codex exec --json`** (W1): one process per turn, line-delimited JSON on
stdout, no framing, no daemon. Simple, `Stable` (a first-class documented
subcommand), and the fallback under every capability this ADR names.

**`codex app-server --listen stdio://`** (W3): an adapter-owned child, one
per `CodexExecution`, for the whole execution's lifetime — not a shared
daemon. `runtime_scope()` stays `RuntimeScope::PerExecution`; W3's spec
argued (and W3's landed code confirms) that every capability upgrade below
is reachable on a per-execution child, so the shared, host-global
`codex app-server daemon start` path — one fixed control socket,
JSON-RPC-2.0-inside-WebSocket-frames even over Unix sockets, a process that
outlives its invoker — buys sergeant nothing this adapter needs and costs
three things it does not want: a new dependency (no WebSocket crate exists
in `Cargo.toml` today — `[measured-negative]`), a blast radius shared
across every execution and every human IDE session on the host, and an
auth-posture question (the daemon's listener is *designed* to be exposed
off-host) that is J0's to answer, not a wave's.

**A4 (owner ruling, mid-W3, 2026-08-21): the WebSocket path stays
R1-deferred, and the forecast that motivated it is recorded here as a
roadmap lead, not a design input it was ever treated as.** The argument
*for* eventually speaking WS was a convergence hunch — opencode and goose
*might* also speak WebSocket, so building the framing once might amortize
across three adapters — and the owner's own words at the time were "a
hunch at best, no reason to decision on it." The facts at ruling time:
no WebSocket crate anywhere in `Cargo.lock`; Claude 2.1.238's own
`--remote-control` surface was unprobed; opencode's and goose's transports
were unmeasured entirely. Recorded so that **the first adapter whose own
measured admission actually requires WebSocket framing** — not a forecast
about a sibling adapter — is what reopens this ladder, at R5, with its own
facts.

Both transports decode into the same `NativeEvent` shapes through one
shared, transport-agnostic `ItemView`/`TurnAccumulator` extraction
(`codex.rs`/`codex_appserver.rs`), so the narration rule below is enforced
structurally by there being exactly one code path that can ever produce a
`tool.*` event, not by two decoders that happen to agree today.

### The admission-rows / L8-structural pattern (R3)

Contract v1 (`src/backend/mod.rs`) ships thirteen booleans plus
`AskAuthor` and `RuntimeScope` — no typed capability enum, no
`StructuredOutputCapability`, no contract-v2 seam. **This adapter adds none
either**, on R3's authority. Where the research proposal's typed vocabulary
(`InterruptCapability::NativeTurnInterrupt`, `StabilityTier::Experimental`,
evidence tiers) is genuinely useful — and it is, for recording *how* a
`true`/`false` was earned — it lives as an **adapter-local**
`AdmissionRow` struct and a `const ADMISSION_ROWS: &[AdmissionRow]` table
inside `codex.rs` (lines 469–495 onward), rendered into `ProbeReport::detail`
and the wave PR body. `admission_rows_agree_with_capabilities`
(`codex.rs`) is a **compile-adjacent unit test**: a row claiming `true`
with no named admission test fails the build. L8 ("every advertised
capability flag needs a contract test against the installed harness") is
therefore not a review discipline for this adapter, it is a type-level
one.

**This is the pattern any future adapter (opencode, goose, or a codex
capability this wave declined) should copy rather than re-derive.** When a
real contract v2 lands, these adapter-local rows are the thing it lifts —
nothing here is thrown away, and no core type was invented on one wave's
authority to get early value out of the vocabulary.

**How much of R-H0-6 this actually answers, stated honestly.** The H0
packet asked the admission ladder for two additional rungs beyond
`documented → implemented → measured → admitted`: a rung for a **measured
negative**, distinct from unknown, and a way to record an **uncorroborated
harness assertion**. `AdmissionRow`'s `Evidence` enum
(`LiveMeasured`/`LocallyMeasured`/`SchemaClaimed`/`Unmeasured`) does not
add a fifth variant for either — a measured negative is recorded as
`Unmeasured` plus a `note` naming it explicitly (e.g. `ask`'s exec row:
`"no ask channel on this transport at all [measured-negative]"`); an
uncorroborated harness assertion (a `configWarning` this build cannot
verify, say) has no dedicated field at all and would today be prose in a
row's `note` if one existed. **This is real, working coverage of the
practical cases R-H0-6 named, reached through prose convention rather than
a typed rung.** A future wave that wants the distinction typed, not just
conventionally-worded, extends `Evidence` with the two rungs the packet
asked for; this ADR records that it has not been done, so a later reader
does not assume the ladder is fully typed when it partly still lives in
`note` strings.

### `ask` — the recorded negative, not a gap left open

`Capabilities::ask` is `false` on **both** transports
(`codex.rs:769-798`). On exec this is structural and unconditional (no
ask channel exists on the transport at all — `[measured-negative]`). On
app-server, W3 specced a five-assertion live admission test
(`live_appserver_actor_authored_question_is_typed`) precisely because the
protocol's `item/tool/requestUserInput` method is a genuinely clean,
schema-distinguishable "the actor asked" record — the one place Codex's
own protocol could have exceeded Claude's `ask` capability. **The measured
outcome is a negative**: `gpt-5.6-luna` — the pinned, cheap dev tier, the
same tier the narration hazard below was measured on — did not reliably
invoke `request_user_input` under either of two tried prompt
formulations, and the row records exactly that, not a placeholder. This is
stated as a wave outcome, not a shortfall: §2.4 of the W3 spec named this
in advance as "a likely and perfectly good outcome," and it is exactly
what a promotion discipline that refuses to move a row on schema evidence
alone is supposed to produce sometimes.

Regardless of `ask`'s outcome, the adapter answers or refuses **every**
server-to-client request the app-server protocol can send
(`codex_appserver.rs`'s answering table) — an unattended stage that could
be parked forever by an unanswered blocking request is the exact failure
class this whole sprint exists to prevent, independent of whether `ask`
itself is ever admitted `true`.

### Sandbox: enforcement is the default, and it is claimed, not proven,
on this host

`SandboxChoice::WorkspaceWrite` is `CodexConfig`'s default
(`codex.rs:343`) — scoped to the Work's own declared surfaces
(`runtimeWorkspaceRoots`/`--add-dir`, composed from the same
`bindings_outside_cwd` function W1 already had). `danger-full-access` is
not offered by this adapter at all; an operator who wants it selects
`SandboxChoice::Inherit` and configures it in their own `~/.codex/
config.toml` — R1's spirit applied to the sandbox axis: sergeant does not
grow a flag whose only function is disabling the enforcement this wave
adds.

**Enforcement is claimed, not proven, and the row says so in exactly those
words.** `thread/start` accepts and *echoes back* the requested policy,
naming the Work's own writable roots (measured, token-free, on this host)
— but whether the OS-level sandbox actually denies an out-of-surface write
could not be verified on the development host at all: Cerberus cannot
initialize the nested network namespace bubblewrap needs
(`unshare: write failed /proc/self/uid_map: Operation not permitted`), so
every sandboxed command dies at bubblewrap setup regardless of what it
would have done. The `sandbox_enforcement` row's own note states this
verbatim: *"enforcement-claimed, not locally proven."* This is not a gap
this ADR asks a future wave to close by trying harder on the same host —
it is a fact about Cerberus, recorded so a reader on a different host
knows what would actually need re-measuring there.

**This is the NORTH-STAR tension H0 §E5 flagged in advance, and the
amendment below is where it is resolved rather than left implicit.**

## NORTH-STAR amendment 4 (dated 2026-08-21, appended by this ADR)

The following is appended verbatim to `NORTH-STAR.md`'s existing amendment
4 (the 2026-08-20 "isolated worktrees stated honestly" amendment, ending
"…prevention or OS-level sandboxing remains a non-goal."). **Re-verified
against the landed code while authoring this ADR** (`codex.rs`'s
`SandboxChoice` default, `danger-full-access` refusal, and the
`sandbox_enforcement` row's note, all read directly, 2026-08-22): the text
below matches what actually shipped exactly, with no adjustment needed.

> **Amended again 2026-08-21** (owner ruling R4 of the *Sergeant speaks
> Codex* sprint; `[[repo-is-a-snapshot]]` — this paragraph records what was
> known at this commit, and a later measurement may amend it again).
>
> *"Non-goal" here scopes core, not adapters.* Sergeant's core still runs no
> OS sandbox and blocks no write: the mutation surface is declared and
> observed, and integrity findings plus estate-drift observations remain
> the only things sergeant will assert about what a Work actually changed.
> What this amendment adds is that **an adapter MAY use its harness's
> native enforcement** where the harness has it, and that doing so does not
> make core an enforcement layer. The Codex adapter does exactly this: it
> scopes `codex`'s own sandbox (`workspace-write`, with the Work's declared
> binding surfaces as the writable roots) to the surface core already
> declares, and `backend/docker.rs` has done the same thing with
> bind-mounts and `--network=none` since before this was written down.
>
> Three consequences, stated rather than implied:
>
> 1. **Observation stays the source of truth.** An adapter's enforcement is
>    a belt over core's braces. Sergeant charges dirty evidence from what it
>    observed, never from what an adapter claims to have prevented — because
>    an enforced surface and an observed surface produce different
>    retirement stories, and only the observed one is designed.
> 2. **Enforcement is a capability, so it is admitted by measurement like
>    every other.** As of this date the Codex adapter's row reads
>    *enforcement-claimed, not locally proven*: the harness accepts and
>    echoes back the requested policy, and whether the OS sandbox denies an
>    out-of-surface write could not be verified on the development host,
>    whose nested-container environment cannot initialize bubblewrap. An
>    unverifiable claim is recorded as unverified, not promoted.
> 3. **A shared mount two Works touch at once remains accepted risk** under
>    this contract, exactly as the 2026-08-20 amendment says, on every
>    backend — an adapter's sandbox scopes one Work's writes, not another
>    Work's.

## D6 — status

**D6 (2026-08-08: "Claude is the only native P0 adapter; Codex deferred
until an environment exists where it can actually be measured") is closed
by this sprint's own work**, and by W1's own header ("closing deviation
D6"). The register itself —
`knowledge/rulings/deviations/d6-codex-descoped.md` — lives in the
`sergeant-rs-workspace` repository, not this one, and **this ADR does not
edit it**: marking D6 resolved in its own register is the captain's
close-out action, recorded here as a hand-off rather than performed by
this wave (this repo has no write access to that file's home as part of
this ADR's own scope, and mixing a doctrine record's closure into a
same-repo ADR would blur which repo owns which record).

**What this wave verified in-repo, so the closure is not asserted
blind:** a repo-wide `grep -rn "doc-stub" src/` finds nothing — the one
place W1 replaced the doc-stub header, it replaced it completely. The one
piece of stale D6 language that *did* survive in-tree
(`docs/DEVELOPMENT.md:43`, "codex is a doc-stub per D6") is fixed by this
same wave (below). Every other D6 citation in the repo
(`daemon.rs:752`, `tests/m4_backends.rs:3`, `cli.rs`, ADR 0003, `GAUNTLET.md`'s
now-pointer-only body) is either about a *different* deviation reusing the
"D6" identifier from ADR 0003's own numbering (durability/storage
preconditions — an unrelated axis) or is accurate, past-tense-correct
prose about what W1/W2 did, not a present-tense claim that codex still
doesn't exist. None require a change.

## Alternatives considered

**The shared, host-global `app-server daemon`** (§ above) — refused on the
measured dependency gap (R5 of the R1–R7 ladder), the blast-radius argument,
and the auth-posture question being J0's, not a wave's.

**A typed `Capabilities` v2 enum for this adapter alone** — refused
directly by R3; the adapter-local `AdmissionRow` ledger is the alternative
actually built, and is designed to be liftable whole into a real v2 when
one lands.

**Refusing an execution whose app-server child failed to spawn, by falling
back silently to exec** — refused (W3 §5.3): a silent transport downgrade
mid-registration would hand the engine a capability row it never
preflighted against. LAUNCH fails honestly instead; the operator sees why.

## Consequences

Codex is a fully registered, routed, capability-honest backend alongside
Claude and Docker. Every capability it claims carries a named admission
test or a named, specific negative — never a schema-only promotion. The
sandbox enforcement it defaults to is real (measured as far as this host
allows) but not proven-effective on Cerberus, and that gap is a fact about
this host, not a silently-assumed guarantee. The version-provenance
posture this sprint gave codex from birth is now this repository's stance
for every adapter, current and future — the strike in `claude.rs` was not
a one-off carve-out.

## Open questions

Whether app-server's `ask` admission would succeed at a frontier model
tier rather than the pinned dev tier (`gpt-5.6-luna`) is explicitly
unanswered — the negative recorded is real but tier-scoped, and a future
wave re-measuring at a different pin is re-opening a measured question,
not re-litigating a settled one. `history` via `thread/items/list` with a
completeness proof against the rollout, `thread/resume` across a stopped
owning process, and the shared-daemon question if a genuine `human_attach`
need ever appears (reopening the R1–R7 ladder at R5 with new facts) are
named hand-offs, not decisions made here.
