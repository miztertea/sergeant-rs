# Sprint plan — CI/CD hardening & version policy (2026-08-20)

Owner-commissioned same-day follow-on to the backlog close-out sprint. Source
material: the owner-supplied repo audit (`~/inbox/repo-analysis.md`, an external
LLM deep-research report) **as re-ranked and verified in conversation** — the
report is a lead sheet, not a spec. Every upstream version number it claims is
re-verified against live registries by the `cicd-audit-recon` workflow before
any spec cites it (results table below).

**Protocol** (same as backlog close-out): integration branch
`integration/cicd-hardening`, draft head PR carrying this plan, wave branches
`cicd/w<N>-<slug>` in `/var/tmp/cicd-impl/` worktrees, per wave:
recon → spec → implement → 4-axis blind panel (spec-fidelity / invariants /
simplicity / test-honesty) + per-axis refuters defaulting to refuted → fixer on
confirmed findings only → wave PR → merge to integration. Sonnet subagents by
default, opus where earned, Fable never in subagents. Only the owner merges
main. No GitHub issues filed for this sprint — this plan + the head PR
checklist are the tracker (the backlog was just cleaned; filing 15 issues to
close them same-day is churn).

**Base**: PR #202 merged to main 2026-08-20 21:47 (merge commit 74fc2deb) —
the owner merged before this sprint started. `integration/cicd-hardening`
sits on main's tip; the head PR targets main with no stacking. v0.1.2 is
merged but not yet tagged/released — **owner ruled 2026-08-20: this sprint
ships as part of 0.1.2.** No version bump; the release waits for this head
PR, then the owner tags v0.1.2 once (its first run also proves #200's
repaired Latest pipeline — now with this sprint's hardened release path).

**Version**: **0.1.2** (owner-ruled) — the bump already landed with #202;
this sprint adds no version change, only extends the 0.1.2 CHANGELOG section.

**Standing constraints** (owner rulings carried forward):
- Designed for Linux/macOS/WSL — never tuned to Cerberus. Concretely: alpine
  gets a minor-line pin, **not** a manifest digest; runner pinning stops at OS
  generation, not self-hosted images.
- `Cargo.toml` keeps compatible semver ranges; `Cargo.lock` is the exact pin;
  never mass-`=x.y.z`.
- Internal `/vN` identifiers (`sergeant.watch/v1`, `sergeant.event/v1`,
  `sergeant.runtime/v1`, `execution/v1`) are compatibility contracts, never
  update targets.
- MSRV is **low priority** (binary ships prebuilt; `rust-version` serves only
  from-source builders) — in scope as a cheap W1 tail item, dropped without
  ceremony if measurement drags.

## The one-rule target state

> Discover the newest acceptable version on an upgrade branch, qualify it
> completely, commit the exact working state, and use non-blocking canaries to
> discover the next change.

Pin at the correct layer: compatible ranges in Cargo.toml, exact graph in
Cargo.lock (+ `--locked` everywhere), exact compiler in rust-toolchain.toml,
full-SHA action pins, exact helper-tool releases, explicit runner generations,
checksummed/attested artifacts.

## Verified upstream versions (recon results)

All numbers below verified live 2026-08-20 by the `cicd-audit-recon` workflow
(5 sonnet agents, evidence URLs in `recon-results.json` alongside this plan).

| Item | Audit claimed | Recon verified (live) | Delta |
|---|---|---|---|
| Rust stable | 1.98.0 | **1.98.0** — released *today* 2026-08-20; no 1.98.1 | confirmed |
| `result_large_err` | "new in 1.98" | exists since 1.66; **1.98 fixed its async-fn false negative** (clippy #17130) — why it newly fired on `with_analytics` | audit framing corrected |
| taiki-e/install-action | v2.86.3 | **v2.86.4** @ `a2a5f6e99e1a31540baa0468acfa302cff0f359f` (also released today; ~daily cadence) | audit stale |
| actions/attest | v4.2.2 | **v4.2.2** @ `1e69f48acb82d1966a394da916b4c1698aa569d6`; inputs `subject-path` + `sbom-path`, README-quoted | confirmed |
| attest-sbom deprecation | claimed | **confirmed verbatim** ("being deprecated in favor of actions/attest… inputs are compatible") | confirmed |
| actions/checkout | v7.0.1 | v7.0.1 still latest — repo's 12 pins current; only release.yml:338 (v4.4.0) stale | confirmed |
| upload/download-artifact, dependency-review, rust-cache, cargo-deny-action | current | all current at repo's exact pinned SHAs | confirmed, no action |
| cargo-llvm-cov | 0.9.0 | **0.9.0** (crates.io max_stable) | confirmed |
| cargo-deny | 0.20.2 | **0.20.2** | confirmed |
| cargo-cyclonedx | 0.5.9 | **0.5.9** | confirmed |
| cargo-dist | 0.32.0 current | **0.32.0** current — repo pin not stale; crates.io name still `cargo-dist` (the `dist` crate is an unrelated squat) | confirmed |
| dist installer verifies checksums? | audit assumed patchable | **NO — upstream gap**: generated installer.sh ships `verify_checksum()` but no code path populates `_checksum_style`/`_checksum_value` for shell artifacts; always prints "no checksums to verify". cargo-dist book: "still a work in progress" | audit's fix premise wrong |
| install-action supports cargo-dist? | (unasked) | **No** — no manifest, zero TOOLS.md hits. Bootstrap hardening needs download→verify→execute or `cargo install --locked` | new finding |
| reqwest | 0.13.4 | **0.13.4**; migration TRIVIAL for our surface — one Cargo.toml feature rename (`rustls-tls`→`rustls`), zero src changes; async-only usage inventoried at every call site. Watch: ring→aws-lc-rs default provider; MSRV 1.85 | confirmed + derisked |
| thiserror | 2.0.20 | **2.0.20** | confirmed |
| ubuntu-latest | 24.04 | **Ubuntu 24.04 x64** (26.04 exists but preview) → pin `ubuntu-24.04` | confirmed |
| macos-latest | unresolved | **macOS 26 arm64** → pin `macos-26` (same image the alias serves today; numbering jumped 15→26) | resolved |
| alpine 3.x | unspecified | latest cycle **3.24** (patch 3.24.1) → pin `alpine:3.24` (minor line, per Cerberus rule) | resolved |

## Waves

### W1 — Toolchain determinism
The local/CI compiler skew that reddened #202's CI is the motivating incident:
local clippy 1.97.1 passed while CI's floating `stable` (1.98) failed.
- Pin `rust-toolchain.toml` `channel` to the exact current stable
  (recon-verified; expect 1.98.x). rustup then gives the dev box and CI the
  same compiler.
- Thread the pin through CI — with the precedence stated CORRECTLY (panel
  fix): rust-toolchain.toml (rustup precedence rank 4) already outranks the
  `rustup default` that `dtolnay/rust-toolchain@stable` sets (rank 5), so
  the pinned file alone controls which compiler bare `cargo` calls use. The
  six action sites (ci.yml:34,85; coverage.yml:40; matrix.yml:65;
  release.yml:274,417) are therefore not overrides but pre-installers of the
  WRONG (floating) toolchain — every CI job would pay an unplanned on-demand
  fetch of the pinned one at first `cargo` call. Spec's task: make the
  pre-install match the file under a hard constraint of **one source of
  truth** (e.g. read the channel out of rust-toolchain.toml, or replace the
  action with a `rustup show`-style warm-up honoring the file), preserving
  coverage.yml's `llvm-tools` component however it lands.
- `--locked` on every cargo invocation across all four workflows (current
  count: zero), plus a cheap first guard
  (`cargo metadata --locked --format-version 1 >/dev/null`) and a
  `git diff --exit-code` post-qualification step in ci.yml.
- Tail item (droppable): measure MSRV empirically from 1.85 upward
  (`cargo check/test --locked`), declare `rust-version`, add a check-only MSRV
  CI job (no clippy on old compilers — lint policy belongs to the pinned dev
  compiler).

### W2 — Actions, runners, canary
- Stray `actions/checkout` v4.4.0 at release.yml:338 (package-installer job) →
  the v7.0.1 SHA already used at 12 other sites.
- `taiki-e/install-action` v2.86.2 → v2.86.4 @
  `a2a5f6e99e1a31540baa0468acfa302cff0f359f` (full-SHA pin; project ships
  ~daily patches — the pin records today's qualified one, the canary finds
  the next).
- Exact-pin the three mutable `tool:` inputs: `cargo-llvm-cov@0.9.0`
  (coverage.yml:45), `cargo-deny@0.20.2` (release.yml:202),
  `cargo-cyclonedx@0.5.9` (release.yml:423) — all three ARE in
  install-action's manifest set (verified).
- `runs-on: ubuntu-latest` → `ubuntu-24.04` and `macos-latest` → `macos-26`
  (arm64 — exactly what the alias serves today) in every required job across
  ci/coverage/matrix/release. Pinning to the alias's current target means
  behavior is unchanged on day one — that IS the qualification; the wave PR's
  own CI run on the pinned runners is the proof.
- New canary workflow (`canary.yml`): weekly schedule + `workflow_dispatch`,
  floating `stable` Rust on `ubuntu-latest`/`macos-latest`, runs
  check/clippy/test `--locked`. Panel fix — the canary must be LOUD when it
  trips, or it's not a canary: NO `continue-on-error` (it's a standalone
  scheduled workflow, not a required PR check, so a red run blocks nothing
  and GitHub's default scheduled-failure notification fires), plus an
  `if: failure()` step that opens-or-updates a single "upstream drift"
  tracking issue with the failing job link. Required CI stays deterministic;
  a red canary is the trigger for a deliberate upgrade PR.
- `Record build environment` evidence step in release.yml (commit, runner
  os/arch, rustc -Vv, cargo, git, bash versions → build-environment.txt
  uploaded with release evidence).

### W3 — Release supply chain
- Replace both `curl …cargo-dist-installer.sh | sh` bootstrap sites
  (release.yml:282, :342). install-action does NOT carry cargo-dist
  (recon-verified: no manifest, zero TOOLS.md hits), so the mechanism is
  download-to-disk → verify → execute. Wave recon determines what axodotdev's
  v0.32.0 release actually publishes to verify against (checksum sidecar
  and/or `gh attestation verify --repo axodotdev/cargo-dist`); fallback if
  neither exists: `cargo install cargo-dist --locked --version 0.32.0`
  (slower, but source-built against our own lockfile discipline).
- Sergeant's own installer verifying SHA-256 before extraction: release.yml's
  own comment (~:315-326) records this as a known gap deferred as "the
  owner's call" — the owner has now made that call via this sprint. But recon
  falsified the audit's premise: dist 0.32's generated installer ships a full
  `verify_checksum()` implementation that is NEVER fed — no code path
  populates `_checksum_style`/`_checksum_value` for shell artifacts (verified
  against our own v0.1.1 release asset), and the cargo-dist book calls shell
  installer checksum wiring "still a work in progress". No config flag or
  version bump fixes this. The spec chooses between exactly two honest
  options and the choice is a **ratify-at-review** item:
  (a) post-process the generated installer in `package-installer` to inject
      the per-artifact sha256 values dist itself computed (the `.sha256`
      sidecars), gated by BOTH a positive smoke test and a corrupted-archive
      negative test — the injection targets the exact hook `verify_checksum()`
      already exposes, so the patch is data, not logic; or
  (b) don't touch the generated artifact: keep the README's documented manual
      `sha256sum -c` + `gh attestation verify` path as the verification
      story, record the upstream gap in docs/version-policy.md, and revisit
      when cargo-dist lands its own wiring.
  My recommendation going in: (a), because the verification hook is
  upstream's own and the negative test makes the patch honest — but the spec
  must prove the injection survives dist regenerating the installer
  byte-for-byte differently across targets before committing to it.
- Migrate the two `actions/attest-sbom` v4.1.0 steps (release.yml:499,:505)
  to `actions/attest` v4.2.2 @ `1e69f48acb82d1966a394da916b4c1698aa569d6`.
  Deprecation confirmed from attest-sbom's own README; inputs are
  declared compatible (`subject-path` + `sbom-path`, quoted from the live
  actions/attest README) — a low-risk swap.
  **Preserve** the 1:1 target↔SBOM pairing the current design defends in its
  comments, and the existing least-privilege permissions layout.
  (`attest-build-provenance` is already current at v4.2.2 — untouched.)
- `alpine:3` probe image (src/backend/docker.rs:138 PROD_PROBE_IMAGE, plus
  test fixtures :1350,:1475,:1521,:1523) → `alpine:3.24` (current cycle,
  patch 3.24.1). Minor pin only, per the Cerberus rule.

### W4 — Dependencies, docs, contracts
- reqwest direct dep 0.12 → 0.13.4: our direct line is the only holder of
  0.12.28 in the lockfile while opentelemetry-otlp already brings 0.13.4 —
  migration deletes a duplicate HTTP/TLS stack. Recon already inventoried
  every `reqwest::` call site against 0.13.0's breaking-change list: our
  surface (async `Client::builder().timeout().build()`, `.send()`,
  `.json()` both directions, `reqwest::Error`/`Response` types) is untouched;
  the ONE required change is the Cargo.toml feature rename
  `rustls-tls` → `rustls`. Two verifications the wave must still run:
  grep for `CryptoProvider`/`rustls::crypto::ring` conflicts (0.13 defaults
  the provider to aws-lc-rs) and confirm both platform builds in CI. Gate:
  full test suite + `cargo tree -i reqwest@0.12.28` returns nothing.
- thiserror 2.0.19 → 2.0.20 (precise bump).
- README release literals: two `v0.1.0` sites (README.md:63,:83) are now two
  releases stale. Spec decides per-site between `/latest` URLs (the badge
  #200 fixed now makes Latest trustworthy) and current-literal + a CI
  docs-consistency check (version extracted from `cargo metadata`, grep for
  stale literals — new cheap step in ci.yml). Recurrence prevention is the
  requirement; the mechanism is the spec's call.
- `docs/version-policy.md`: the policy table (pin-at-the-correct-layer rules
  above, including what is deliberately NOT pinned and why — alpine digest,
  crate `=` pins — so future audits don't re-litigate).
- Contract tests for the internal schema identifiers — precise inventory
  first (panel correction: the runtime descriptor schema is already
  `sergeant.runtime/v2` at daemon.rs:58, not v1; current set is watch/v1,
  event/v1, execution/v1, runtime/v2). Verify what coverage already exists
  (event.rs and daemon.rs already assert schema strings), add golden
  fixtures only where missing. Small; no protocol changes.
- Stale-comment truth pass (audit Low item, panel-verified live instance):
  ci.yml:56 claims matrix runs "PR-to-main, nightly, and release" but
  matrix.yml is `workflow_call`/`workflow_dispatch` only (ADR-0014-D9
  topology). Fix that comment and sweep workflows for similar
  topology-drift comments.
- Audit Low-item dispositions, stated so nothing drops silently:
  duplicate-transitive-version documentation → a short subsection of
  docs/version-policy.md (the one named duplicate, reqwest 0.12/0.13, is
  eliminated by this very wave); machine-readable non-Cargo tool inventory →
  **deliberately declined** (Cerberus rule — version-policy.md's prose table
  is the inventory; a schema for two shell scripts and four tools is
  process for its own sake).

### Finalize (head PR release-ready)
Extend the existing 0.1.2 CHANGELOG section with this sprint's entries (no
version bump — owner-ruled), Cargo.lock sync (reqwest/thiserror), README
consistency re-check against 0.1.2, full local gate (fmt/clippy/test with
the pinned toolchain, `--locked`), head PR with wave checklist +
ratify-at-review items.

## Wave ordering & conflict control
Strictly serial W1→W2→W3→W4. W1↔W2↔W3 is conflict-forced (W1's pin +
`--locked` change every workflow file W2/W3 also touch; W2 and W3 both edit
release.yml). W3→W4 is NOT conflict-forced (panel note: W4 touches only
Cargo.{toml,lock}, README, docs, fixtures) — serial there is a deliberate
simplicity choice: one integration head to reason about, one wave in flight,
same rebase discipline throughout. Each wave rebases on integration head
before its PR.

## Ratify-at-review items (owner, at head PR)
1. ~~Version~~ — ruled in-session: ships in 0.1.2, no bump.
2. Installer verification: option (a) checksum injection vs (b) documented
   manual verification + upstream wait — spec's choice flagged in the PR body
   with the dist-gap evidence.
3. macOS pin `macos-26` (arm64 — what `macos-latest` already serves; flagged
   in case the owner prefers `macos-15` for an extra generation of headroom).

## Risks
- reqwest 0.13 migration is the only change touching runtime code paths
  (webhooks/OTLP HTTP); mitigated by wave recon + full suite + panel.
- Pinned-runner day-one equivalence depends on recon reading the alias
  mapping correctly; mitigated by CI itself (wave PR runs on the pinned
  runners).
- (resolved) The plan originally stacked on unmerged #202; the owner had in
  fact already merged it — branch was reset onto main @ 74fc2deb.
