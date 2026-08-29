# Changelog

All notable changes to sergeant-rs are recorded here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/); Sergeant is pre-1.0, so
ordinary SemVer 0.x.y semantics apply and every release is potentially
breaking (proposal-ci-cd-release-engineering.md §8).

`release.yml`'s Gate A requires this file to mention the version being
released before a release can proceed.

## [Unreleased]

(nothing yet)

## [0.3.0] - accumulating

Not yet released: per the Host+Atlas release-shape ruling, v0.3.0 ships once,
at program completion, carrying the whole Host+Atlas program (this sprint's
host runtime plus S2-S6's knowledge/intelligence stack). This section
accumulates sprint over sprint on the integration branch; each sprint close
still runs the full release pipeline in dry-run against the integration
head, so every gate below is proven at every sprint boundary even though
nothing is actually tagged or published until the program-completion
release.

### Host runtime (S1)

- One Sergeant daemon per user installation now serves every admitted
  estate, not one daemon per estate: lazy, observational admission by
  exact root, a descriptor schema bump to `sergeant.runtime/v3` (no baked-in
  estate set — the admitted registry is dynamic daemon state), and a
  systemd user service / macOS LaunchAgent installer (`sgt daemon
  install-service`, with `--print` to inspect the generated unit without
  writing it) so the daemon survives logout and restarts on crash
  under native per-user service management. `sgt doctor` gains a
  `host_service_manager` row reporting whether a native per-user service
  manager (systemd on Linux, launchd on macOS) is reachable from this
  session. (#275, #276)
- Journal, DuckDB projection, and daemon runtime state moved to a shared
  host runtime root; estate-local material (manifest, repository mounts,
  `.sergeant/workflows/`, Work surfaces) stays exactly where it was.
  `[estate] data_dir` is deprecated at cutover; `sgt doctor` gains a
  `legacy_estate_runtime` row warning when legacy daemon runtime state
  (`.sergeant/data/{daemon.lock,runtime.json,journal/}`) is still present on
  disk.
- Retention is now partitioned per admitted estate (own `[estate]
  retention`, resolved fresh every prune cycle) while the blob-reference
  scan that garbage collection depends on stays journal-wide, so one
  estate's retention can never condemn another's still-live blobs. Both
  the rotation-triggered prune tick and the daemon-start prune trigger
  use the same per-estate policy resolution.
- A bounded execution capacity lane (`Arc<Semaphore>`, sized by
  `SGT_EXECUTION_LANE_CAP`, acquired between PREPARE and LAUNCH, outside the
  core lock) caps concurrent native-adapter
  launches daemon-wide; a lane-queued Work is observably distinct from
  turn-cap exhaustion. A second, independently bounded config-only
  intelligence lane exists alongside it with no scheduler wired to it yet
  — proving the two lanes' independence now rather than retrofitting it
  once intelligence workers exist.
- The TUI widens to a Host / Estate / Work / Stage / Execution scope: the
  fleet endpoint returns every admitted estate's Work and the TUI filters
  client-side; `sgt watch` keeps estate-scoped meaning by default inside an estate
  (host-wide outside one) and gained `--all` to watch every admitted estate.
- One bearer token now authorizes every estate the daemon has admitted —
  a real widening of blast radius from the one-estate world, accepted
  deliberately under this installation's single-user trust model and
  recorded, not silently inherited (see [host runtime and
  estates](docs/concepts/host-runtime.md)).
- Startup backend probes now run concurrently instead of sequentially,
  and the runtime descriptor is published before the probe walk finishes
  rather than after — cutting fresh-daemon time-to-healthy well under
  the client's own auto-spawn wait on a fully-provisioned host. (#293)
- `[profile.dev.package."*"] debug = false` (workspace crate keeps its
  own debuginfo) — a measured, qualified cut to fresh-build size and
  incremental compile time with no loss of in-tree backtraces. (#299)
- Fixed a doctor false positive in the embedded-content route checker.
  (#282)
- Documentation frame extended with host-runtime/cutover prose (service
  installation, the legacy-runtime reconcile-or-abandon remedy, daemon
  stop's host-wide blast radius) and a corrected Cerberus cold-build
  figure (~2m18s for a solo cold `cargo build --tests`; up to ~4 minutes
  under measured concurrent-build contention — not the stale ~10 minute
  figure that had been carried from an earlier, differently-provisioned
  environment).

### Hierarchical execution (S2)

- A workflow stage directory may now itself hold a `workflow.toml`: the
  loader recurses (embedded and repository packages alike), splicing the
  nested package's leaves into the parent's one flat `stages` list at the
  container's position, with `parent/child` composed hierarchical stage
  ids. The container itself is never a `StageDefinition` or a
  `StageRecord` — it enters and completes no event of its own; a
  container "completing" is simply its last flattened leaf completing.
  Nesting is unbounded by design; a package that resolves to one of its
  own ancestors (a symlink cycle) fails closed by name at load time
  instead of overflowing the recursion. A stage directory carrying both
  `workflow.toml` and `CONTEXT.md` is a load error — a container has no
  actor, so its `CONTEXT.md` could never be read.
- A closing container's own declared output contract is checked at its
  boundary-closing leaf's completion, reusing the existing output-gate
  mechanics verbatim: a bounded re-prompt of that leaf, then a park
  naming the *container's* id if still unmet. A leaf that simultaneously
  closes several ancestor containers gates them innermost first. Nested
  leaves' own output/required-column/finalize contracts behave
  identically to top-level ones and are never bypassed by container
  completion.
- Composed hierarchical stage ids journal, replay, and render correctly:
  the analytics fold, `sgt work show`, and a daemon restarted mid-nest
  all carry/reconstruct the exact composed id from the journal alone —
  recovery needs no process tree, only the last `StageRecord`. The
  workflow's own wire shape stays the flat leaf list (hierarchical ids as
  opaque strings); the TUI's workflow rail draws that flat list as the
  tree it is.
- `sgt -C <estate> run` from inside a managed execution can create an
  ordinary, separately admitted child Work. Every managed execution now
  carries an estate/Work/execution causation triple
  (`SERGEANT_ESTATE_ROOT`/`SERGEANT_WORK_ID`/`SERGEANT_EXECUTION_ID`,
  distinct from the harness's own `SGT_ESTATE_ROOT`) to every adapter's
  spawned process, survives a daemon restart, and — when the child's `sgt
  run` inherits and spends it — the daemon validates the claimed parent
  against its own journal before recording the relation. A claim that
  fails validation is never refused: the submission proceeds as an
  ordinary causation-less Work, and the daemon journals an explicit,
  visible `causation_unverified` marker naming the failed claim (journal
  is truth; substance is preserved by admission and explicit addressing
  rather than by refusal). Child Work has fully independent scope,
  surface, and lifecycle — parent completion, cancellation, or (there
  being no merge primitive in the engine at all) any notion of merge
  never cascades to it, and a bare `sgt run` from inside a Work surface
  still refuses; only explicit `-C` addressing is exempt.
- `sgt run --wait` observes the Work it just submitted through to a
  terminal state client-side (the existing watch mechanism, scoped to
  the new Work id) — no new engine hold state.
- The Fleet TUI groups a child Work immediately under its own parent,
  recursively, indenting the intent cell one level per ancestor hop — the
  causal-child tree, derived entirely from a row's own already-projected
  `parent` field (no second request per Work). A child whose parent
  isn't currently visible (filtered out, or aged out of the daemon's
  caches) renders as a root rather than being dropped.
- `sgt doctor` gained a `workflow_stage_declarations` row: a
  directory inside a workflow package that looks like a stage (holds
  `CONTEXT.md`, `README.md`, or its own `workflow.toml`) but isn't named
  in that package's declared `stages` warns, naming the package and the
  directory (or, for an undeclared nested package, the whole unreachable
  subtree) — a real directory on disk the loader will never reach. Warn,
  not fail: this is an authoring-drift observation, not a broken
  declaration.
- Fixed a probe-child leak: a killed probe's own children (and their
  children) now die with the probe and with the daemon instead of being
  reparented and left running, orphaned. (#310)
- The test suite runs roughly 40% faster: ten small no-daemon integration
  suites consolidated into one harness binary (`c2_light`, paying one
  link instead of ten), `cargo-nextest` adopted (exact-pinned) locally,
  in CI, and in the coverage lane, and `ci.yml`'s fmt/clippy job split
  from the test job so wall-clock is the slower of the two rather than
  their sum. Fixed a cross-process-re-entrant OTLP-disabled test that the
  consolidation surfaced. (#305)
- Doctrine amendment: `AGENTS.md`'s ESTATE bullet 5 and `icm-policy.md`'s
  rule 1 now point at the sanctioned child-Work path (`sgt -C run`) as
  the real nesting/possession primitive, replacing an ambiguous
  possession-vs-injection reading; a new `docs/concepts/
  hierarchical-execution.md` page documents nested packages and child
  Work for operators.
- The W1 §13 acceptance battery (`tests/v4_w1_acceptance.rs`) walks all
  nine acceptance criteria literally, citing the named pin for each
  already-proven claim and adding one self-contained structural test for
  the one gap (no merge primitive exists in the engine to cascade
  through); item 9's required-column half now has its own nested-leaf
  test (`tests/m11_nested_workflow.rs::a_nested_leafs_required_column_contract_is_enforced_exactly_as_a_flat_ones_is`)
  and its finalize half cites the pre-existing
  `src/runtime/surface.rs::tests::the_finalize_sweep_reaches_a_nested_leafs_output`.

#### V4 live evidence

- **Opt-in live suites, serially, against the real `aria` opencode
  endpoint** (`SERGEANT_OPENCODE_TESTS=1 cargo test --locked --test
  opencode_backend -- --ignored --test-threads=1`): 8 passed, 0 failed
  (`live_opencode_history_exports_the_whole_session`,
  `live_opencode_minimal_turn_completes_with_usage`,
  `live_opencode_probe_reports_the_installed_version`,
  `live_opencode_resume_recalls_a_nonce_across_processes`,
  `live_opencode_serve_abort_yields_an_interrupted_terminal_and_a_usable_session`,
  `live_opencode_serve_actor_question_parks_and_resumes_on_answer`,
  `live_opencode_serve_approval_round_trip_runs_the_gated_tool`,
  `live_opencode_serve_minimal_turn_completes_with_usage`); finished in
  140.55s. No capability differences from the measured `opencode`
  admission row surfaced.
- **Real end-to-end live proof** on a scratch estate
  (`/var/tmp/hats2/v4-live-proof/estate`) against this box's real daemon
  binary, bound to a scratch host-mode data dir via `SGT_DATA_DIR`
  (`/var/tmp/hats2/v4-live-proof/data`) — never the estate's own
  production journal. A real Work on the `opencode` backend (the `aria`
  model) ran a workflow whose `CONTEXT.md` instructed the actor to submit
  a child Work via the sanctioned `sgt -C "$SERGEANT_ESTATE_ROOT" run`
  path; both parent (`01M10KPMV6K8JD6GYM1ZW0WA2V`) and child
  (`01M10KR5W2ERFPCHMWH1ECQNNV`) reached `work.completed`. The child's own
  `work.submitted` journal payload records the validated relation
  verbatim: `"parent_work_id": "01M10KPMV6K8JD6GYM1ZW0WA2V"`,
  `"parent_execution_id": "01M10KPQVS6ZJMT98CGBVP586S"` — no
  `causation_unverified` marker, i.e. the claim validated clean against
  the daemon's own journal, not merely accepted unverified.
  - **A first attempt at this same proof produced no parent relation at
    all** (neither `parent_work_id` nor a `causation_unverified` marker):
    the actor's shell resolved the bare `sgt` on `$PATH` to a stale
    installed binary (`~/.cargo/bin/sgt`, built before this sprint's
    causation transport) rather than this branch's own build, so the
    submitted child never claimed a parent to begin with. Not a
    sergeant-rs defect — a live-environment `$PATH` artifact of this one
    run — diagnosed by comparing binary contents (`strings … | grep
    claimed_parent_work_id`) between the two, then re-run with the
    branch's own build first on `$PATH`, which produced the clean result
    above. Recorded here per the brief's own honesty requirement rather
    than silently discarded as a bad take.

### Atlas substrate (S3)

- Atlas — the world-intelligence store — gets its own module tree
  (`src/runtime/atlas/`) and its own database file, `<data-dir>/atlas/
  atlas.duckdb`, deliberately outside the disposable `projections/`
  directory. `runtime/atlas/db.rs` is its single owning file: it is the
  only module in that tree that reaches the database driver, and it hands
  no live connection out. Two independent structural tests now hold two
  independent one-owner invariants — M5's keeps naming `runtime/
  analytics.rs` as the operations projection's sole owner, and a new suite
  (`tests/x1_atlas_substrate.rs`) names `runtime/atlas/db.rs` as Atlas's.
  They are never merged into one "either of these files may open a
  database" rule.
- Atlas declares the `meta`, `source`, `git` and `context` schema
  namespaces. Every table lands in the change that lands its writer — the
  same empty-table refusal doctrine the operations projection already
  applies — which is why `git` carries no table of its own: a repository, a
  knowledge path and a Work overlay produce the same kind of fact, so
  repository-derived rows live in `source.*` with the source kind as a
  column rather than in a schema of their own.
- The two stores do not share a rebuild discipline, and both module docs
  say so: the operations projection is deleted and re-folded from the
  journal on every daemon start, while Atlas's `source.*`, `context.*` and
  `meta.coverage` persist across restarts — they are derived from source
  bytes plus extractor identity, keyed by source generation, and a
  generation is evicted only when those bytes change.
- The ten operations tables moved into an `ops` schema inside
  `sergeant.duckdb`. Physical requalification only: every table name the
  daemon reports (`/v1/analytics`, `sgt analytics`) and every answer it
  gives is unchanged, and the existing projection suite passes as-is.

### Knowledge sources and coverage (S3)

- `sergeant.toml` gains `[[knowledge]]`: a named local path the estate reads
  as **evidence**, with optional `ignore` globs. It is never a mount —
  nothing is cloned, no worktree is cut from it, nothing writes to it — and
  a declared path that resolves inside a repository mount, inside
  `surfaces_dir`, or inside `data_dir` is refused by name at manifest parse,
  because those are exactly the locations the estate itself mutates.
  `sgt knowledge add`/`list` declare and read them back; both are pure
  manifest operations with no daemon involved.
- Atlas gains its first real writer, the local-knowledge scanner, and with
  it the four tables it populates: `source.generations`, `source.files`,
  `source.units` and `meta.coverage`. Markdown and plain text are extracted
  into document and heading-delimited section units carrying byte offsets
  into the original file, so every derived unit can be traced back to the
  bytes it came from. Extraction is a set of pure functions over bytes; the
  database glue around them is separate and thin.
- Secrets are excluded at the acquisition boundary, before a file is opened:
  dotfiles and dot-directories, `.env` files, private keys, keystores, and
  credential/secret files by convention, plus each source's own `ignore`
  globs — which extend that floor and can never narrow it. Excluded paths
  are **counted and reported** as excluded, with the pattern that refused
  them, rather than being silently absent.
- Every path a scan sees leaves exactly one coverage row (`discovered`,
  `indexed`, `excluded`, `unavailable`, `unsupported`, `error`, or
  `generation_evicted`), and each completed scan journals exactly one
  compact `source.scanned` summary — source, generation, content key,
  counts, extractor identities — never a per-file event stream.
- Re-scanning a source whose bytes have not changed writes nothing and
  evicts nothing; a generation is superseded only when the content it was
  derived from actually changed, and the superseded generation leaves an
  explicit eviction row rather than vanishing.
- A crash between a scan's rows and its summary now leaves neither reported.
  Rows commit as one provisional generation, the summary is journaled, and a
  second transaction confirms it and evicts its predecessor; no read path
  can see an unconfirmed generation, and opening Atlas — which every daemon
  start does — evicts any that a crash left behind, leaving a
  `generation_evicted` coverage row that names the crash window.
- New dependency: `globset`, for the exclusion set and `[[knowledge]]
  ignore` glob matching.

### Estate-repository indexing (S3)

- Atlas now indexes declared repositories as well as knowledge paths. A
  repository is read **at the commit its Work admission pinned**, out of the
  Git object store — never from the mount's working tree, and never by
  fetching, pulling, switching or writing anything. A scan running while the
  mount's HEAD moves stays on the commit it pinned; the move is reported as a
  drift observation beside the scan, never blended into it.
- Blob reads are batched: one `git cat-file --batch` process answers many
  objects, instead of one Git process per file.
- Cached extractions of repository content are keyed by **Git's own blob
  object id** plus the extractor's identity. Bytes Git has already hashed are
  never hashed a second time, and two identical files share one extraction by
  construction. A repository generation is identified by its tree, so a commit
  that changed no file — an empty commit, a reworded message — is recognized
  as the same world and evicts nothing.
- A Work surface is indexed as an **overlay** over its base commit: files the
  Work changed are content-hashed from the surface, every unchanged file keeps
  the base tree's object-id key, and the generation is identified by the base
  commit composed with a digest of the changes. Overlay evidence is scoped to
  its Work and removed when that Work is retired, leaving an explicit eviction
  row.
- The intelligence capacity lane declared in the host-runtime work now has a
  real consumer: extraction acquires it, runs on the blocking pool, and is
  bounded by `SGT_INTELLIGENCE_LANE_CAP`. It never draws on the execution
  lane, so indexing a large repository cannot reduce the number of Works that
  can run.
- No new dependency: the existing Git-CLI module gained a batched object-read
  primitive rather than the codebase gaining a Git library.

### Language-aware extraction (S3)

- Source files are now parsed, not just read. Rust, TOML, Markdown, Python,
  JavaScript, TypeScript and shell are indexed with tree-sitter grammars, and
  what they yield lands in three new tables: a **symbol index**
  (`source.symbols`), the **sites** that wrote each symbol
  (`source.occurrences`), and **import edges** (`source.edges`).
- Everything stored is **syntax, not semantics**. A symbol's label is what the
  grammar called the node — `function`, `struct`, `class`, `heading` — and an
  import's target is the text the file wrote, unresolved. Nothing follows a
  re-export or decides which definition a name meant, and nothing claims to.
- A file whose extension only a grammar claims — `.rs`, `.toml`, `.py`, `.ts` —
  is `indexed` on the strength of that grammar alone. A language no grammar in
  this build claims (`.tsx`, for instance) is `unsupported` and says so, rather
  than being parsed by an almost-right grammar.
- A file a grammar cannot parse is reported `error` and contributes **no**
  symbols at all. tree-sitter's error tolerance would otherwise produce a
  shorter symbol list that nothing downstream could distinguish from a
  complete one.
- The syntax extraction of a file is cached separately from its structure
  extraction: one blob read by two extractors is two extractions with two
  keys, so revising a grammar re-derives symbols without invalidating a single
  document unit. Repository content keys on Git's blob object id, exactly as
  before; local knowledge keys on its content hash.
- Extraction runs where the repository plumbing already ran — on the
  intelligence lane, over batched blob reads — and the scan summary each
  completed scan journals now carries symbol and edge counts.
- Eight new dependencies, all from crates.io, all MIT, none vendored or
  forked: `tree-sitter` and one grammar per indexed language. `cargo deny`
  is green with them, with no new duplicate-version warning.
- The declared minimum supported Rust version moves from 1.89 to 1.98,
  matching the toolchain this repository already pins.

### Tabular sources and the map surface (S3)

- CSV, JSON and Parquet files under a `[[knowledge]]` source are now indexed
  as **tabular datasets, read in place**: DuckDB opens the operator's own
  file through a canned, fully parameterized query and no copy of those bytes
  lands in Sergeant's store. Each dataset records where it is, what it hashes
  to, its columns, and a bounded row count; each canned query's answer is
  stored as derived evidence carrying the generation it read, the identity of
  the question (name, version and a digest of the SQL that ran) and a hash of
  its own output, so an answer can be checked rather than trusted.
- A deterministic per-column aggregate ships with it — rows, non-null rows
  and exact distinct values per column — ordered by column name, so two runs
  over an unchanged file produce byte-identical output.
- **A tabular row's text becomes a retrievable context unit only through an
  operator-declared column allowlist, and the default is none.**
  `[[knowledge]] context_fields = [...]` (or `sgt knowledge add
  --context-field`) names the columns that may be exposed; without it a
  dataset is still discovered, registered, counted and profiled in aggregate,
  and not one row's text is published. The declared columns are part of the
  reader's identity, so *narrowing* the list supersedes the generation and
  removes the units the wider one exposed rather than leaving them behind.
  `sgt knowledge list` reports each source's declared allowlist (`--json`
  included), so what a source may expose is auditable without opening
  `sergeant.toml`.
- A dataset file whose *path* contains a glob metacharacter (`*`, `?`, `[`)
  is reported `unsupported` rather than read. A tabular reader takes its path
  as a multi-file pattern, so such a name would make one read fan out across
  sibling files the source's `ignore` globs and the built-in deny set had
  excluded — recorded under provenance computed from only the named file.
- Row identity is content-derived where the data permits it — a row keeps its
  name when something above it is deleted — and honestly re-keyed where it
  does not: rows the allowlist cannot tell apart are all re-keyed with their
  position and labelled as such, so a consumer knows which claim it may make.
- New read surfaces: `sgt intelligence status` reports every indexed source's
  generation and its full coverage breakdown (including what was *excluded*,
  which is what makes the secrets posture checkable), and `sgt map
  repos|outline|symbol|references|stats` reads the derived world map, where
  `references` returns the definition occurrence sites a grammar can state
  without resolving anything — not resolved cross-file usages. All of
  them are canned and parameterized — no client SQL, no client-named path, no
  match pattern — and every read is bounded, with a row cap a client can lower
  but never raise. `map neighbors` and `map changed` are deliberately not
  shipped: they land with the waves whose consumers need them.
- `sgt doctor` gains an `atlas` row reporting indexed/excluded/unsupported/
  unavailable/error counts across sources, and warning when paths could not be
  read or extracted.
- The `duckdb` dependency gains its `json` and `parquet` features. Both are
  DuckDB's own readers compiled into the bundled library — **no new crate
  joins the dependency graph** — and extension autoloading, autoinstalling and
  community extensions are turned off and locked off on every connection, so
  reading a dataset can never become a network fetch.

### Atlas adapters, indexing, and the scan trigger (S4)

Atlas goes from a daemon-owned substrate with two demonstrated source kinds
and no operator-reachable trigger (S3) to an estate-wide indexing surface
with a real scan behind it: a supervised worker transport carries three real
content adapters (Office, ZIP, mail) out of the daemon's own process; a
fourth source kind (`external Git`) and a package-identity derivation join
`estate_git`/`local_knowledge`; `sgt intelligence scan` finally gives an
operator something to run — across all three source kinds, not only the one
S4 Y5 first wired — with a cloud-sync placeholder finally reported honestly
instead of silently as empty; and, closing this sprint's own signature
defect a third time (Y8, below), that scan actually **dispatches** a claimed
`.docx`/`.zip`/`.eml` to the worker rather than reporting it unsupported —
the adapters and the transport were built and proven through the real
subprocess from Y1 through Y4, but nothing in `scan.rs`'s or `git.rs`'s own
routing table claimed those three extensions until Y8 wired them, so no
installation before this release could actually reach any of the three from
an ordinary scan. What follows is the one arc, in the order it was built.

**Supervised parse workers.** Third-party document parsing moves out of the daemon and into supervised
  child processes: a worker takes bytes and returns a normalized batch, and
  the daemon remains the only writer Atlas has. The adapters themselves did
  not change shape — they were already pure functions over bytes — so this
  is a transport boundary, not a new architecture.
- **The daemon's validation of a returned batch is the authority, not a
  formality.** Every batch is re-checked before a row is written: the
  identity triple (source generation, resource hash, extractor identity),
  path safety for every child resource the worker declares (the same
  containment rule an archive entry gets), and the secrets deny set matched
  on the declared *name* as well as the path. A worker that declares a child
  called `.env`, or one whose path escapes its parent, is refused with a
  named coverage row rather than trusted. The worker's own checks are
  defence in depth; they are not what the store relies on.
- Every worker spawn carries the child-lifetime discipline the probe leak
  established (its own process group, `PDEATHSIG`, a bounded deadline, and
  kill-plus-reap on *every* exit path, not only the timeout one), and runs
  under an intelligence-lane permit that is released even when the child
  panics or is killed.
- **A memory bound, because a deadline is a hang guard and not a memory
  guard.** On Linux each worker is spawned under an `RLIMIT_AS` address-space
  cap, so a child that allocates without bound dies by its own limit instead
  of raising host-wide memory pressure and letting the kernel's OOM killer
  choose a victim — which, on the machine this was developed on, repeatedly
  meant the session manager rather than the process actually at fault. The
  cap composes with the existing process hardening rather than replacing it,
  and the child cannot raise its own ceiling before `exec`.
  - **Platform honesty:** this is a Linux mechanism. On macOS the worker is
    still supervised, still deadline-bounded, and still killed and reaped —
    but no memory containment is claimed there, and the module says so
    rather than implying the guarantee is universal.
  - **Number honesty:** the 512 MiB ceiling is provisional and unmeasured.
    No real parser ships yet, so there was no corpus to size it against; it
    must be re-derived once one lands. The effective host ceiling is that
    figure multiplied by the intelligence lane's concurrency cap, not the
    per-child number alone.
  - A memory kill is reported as its own named fault when the evidence
    supports it, matching a deliberately narrow set of real allocator
    failure signatures. When it cannot be attributed that precisely, the
    fault is still reported honestly as a worker failure — never absorbed,
    and never relabelled as a timeout.
- `sgt doctor`'s `atlas` row now opens the store read-only: it creates no
  database file and runs no schema statements to report on one. The row's
  behaviour is otherwise unchanged — it still checks for the file first, and
  still defers to the daemon when a running daemon holds the store.

**Office document adoption.** `.docx` documents now normalize into document units, running the `anydoc`
  crate behind a narrow adapter — bytes in, our own vocabulary out — inside
  the supervised worker transport the previous entry shipped. The owner
  ruled to adopt anydoc past a RUSTSEC advisory reached through one of its
  optional PDF-conversion dependencies
  (`knowledge/rulings/owner-rulings/anydoc-adoption-2026-08-27.md`): a
  per-format alternative would have defeated the "any doc" capability the
  contract actually asks for, and the advisory is accepted risk in a
  pre-alpha, open-source, MIT harness. `deny.toml` records the exception
  scoped to that one advisory ID, with a proof that a *different* advisory
  reached through the same dependency subtree still fails the gate.
- **The replaceability boundary the ruling conditioned adoption on is
  structural, not promised.** No anydoc type, error, or concept crosses out
  of the adapter's own module — a dedicated test scans every source and test
  file in the crate and fails if the crate's name appears anywhere else, the
  same shape the existing one-owner `duckdb` tests already use. Document
  units carry provenance in this crate's own terms: a normalizer identity
  and version, the original resource (never a temp path), and — where
  recoverable — a coordinate. For an Office section that coordinate is
  structural (`block:<n>`) rather than a byte range: the original bytes are
  a compressed container the normalizer has already unpacked, so no byte
  offset in them corresponds to a position in the extracted text, unlike
  plain text or Markdown, which keep exact byte offsets as before.
- **No new schema variant.** Office units stay `Document`/`Section` — the
  same two kinds Markdown already produces — because a `.docx` normalizes
  into the identical shape: one whole-resource unit, plus flat sections
  delimited by headings. The decision and its reasoning are recorded on the
  schema type itself, not left as an implementation detail.
- **This is also where a real parser first runs inside the supervised
  worker**, and the previous entry's supervision contract is proven against
  it: a genuinely malformed document (well-formed ZIP, invalid XML) and a
  genuinely hostile one (a small file that decompresses far past the
  adapter's own internal size ceiling — a classic zip-bomb ratio) each fail
  their worker alone. The worker process exits, the coverage row names why,
  no partial units are ever written, and the process supervising it stays
  up and reusable. The adapter is stricter than anydoc's own default:
  anydoc's XML layer will *repair* a well-formedness problem rather than
  fail outright, and the adapter treats a document that needed repair the
  same as one that could not be read at all, rather than trusting a
  silently patched result.
- A hand-verified `.docx` fixture corpus (five documents plus the hostile
  one above), with expected counts written down and independently
  cross-checked before any extractor existed, backs every claim above.

**Bounded ZIP containers.** ZIP archives now expand into child resources — bytes in, admitted
  entries out — running the `zip` crate (the maintained `zip2` fork) behind
  a pure-function adapter inside the same supervised worker transport the
  Office adapter uses. `enclosed_name` (already cited for path safety) is a
  path-STRING validator only: it rejects NUL bytes, absolute paths, drive
  letters, UNC prefixes and `..` traversal, and says nothing about entry
  TYPE. This wave adds the checks it does not cover, each its own named,
  never-silent refusal: a symlink (checked first and unconditionally of the
  entry's raw name, so a symlink whose name happens to end in `/` cannot be
  misfiled as a directory marker), or any other non-regular-file entry —
  FIFO, character device, block device, socket — refused by masking the
  entry's own Unix mode bits directly rather than trusting `zip`'s own
  `is_file()`, which does not check for a regular file at all; an empty
  name, a name that duplicates one already admitted, and a name that
  collides with one already admitted once both are canonicalised (Unicode
  NFC normalisation, then full Unicode case conversion — not a bare
  `to_lowercase()`, which would miss a normalisation-form mismatch on its
  own).
- **The two size bounds that matter under a lying header are enforced by
  counting bytes as they stream out of the decompressor — never by
  trusting the archive's own declared size.** `size()`/`compressed_size()`
  are attacker-controlled header fields, reconciled with reality only by a
  CRC-32 check performed after the whole entry is already decompressed;
  per-entry uncompressed size (reused outright from the existing
  whole-resource size ceiling, rather than a second independently-tuned
  number) and cumulative expanded size are both checked against the actual
  streamed byte count, never the header. Two further bounds are honestly
  labelled differently rather than folded into that same claim: entry
  count is checked against the central directory's own real record count
  (each record is a real struct plus a real filename string, not a single
  lied-about integer, so it is not attacker-inflatable the way a declared
  size is) — refusing the whole archive before any entry opens; the
  compression-ratio bound is an ADVISORY pre-filter computed from the two
  declared header fields themselves, before any byte is decompressed —
  cheap triage in front of the per-entry streamed check, which is what
  actually holds under a header that lies about the ratio too. Every
  ceiling names the specific bomb shape it defends against and is stated
  as provisional and unmeasured except the reused per-entry-size one, the
  same honesty this codebase already applies to its other unvalidated
  numeric ceilings — **including that the cumulative expanded-size ceiling
  is a single WHOLE-TREE budget, not a per-level one** (corrected here,
  Y4: an earlier draft of this bullet described a per-level budget that
  does not match the shipped code, which threads one accumulator by
  mutable reference through the whole recursive walk — see `archive.rs`'s
  own module doc for the fix and the bug it closed: a nested archive tree
  demanding the per-level ceiling raised to the depth power before
  anything tripped).
- **A prior open research question is now closed: this crate does not
  reject overlapping or self-referential (quine-shaped) archive
  constructions on its own.** Its own documentation says so verbatim for
  the one relevant diagnostic it ships, which is opt-in, not automatic. The
  adapter calls that diagnostic itself, before opening any entry, and
  refuses the whole archive rather than any single entry when it fires —
  proven against a hand-crafted fixture whose central directory legitimately
  points two entries at the identical compressed-data byte range. A second,
  narrower correction surfaced while building this wave's own fixtures:
  two central-directory records sharing byte-identical raw names do not
  both survive to be visited at all — the crate's own internal bookkeeping
  silently collapses them to one entry, last-write-wins, before the archive
  even reports how many entries it holds. Both findings are pinned by their
  own decisive tests, not merely asserted.
- **Child resources keep parent provenance in the adapter's own return
  value** — parent archive source/resource, entry path, entry content
  hash, and entry adapter identity — composed rather than restated: every
  admitted entry's cache key folds in its immediate parent's own key — a
  top-level archive's, or (for an entry that is itself a nested archive)
  that entry's own composed key — so a grandchild's key already encodes
  its whole ancestry without this crate storing an explicit chain, and
  cannot collide with an unrelated top-level resource that happens to
  share a content hash. A nested archive within the depth ceiling recurses;
  one past it still becomes its own child resource, just not opened
  further. **Only entry path reaches the worker→daemon wire today** — see
  the named-seam bullet below for the other three fields.
- No entry is ever executed, and nothing this adapter produces is written
  to a real filesystem path — expansion produces bytes and structured
  records only.
- **A named seam, not a silent one**: the worker's wire contract does not
  yet carry a child's content bytes, hash, or composed key — only its name
  and path, the shape the transport already had. Every admission, bounds
  and provenance claim above is proven exhaustively against the adapter's
  own return value; widening the shared wire type to carry a child's real
  bytes end to end is left, explicitly, to the wave that wires real
  daemon-side persistence for archive children.

**Mail (`.eml`) adoption.** `.eml` messages now normalize into message shape (A1 §6.5) — from/to/cc,
  sent timestamp (RFC3339), subject, text AND html bodies, message id, and
  thread identifiers (References plus any In-Reply-To id not already
  present) — running the `mail-parser` crate (0.11.8, `full_encoding`
  feature) behind a pure-function adapter inside the same supervised worker
  transport the Office and ZIP adapters use. Adopted after the same
  candidate-spike gate order those two used: deny gate first (zero new
  advisory/license/ban/source failure, diffed against the pre-existing
  yanked-`chacha20` baseline this crate does not own), a hand-verified
  six-fixture corpus with every count cross-checked independently against
  Python's stdlib `email` package before `mail-parser` ever ran, then
  footprint (two new packages, well under noise once linker
  nondeterminism is corrected out of the linked-binary measurement).
- **Two gaps the spike found in `mail-parser`'s own lenient behavior are
  closed, not merely documented.** First: `mail-parser` synthesizes a
  missing text OR html body from the message's one real part by aliasing
  the identical internal part index into both its text and HTML body lists
  rather than fabricating separate content — this build inspects the
  aliased part's own physical type to tell which side is genuine and
  reports the OTHER side absent, rather than one the wire bytes never
  declared. (An earlier version of this fix only checked which build
  applied to a plain-text-only message and got a genuinely HTML-only
  message backwards — caught in review and corrected before release; both
  directions now have their own regression fixture.)
  Second: a message that is MIME-shaped but structurally broken (an
  unterminated multipart boundary) does not fail this crate's own parse at
  all — the body part that should have been read normally instead
  reappears as an attachment with no recoverable name. Any such nameless
  attachment is treated as evidence of exactly this silent downgrade and
  refuses the WHOLE message, coverage-honest rather than a partial or
  wrong-shaped success — the one narrow, stated exception being a
  `message/rfc822` attachment, which legitimately carries no filename far
  more often than an ordinary attachment does.
- **A structurally encrypted or S/MIME-sealed message gets its own honest
  refused status**, never a body of decoded-looking garbage: `mail-parser`
  has no PKCS#7/S-MIME awareness of its own, so this adapter detects
  `multipart/encrypted` and `application/pkcs7-mime` from the message's own
  declared Content-Type before treating it as ordinarily readable — never
  by attempting to decrypt or verify anything.
- **Attachments recurse through the same container machinery the ZIP wave
  built**, with a parent-message coordinate: a `message/rfc822` attachment
  is a nested message, parsed directly from `mail-parser`'s own
  already-parsed embedded value rather than re-serialized to raw bytes and
  parsed a second time (an early draft did the latter and picked up a
  boundary-adjacent line ending the second parse mis-attributed to body
  content — caught and corrected, including a matching correction to the
  pre-existing test fixture's own hand-verified answer, before it shipped).
  An attachment that is itself a ZIP archive chains through the identical
  ZIP-recursion function the ZIP wave uses on itself, sharing one nesting-
  depth ceiling and one whole-tree cumulative-bytes budget across mail and
  archive nesting alike, never two independently sized ones. **The reverse
  direction chains too**: a ZIP entry named `.eml` recurses back into this
  same mail adapter (`archive::classify` now routes `.eml`, closing a gap a
  review finding caught — a `.eml` entry inside a ZIP previously fell
  through to `unsupported` instead), through the same shared depth/budget —
  so a mail-inside-a-zip-inside-a-mail chain is bounded by one ceiling and
  one budget across the whole tree, whichever container kind each level
  happens to be. The same admission discipline the ZIP wave established
  applies unchanged:
  empty-name refusal, path safety, name uniqueness, and the identical
  Unicode NFC-then-case-fold collision rule, all reused outright rather
  than rebuilt.
- **Every size bound is reused outright from the ZIP wave's own ceilings**
  rather than three new, independently-tuned numbers — all still
  PROVISIONAL, same footing as this program's other unmeasured ceilings.
  **Named honestly, not glossed over**: unlike the ZIP adapter's streamed
  read, `mail-parser` decodes every message part into memory before this
  adapter's own bounds ever run, so these are admission checks over what
  was already decoded, not a guarantee against the decode itself
  allocating — the supervised worker's own per-child address-space limit
  is the real backstop for that, exactly as it already is for a worker
  that never returns a batch at all. Considered separately: ordinary mail
  transfer encodings expand by at most a small, fixed ratio, nowhere near
  a compressed container's achievable worst case, so this gap is
  materially smaller than the equivalent would be for ZIP.
- Register row 8 (`.eml`/mail message evidence) moves from deferred to met,
  citing the real subprocess acceptance proof the same way rows 5 and 7
  already do.

**External Git, package identity, and the scan trigger.** Atlas stops being writers-and-readers with nothing between them: a scan
trigger, and a second source kind (`external_git`) alongside
`estate_git`/`local_knowledge`.

- **`sgt knowledge scan` (`POST /v1/intelligence/scan`, G8)** drives a full
  scan of an estate's declared `[[knowledge]]` sources through the daemon,
  on the intelligence lane, with each source's own walk queuing on the
  lane's existing permit — bounded concurrency, not a second cap invented
  here. Reports what it indexed and what it could not **from each source's
  own coverage counts**, never a guess. Scheduling and cadence stay
  deliberately unbuilt (G10): a recurring trigger is later work's, when
  retrieval needs one. This supersedes two settled records, both edited in
  this same PR as in-scope work rather than left to drift: the acceptance
  register's tripwire asserting no production caller of the scan pipeline
  existed (it was built to fail on exactly this day — see the register
  edit below), and `docs/concepts/atlas-and-knowledge.md`'s "no way to
  start a scan" sentence.
- **`sgt intelligence add <url> [--ref <ref>] [--name <name>]` / `list`**
  (`POST`/`GET /v1/intelligence/sources`, G6, A1 §9, item 10) — external
  Git acquisition:
  - **Locator allowlist BEFORE Git ever sees the string** (A1-24), the
    primary control, researched against Git's own documentation rather
    than recalled: exactly `https://…` and `ssh://…`/`user@host:path`
    (scp-like, `user@` required) are accepted; `ext::`/any other
    `<transport>::` remote-helper form, `file://`, a bare local path,
    `git://`, and `ftp[s]://` are all refused by name. An embedded
    credential (`user:pass@…`) is refused outright — the operator's
    ambient git/ssh credential helper is the only accepted auth path.
    Second, independent control: every acquisition subprocess sets
    `GIT_ALLOW_PROTOCOL=https:ssh` — narrower than `sgt repo add`'s own
    default — which Git's own documentation states overrides even a
    `~/.gitconfig` `insteadOf` rewrite, closing that class of bypass even
    if the string-level allowlist ever had a bug.
  - **Bare, no-working-tree host cache** outside every estate
    (`<data-dir>/atlas/external-git/<name>`), one per declared source,
    reused across refreshes. Exact-commit resolution: the requested ref
    (default the remote's own `HEAD`) is fetched shallow (`--depth 1`)
    into a fixed local ref and resolved to a full commit SHA, which is
    what every downstream row keys on.
  - **Reads through the estate-git plumbing verbatim (R2)** — the fetched
    bare repository is listed and extracted through the identical
    `list_tree`/`extract_blobs`/`directory_coverage` an admitted `[[repo]]`
    mount already uses; the only new code is getting bytes into the cache
    and stamping the result `external_git`/`external` instead of
    `estate_git`/`estate_mutable`.
  - **Provenance** (origin, requested ref, resolved commit, retrieved at,
    `authority_class=external`) lands in a new table, `git.provenance` —
    the first writer into the `git.*` namespace X1 reserved and left
    empty — never a column bolted onto `source.generations`, which is
    "only ever added to, never altered" by this store's own standing rule.
    Written atomically inside the same staging transaction as every other
    row a generation gets.
  - **External content is DATA, never instructions (A1-25), proven, not
    merely asserted**: a fetched repository's `AGENTS.md` is claimed by
    the ordinary Markdown extractor and lands as `Document`/`Section`
    units — the identical treatment any other `.md` file gets — and
    nothing here executes anything a fetched byte says. A bare repository
    has no working tree to check anything out into, so there is no
    hook-triggering operation for the fetched repository's own configured
    hooks to ride, and reads go through `cat-file --batch`, never
    `--filters` (confirmed against Git's own documentation: `--filters` is
    the one `cat-file` mode that invokes a clean/smudge filter driver, and
    this build never passes it).
  - **The git subprocess runs supervised**, the same #310 discipline a
    parse worker gets (own process group, `PR_SET_PDEATHSIG`, killed and
    reaped past a deadline) — S4 Y5's own reading of G2's amendment: a
    remote is attacker-influenced input. No address-space cap: `git` is
    the trusted, memory-safe binary this codebase already shells out to
    everywhere, not a generated grammar parsing untrusted bytes in-process
    — that risk class is what the parse-worker memory cap exists for, and
    it does not apply here. Fetch deadline: 120s, PROVISIONAL (#325's
    precedent), chosen generous rather than measured against a corpus that
    does not exist yet.
  - **Refresh = a new `SourceGeneration`**, ruling §4's ordinary rule,
    unmodified for this source kind: an unchanged tree writes and evicts
    nothing, a changed one evicts the superseded generation (its
    `git.provenance` row goes with it). "Pinned Works keep theirs" (G6) is
    honest by construction rather than by new machinery: nothing in this
    build binds a Work to a source generation yet — Atlas is still
    read-only evidence with no consumer (S5's retrieval work is what would
    actually create that binding), so there is nothing a refresh could
    pull out from under one.
- **PURL-shaped package identity (A1-26, §10)**: `Cargo.lock` parsed into
  `(name, version, source)` per locked package, with a purl
  (`pkg:cargo/<name>@<version>`) derived for a plain registry dependency —
  verified against the `package-url` project's own reference examples,
  not recalled — and named `None`, honestly, for a git-sourced or path
  dependency rather than guessing a purl shape the specification does not
  cleanly define for either. Ships as a tested, pure derivation
  (`domain::package`); no CLI verb or table wires it yet, because no
  register item or consumer commissions one this wave and an unused
  surface is the same false promise an empty table is (R1) — the
  derivation itself is what §10 actually asks A1 to be able to do. **S4 Y7
  closeout correction**: this was itself an unpinned instance of the
  sprint's own signature defect (built, tested, zero production caller —
  no `git.package_dependencies` table, no CLI verb, no API route ever
  calls `domain::package`) and had no tripwire holding it to that fact,
  unlike row 2's Work-overlay gap. A tripwire now does
  (`domain::package::the_derivation_has_no_production_caller_yet`,
  `src/domain/package.rs`); the destination is unbuilt scope, named rather
  than guessed at — whichever wave first commissions a `git.*` consumer
  for lockfile-derived package identity.
- **G2's revisit trigger, answered rather than left to pass silently.**
  G2 kept the tree-sitter syntax extractor in-process on the stated
  predicate "inputs are local/estate-owned rather than attacker-chosen",
  with a revisit trigger naming the first external-git source feeding it
  remote bytes as one of two conditions. That condition is this wave.
  **Decided: stays in-process.** Not because the predicate still holds —
  it does not — but because the bytes that reach it are already
  size-bounded, UTF-8-checked text on the identical code path
  `estate_git` bytes have run through with a clean crash record since X3b
  (external-git reuses `extract_blobs` verbatim rather than adding a new
  path), tree-sitter's own design goal is robustness to adversarial input
  (not merely well-formed input, a stronger posture than the general
  parser class §12 actually targets), and the acquisition half already
  runs supervised. Moving it worker-side was considered and set aside for
  this wave specifically — it would mean splitting one shared extraction
  call into two execution shapes keyed on source kind, a real change to
  code every source kind currently relies on, not a safe rider on an
  already-large wave. The trigger's other leg — the first syntax-lane
  crash — stays armed, now with genuinely attacker-influenced bytes
  reaching it for the first time. Full reasoning:
  `src/runtime/atlas/syntax.rs`'s own module doc, "G2's revisit trigger,
  answered".
- **Doctrine amendment, ADR filed (ADR 0023, workspace-native).**
  `AGENTS.md`'s "`sgt` never fetches" sentence is scoped to admission by its
  own words, but external-Git acquisition now genuinely does fetch,
  deliberately, in a different subsystem with no Work authority — never
  touching a repository mount, a Work surface, or admission's own
  preflight. `AGENTS.md`'s CAN section cites "ADR 0023" in place for the
  scoped meaning, unweakened for admission itself; a red-then-green test
  (`tests/y5_doctrine_never_fetches_is_scoped.rs`) pins both the doctrine
  text and the structural boundary (admission/materialization never
  references the external-git fetch surface). An earlier version of this
  entry claimed the changelog paragraph itself was the ADR's binding
  statement, then a later revision correctly retracted that (a changelog
  paragraph is not a filed, numbered, Status/Context/Decisions record) but
  treated the gap as an unresolved J0 conflict awaiting a fresh owner
  ruling. It was not: the conflict was already settled five days earlier by
  the split-hardening sprint's own Amendment 9
  (`knowledge/evidence/resources/split-hardening-series/sprint-
  plan-2026-08-24.md`), which pre-names this exact number and rules that
  "W1's ADR 0023/0024/0025 are **not authored as product ADRs** — binding
  text homes in `AGENTS.md`... and workspace rulings." Per that settled
  record (J3), the ADR is filed directly in the workspace —
  `knowledge/rulings/adr/0023-external-git-acquisition-scopes-never-
  fetches.md` — with no corresponding `sergeant-rs` file, and this
  changelog entry, `AGENTS.md`'s citation, and the module docs citing "ADR
  0023" now resolve to that record.
- **Register.** Row 10 (external Git) moves from `deferred-s4` to `met`,
  citing `src/runtime/atlas/external_git.rs`'s own acquisition-mechanics
  tests and `tests/y5_external_git_triggers.rs`'s HTTP-surface proof. Row
  9 (OCR)'s citation is corrected: owner ruling 3 and the ratified S4 re-cut
  place OCR **after** S4, not inside it — the register's earlier "S4's,
  same citation as item 7" was a mis-citation, now `deferred-post-s4` with
  the ruling cited. Both are documentation corrections to where an item
  already lived, not scope changes.

**The scan trigger becomes estate-scoped, and online-only detection ships.** The owner correction **"estate intelligence is the feature. Not index a
knowledge repo"**
(`knowledge/rulings/owner-rulings/estate-intelligence-is-the-feature-2026-08-28.md`
in the workspace estate) named a gap S4 Y5's own scan trigger left open: it
iterated `[[knowledge]]` sources only, even though S3's X3a had already
shipped the entire `estate_git` extraction path
(`scan_estate_git`/`scan_estate_git_on_lane`) with no production caller. A
copy of `sergeant-rs` had to be declared as a knowledge source to get any
code intelligence over it at all — losing blob-OID keys, the pinned SHA,
Work overlays and drift observation, every property X3a exists to provide.
This closes that, as G8's own completion rather than new scope, and ships
the wave's other named item alongside it.

- **`sgt intelligence scan` — the estate-scoped trigger.** `POST
  /v1/intelligence/scan` now scans everything the addressed estate
  declares, across all three A1 §2 source kinds, in one call:
  - **`estate_git`** — every declared `[[repo]]` repository, through the
    identical Git path (`scan_estate_git_on_lane`) an admitted Work
    already reads through: `git rev-parse HEAD` at the mount for the
    pinned SHA (plain plumbing, run off the intelligence lane — extraction
    itself still queues on the lane's own permit), then `ls-tree`/
    `cat-file --batch` at that exact commit. **Repository bytes never go
    through the folder walker** — routing them there would lose blob-OID
    content keys, the pinned SHA, Work-overlay hashing and drift
    observation, which is a bug this correction closes rather than an
    accepted alternative. A mount whose HEAD moved since the scan pinned
    it is reported as a drift observation riding beside its row, the same
    §11.4 vocabulary Work retirement already uses.
  - **`local_knowledge`** — every declared `[[knowledge]]` source, through
    the folder walker (S4 Y5, unchanged).
  - **`external_git`** — every external-Git source already recorded on
    this host, refreshed through Y5's own acquisition. Host-scoped, not
    estate-scoped (Atlas has no per-estate association for these yet, so
    an estate-scoped scan refreshes every one the host holds) — named
    here rather than silently assumed; a finer per-estate binding is
    unbuilt scope.

  Each row in the report now also names its `kind`, so per-source coverage
  is honestly distinguishable across all three — the ruling's own "one
  world model over one estate, reporting per-source coverage across all
  kinds" requirement.

  **Naming.** `sgt intelligence scan` is the primary spelling — an
  intelligence/estate verb belongs beside `sgt intelligence status`/
  `add`/`list` and `sgt map`, argued in this wave's own commit body, not
  `sgt knowledge scan`'s knowledge-folder-only vocabulary describing a
  now-estate-wide capability. **`sgt knowledge scan` keeps working,
  unchanged spelling**, and now runs the identical widened scan — nothing
  that already used it stops working or starts doing less.

  **Proven end to end**, not merely at the pure-function level
  (`tests/y6a_estate_scoped_scan.rs`): a real `[[repo]]` mount holding this
  build's own `src/` tree, scanned through the real daemon trigger, with
  `sgt map symbol`/`map references` resolving a real function
  (`scan_estate_git`) at its real path — and the reported `content_key`
  asserted **exactly equal** to `git rev-parse <pinned-sha>^{tree}`
  computed independently on the mount, proving the generation is keyed on
  the pinned commit's tree OID (X3a's own identity) rather than any
  content hash a working-tree walk could have produced.

- **Online-only / cloud-sync placeholder detection (G7, A1-06)** — the
  wave's other named item, item 4's own open GAP since S3 close. A
  synced-but-not-materialized file (a OneDrive/Dropbox/iCloud "online-only"
  placeholder) now gets its own named coverage state, `online_only`,
  **never** `indexed` with the file counted as though its bytes had
  actually been read — the exact "silently indexed as empty" shape item 4
  forbids.
  - **Signal**: `st_blocks == 0` with `st_size > 0` — verified against the
    Rust standard library's own documentation for
    `std::os::unix::fs::MetadataExt::blocks` (a file with holes reports
    fewer blocks than its size implies), not assumed. Read from the
    `symlink_metadata` call the walker already makes for every entry —
    **no additional syscall**.
  - **Permitted syscall set, settled and checkable**: exactly `lstat`/
    `stat`. `open()`/`read()` are never attempted on a path this check
    flags — the classification runs strictly before the byte-read
    boundary in both the document and dataset paths
    (`src/runtime/atlas/scan.rs`'s `Walk::file`/`Walk::dataset`), because
    A1 §7 forbids auto-hydrating a library and `open()` is the documented
    hydration trigger on several cloud filesystems.
    `listxattr`/`getxattr` were investigated and **not adopted**: no
    single verifiable-via-documentation xattr convention for a cloud
    placeholder covers this build's Linux/macOS targets, and guessing one
    would repeat the `enclosed_name` mistake already made once this
    sprint.
  - **Honest about being a heuristic, in the coverage row's own `detail`
    text every time, not only in a doc comment**: a legitimate sparse file
    (a disk image, a punched-out log) reads identically and is a false
    positive; a placeholder a sync client reports with full block
    allocation before the byte is fetched is not caught at all and is a
    false negative. A genuinely empty file (`st_size == 0`) is never
    flagged — flagging it would be the opposite dishonesty.
  - **Proven end to end through the real trigger**
    (`tests/y6b_online_only.rs`): a sparse stand-in file (`truncate`'s own
    documented effect on ext4, verified in this wave's own sandbox rather
    than assumed) scanned via `POST /v1/intelligence/scan` lands
    `online_only` in the real, daemon-recorded coverage — stated plainly
    as a stand-in for a true cloud placeholder, not a claim that this
    proves detection against a real sync client.
  - **Register row 4 updated, not deleted**: `gap` -> `met`, citing the
    new end-to-end and unit tests; the negative tripwire that used to pin
    the gap (`a1a_item_4_gap_cloud_placeholder_detection_is_not_shipped`)
    is retired per its own assertion message's instruction and replaced
    with a positive vocabulary pin
    (`a1a_item_4_the_coverage_vocabulary_now_names_online_only`), the same
    pattern the cross-cutting-gap tripwire already set at S4 Y5.

**Review-panel fixes.** Three findings from this wave's own review, fixed against the commit above
rather than folded silently into it:

- **Register row 2 corrected: honest about the Work-overlay half.** The
  estate-scoped trigger above wires a real production caller for the
  `estate_git` **base-repo** half of item 2's claim
  (`scan_estate_git_on_lane`, proven in `tests/y6a_estate_scoped_scan.rs`) —
  but the sibling **Work-overlay** half (`scan_work_overlay`,
  `scan_work_overlay_on_lane`, `scan_and_record_overlay`) still has zero
  production callers: no route, no verb, no Work-lifecycle hook, only test
  callers. Row 2's verdict moves from `met` to `met-with-deviation`, its
  note says so plainly, and a new tripwire
  (`a1a_item_2_gap_work_overlay_scan_has_no_production_trigger`) fails the
  day a production caller lands, so this gap cannot be reported closed by
  accident the way the base-repo half almost was. Wiring an actual trigger
  (a Work-lifecycle hook, or an explicit scan parameter) is CLI/API design
  work this wave's own brief reserves for argument in review — not decided
  unilaterally here (A1 §14).
- **Cross-kind source-name collision refused at manifest-parse time.**
  `source.generations` keys a confirmed generation by `source_name` alone,
  with no column for which of the three source kinds produced it — so a
  `[[repo]]` and a `[[knowledge]]` entry sharing a declared name silently
  contended for one generation lineage, each scan of one kind evicting the
  other's evidence. `resolve_knowledge` now refuses a knowledge entry named
  after an already-declared repository
  (`EstateError::KnowledgeNameCollidesWithRepository`), the cross-kind twin
  of the same-kind `DuplicateRepository`/`DuplicateKnowledge` refusals that
  already existed — closed before either scan can run, not detected after.
- **`sgt doctor`'s atlas row surfaces `online_only` (and every other
  coverage state).** The row's tally used to hand-pick five coverage
  statuses; a source whose rows were entirely `online_only` (an estate
  synced from a cloud client that has hydrated nothing) tallied zero in
  every counted bucket and printed `ok` — indistinguishable from "nothing
  to report", the exact silent-gap failure this row exists to prevent. The
  tally now iterates `Coverage::ALL` (so a status added later cannot repeat
  this omission by being forgotten here), the detail string names every
  one including `online_only` and `generation_evicted`, and a nonzero
  `online_only`/`error`/`unavailable` count now warns
  (`generation_evicted` stays reported-but-not-warned: an eviction is
  ordinary lifecycle, not a fault).

**S4 Y7 closeout — the contract walk, not the wave ledger.** This sprint
shipped its own signature defect twice before this closeout (Y5's
`estate_git` scan built with no production caller, closed by Y6; Y6's own
review then caught Work-overlay scanning in the identical shape). The
closeout's own brief: don't check work against its own brief again — walk
`A1-ATLAS-WORLD-INTELLIGENCE.md` directly and deliberately search for a
third instance. There was one.

- **Package identity (§10, A1-26) was the third instance, unpinned.**
  `domain::package`'s derivation — tested at the unit level since S4 Y5 —
  has zero production callers: no `git.package_dependencies` table, no CLI
  verb, no API route. This CHANGELOG already said so honestly at Y5; what
  it lacked was a tripwire holding it to that fact the way row 2's
  Work-overlay gap has one. `domain::package::the_derivation_has_no_production_caller_yet`
  is that tripwire now, watched red by hand (a temporary reference from
  `src/api.rs`, reverted) before landing.
- **"A worker never opens the store" was asserted only in a comment.**
  `src/bin/atlas_worker.rs`'s own module doc claimed this structurally; no
  test checked it. `runtime::atlas::worker::tests::a_worker_never_names_the_atlas_store`
  now does, watched red against both the worker binary and its own module
  before landing, the same discipline the one-owner `duckdb` tests already
  set.
- **`docs/concepts/atlas-and-knowledge.md` silently implied Work-overlay
  indexing was live.** It described the overlay in the same present tense
  as the three source kinds `sgt intelligence scan` actually drives,
  without ever saying it has no trigger — the omission the brief asked
  this closeout to find and correct (four other doc-versus-code overclaims
  were caught in this sprint's earlier reviews; this is the fifth). Fixed:
  the doc now says plainly that overlay indexing is built, correct, and
  untriggered, with a new decision row (D14) pinning it.
- **Boundary audit.** One-owner-per-database (both `x1_atlas_substrate`'s
  and `m5_projections`'s tests) and no-client-SQL
  (`a1a_item_13_no_client_sql_reaches_the_store`) were each personally
  watched red, by hand, this closeout — not merely re-run green. The
  anydoc replaceability boundary's cargo-metadata alias check
  (`tests/y2_office_boundary.rs`) and the AGENTS.md "never fetches" scoping
  (`tests/y5_doctrine_never_fetches_is_scoped.rs`) already carry their own
  documented red-then-green history from the waves that built them. The
  shared archive/mail depth-and-byte budget is proven by a real nested
  fixture (`archive.rs`'s own `a_zip_entry_named_eml_past_the_depth_ceiling_is_admitted_but_not_parsed`)
  rather than merely asserted, though this closeout did not itself revert
  it to watch it fail.

**S4 Y8 — dispatch, not just adapters.** The Y7 closeout's own sweep
reported finding exactly one remaining unreachable-code instance (package
identity) and called the sweep complete. That claim was false: the largest
instance of the sprint's own signature defect was still standing. `sgt
intelligence scan` walked a folder, saw a `.docx`, and did not extract it —
`scan.rs`'s routing table (`claims_for`) never claimed `.docx`/`.zip`/`.eml`,
so the resource fell straight through to `unsupported`, and neither
`scan.rs` nor `record.rs` ever called `run_worker_on_lane` or `worker::`
anything. The three content adapters and the worker transport that carries
them were dead code in every real installation.

- **The scan walk now dispatches.** `scan.rs`'s `Walk::file` and `git.rs`'s
  `extract_blobs` both route a resource their own new `worker_extractor_for`
  table claims through the real `run_worker` transport and the daemon-side
  `validate_batch` AUTHORITY — the identical path Y1 designed and Y2/Y3/Y4
  built the three adapters to run inside, exercised for the first time from
  a real walk rather than only from each adapter's own test suite. The
  in-process text/Markdown/syntax/tabular paths are untouched: pure Rust
  over local bytes was never the worker's business, and stays that way.
  `scan_local_knowledge_on_lane`/`scan_estate_git_on_lane` (the production
  entry points) resolve a real `WorkerRuntime` and thread it down; every
  existing caller of the plain, worker-free
  `scan_local_knowledge`/`scan_estate_git` keeps working exactly as before,
  unchanged.
  **Fix-agent correction:** the originally landed `WorkerRuntime` resolution
  returned this host's own binary path (`std::env::current_exe()`)
  unchanged — the *daemon's* own binary, which `sgt`'s own subcommand-only
  CLI cannot be re-exec'd as a worker with (`--generation`/`--extractor`
  fails to parse), so every real, non-test installation would have hit a
  clap error on the very first claimed `.docx`/`.zip`/`.eml`, never an
  extraction. `tests/y8_adapter_dispatch.rs`'s original acceptance test
  stayed green throughout because it drives `scan_local_knowledge_with_worker`
  directly with a hand-built `WorkerRuntime`, never the real
  `scan_local_knowledge_on_lane` entry point or its resolution. Fixed to
  resolve the real sibling `sgt-atlas-worker` binary Cargo builds beside
  `sgt` (same `target/<profile>/` / install directory), and proven by a
  new `scan_local_knowledge_on_lane_resolves_and_dispatches_the_real_worker_binary`
  test that plants the real worker binary beside the test binary's own
  `current_exe()` and drives the actual lane entry point end to end —
  confirmed to fail red (a clap "Unrecognized option: 'generation'" error)
  against the original, unfixed resolution before this correction landed.
- **Proven against real bytes, not a synthetic fixture.**
  `tests/y8_adapter_dispatch.rs` scans a directory holding one real
  `.docx`, one real `.zip` and one real `.eml` (this repo's own
  hand-verified corpora) through the worker-enabled scan/dispatch logic
  (`scan_local_knowledge_with_worker`), then records the result through
  `record_scan`'s real three-step discipline, and asserts the RECORDED
  generation's own extractor set names all three adapter identities and
  that document units carry real, non-empty text — exactly the shape an
  isolated adapter unit test cannot prove, which is what let this defect
  stand through four prior waves' own acceptance batteries.
  `tests/x3a_git_plumbing.rs` carries the estate-git-side twin. A second
  test, added by the fix-agent panel pass below, additionally drives the
  real production entry point (`scan_local_knowledge_on_lane`) end to end
  so the worker-binary RESOLUTION this dispatch depends on is exercised
  too, not only the pure dispatch logic.
- **Children, honestly.** A worker-declared child (an archive entry, a mail
  attachment) is validated daemon-side by the same `validate_batch`
  AUTHORITY (path safety, F10 deny-set membership) and its name lands,
  visibly, in its parent's own coverage detail — but its CONTENT does not
  yet reach the store as its own `source.files` row. `WorkerBatch`'s
  `DeclaredChild` still carries only a name and a path on the wire, exactly
  the "named seam" `archive.rs`'s own module doc has flagged since Y3;
  widening that shared wire type to carry a child's bytes, hash, or
  composed key is a security/footprint decision this wave's brief does not
  settle and this wave does not decide unilaterally. Register rows 7 and 8
  (`tests/x5_a1a_acceptance.rs`) are corrected to `met-with-deviation`
  naming exactly this gap and its destination, rather than continuing to
  read `met` on the strength of the adapter alone.
- **The register's own stale claim, corrected.** Item 13's note still read
  "nothing shipped triggers a scan" — false since S4 Y5/Y6 shipped `sgt
  intelligence scan`, independently of this wave's own dispatch fix. Fixed
  to state the actual, current gap: the surfaces answer empty only until an
  operator runs the shipped trigger, not because none exists.
- **Fix-agent panel pass.** `run_worker_on_lane` — the permit-acquiring
  wrapper this same wave's `lane.rs` heavily edits — turned out to be a
  FOURTH orphaned production path: the dispatch this wave wired calls
  `run_worker` directly from `dispatch_worker_resource`, already inside the
  whole-scan permit `scan_local_knowledge_on_lane`/`scan_estate_git_on_lane`
  hold, never through `run_worker_on_lane` itself, which remains reachable
  only from `tests/y1_worker_transport.rs` and its Y2-Y4 siblings. Its own
  doc comment now says so explicitly, and a recursive `src/`-wide tripwire
  (`lane::tests::run_worker_on_lane_has_no_production_caller_yet`, the same
  shape as `domain::package`'s) pins it — matching this wave's own
  package-identity precedent rather than leaving a fifth silent instance.
  Two doc comments (`office::extractor_for`, `worker::run_worker`) that
  described this wave's own dispatch as running through
  `run_worker_on_lane` are corrected to name the real path
  (`dispatch_worker_resource` -> `run_worker`, directly). The
  `WorkerRuntime` resolution bug this same pass found and fixed is its own
  entry above ("The scan walk now dispatches", fix-agent correction).

### Atlas closeout (S3)

- `docs/concepts/atlas-and-knowledge.md` documents Atlas for operators: what
  persists versus what refolds from the journal, how sources and generations
  are identified, the coverage vocabulary and why an excluded byte is counted
  rather than absent, what structural extraction does and does not claim, and
  the bounded map surface.
- An acceptance battery (`tests/x5_a1a_acceptance.rs`) walks all fourteen
  contract acceptance items and holds itself to them: every verdict names a
  decisive check, every citation is verified to still exist in the suite it
  names, the documented walk table is checked against the executable register,
  and the items belonging to later work are recorded as such rather than
  quietly passed.
- **Two limits the battery names rather than glosses.** Atlas ships its
  writers, its readers and no trigger between them: no `sgt` verb, no route
  and no daemon job starts a scan, so on a fresh installation the store is
  empty and `sgt doctor`'s atlas row says exactly that. And a file a sync
  client has listed but not materialized, which the filesystem presents as a
  readable empty file, is indexed as the empty file it appears to be —
  detecting that case is best-effort heuristics and is not shipped. Both are
  pinned by tests that fail when the situation changes, so neither can close
  or widen unnoticed.
- Combined dependency footprint, measured once at the sprint boundary rather
  than inferred by adding the two per-wave deltas: against the baseline before
  the two heavy dependency additions — the integration head after X3a, the same
  commit the X3b evidence measured, which already carries X1–X3a's own code —
  `Cargo.lock` gains 10 packages, a cold `cargo build --tests` gains
  ~15 s (+10%), dev `target/` gains 1.41 GB (+9.7%), the dev binary gains 12.5
  MiB (+5.0%) and — measured for the first time this sprint — the **release**
  binary gains 4.66 MiB (+6.9%), 67.8 MiB to 72.4 MiB. The grammar objects are
  not in those binary numbers: nothing reachable from `sgt`'s own entry point
  references a grammar yet, so the linker drops them. Forcing a reference and
  reverting it (the binary returns byte-for-byte) measures what they will cost
  when something does: a further 5.27 MiB, for a dev-binary total of 17.7 MiB.

### Test contract (S5)

- **The submit-throughput guard asserts a ratio, not a rate (#278).**
  `t12_submission_throughput_has_an_automated_floor` asserted an absolute
  floor of 8.0 works/s, which is a bound about the host it was tuned on: on
  one unchanged commit it measured 6.5 works/s and failed (GH run
  33250519400) and passed forty minutes later (run 33251738966) on runner
  load alone — and 6.5 was below the 10.2 works/s regression-path simulation
  its own derivation recorded, so where it ran it could no longer tell a
  healthy path from a regressed one. This is a contract-shaped change: the
  guard now asserts the **per-submission cost of the guarded section in units
  of one `git worktree add` measured on the same host in the same run**
  (ceiling 12.0 units, best of three attempts) — healthy 2.4–8.8 units across
  idle / CPU-hogged / fsync-hogged / 2-core / 2-core-plus-hogs conditions,
  17.5–64.9 with 86 ms serialized under `runtime::surface::with_repository`
  (N3R2-04's own number), so a slower target moves the numerator and the unit
  together instead of failing the build. The #128 macOS special case is
  retired with it: a ratio in git-spawn units does not need a per-platform
  constant. The burst is still driven to a terminal state and asserted
  `completed`, which is what keeps the measurement on the whole submit path
  rather than the HTTP surface. `scripts/coverage/README.md` gains the general
  rule an asserted performance bound is judged against (headroom on the
  slowest supported target — `docs/reference/glossary-and-support.md:18` — or
  convert it to a ratio, or delete it) and the class table it belongs to.
  (R2/R7, J5: the coverage README's own risk-class rule; the serialized-burst
  control the wave brief proposed was implemented, measured — healthy speedup
  0.48×–3.78× vs 1.29× regressed — and rejected as indistinguishable from the
  regression on two cores.)

### One Atlas database (S5)

- **`sergeant.duckdb` is gone.** A1 §5 declares one physical file,
  `atlas.duckdb`, carrying five logical schemas — `meta`, `ops`, `source`,
  `git`, `context` — and decision A1-02's stated rationale is "schemas provide
  separation without more databases". S3 shipped two files and recorded that
  in the A1a register as a *ratified* deviation; no owner ruling ever ratified
  it. The owner correction of 2026-08-29 settled that the code converges to
  the contract, so the operations projection's `ops.*` tables now live in
  `atlas.duckdb`, and `sergeant.duckdb`, `DUCKDB_FILE` and `duckdb_path` are
  deleted. Register item 1 is `met`, and the word "ratified" is struck from
  its note.
- **The rebuild discipline survived the merge as scope, not as separate
  files.** `ops` is still a pure fold of the journal and is still rebuilt on
  every daemon start — by `DROP SCHEMA ops CASCADE` + recreate + refold, which
  has exactly the reach the old `remove_file` had. `meta`, `source`, `git` and
  `context` persist, untouched by a refold.
- **What deleting the file costs changed, and is now stated wherever it was
  promised.** Deleting the projection file used to be lossless. Deleting
  `atlas.duckdb` still rebuilds every `ops` row from the journal — and
  discards every persisted source generation, which must be re-scanned.
  `docs/concepts/atlas-and-knowledge.md`, both module docs and `sgt doctor`'s
  projection remedy say so; the remedy now names a *restart* as the supported
  way to rebuild the operations tables and warns against deleting the file to
  force one. Both halves of the new sentence are measured, not just written
  (`w1c_one_atlas_database::deleting_atlas_duckdb_rebuilds_ops_and_loses_source_facts`).
- **The cross-schema join A1 §5 cites as its reason for one database is
  proven.** A single statement joins a Work's identity in `ops.work` to its
  overlay generations in `source.generations` — no `ATTACH`, no second
  database name — and a Work without an overlay is not returned
  (`w1c_one_atlas_database::one_statement_joins_ops_work_identity_to_source_generations`).
  This is the capability A2 §2's `--work` filter is built on.
- **The two one-owner structural tests collapsed into one, because their
  premise did.** `m5_projections::t2_the_duckdb_file_has_exactly_one_owner`
  and `x1_atlas_substrate::atlas_database_has_exactly_one_owner` existed as a
  pair only because there were two databases, each with one owning file, and
  both suites forbade merging them into a union rule. With one database there
  is one owner: `x1`'s test now scans the whole of `src/` with no exempted
  tree, and `src/runtime/atlas/db.rs` is the only file in the crate that may
  name the database driver. A note where `t2` stood records why it was
  removed.
- **One file is one DuckDB instance.** Two `Connection::open` calls against
  the same path produce two independent instances that neither see each
  other's writes nor error — the last close silently wins. Harmless while
  `ops` had a file to itself; not harmless now. The daemon and the API derive
  their Atlas handle from the open projection (`Analytics::atlas`), and
  `AtlasDb::open` documents that it is for a process holding no other handle.
- **"This host has indexed nothing" is now read off the evidence, not off the
  file.** `/v1/intelligence/*` and the `map` reads answered
  `atlas.present: false` when the Atlas file did not exist; the file now
  exists on every host from the daemon's first start, because `ops` lives in
  it. The same answer, with the same wording, is now given when no source has
  a confirmed generation — which is the question its `detail` string was
  always answering. A Work on an estate that indexes nothing still gains no
  Atlas evidence and no repository walk
  (`w1b_overlay_lifecycle_trigger::a_work_on_an_unindexed_estate_gains_no_atlas_evidence`).
- `floor-state.json` did not move: `projections/` still holds the FloorState
  startup cache, and deleting that directory still loses nothing. Only the
  database left it.


## [0.2.4] - 2026-08-25

sergeant-rs narrows to a product-documents-only repo, codex actors gain
commit rights, capability reporting stops overclaiming, dirty patches
recover byte-exact, and distro trees become atomic and self-checking.

### Repo scope

- `sergeant-rs` is now product-documents-only: `docs/` and
  `NORTH-STAR`/`GAUNTLET`/`LESSONS` relocated to the development
  workspace, with their supporting assets re-homed alongside them. This
  repo no longer carries the development-process record it used to.

### Codex commit grants

- Codex actors can commit — linked-worktree git grants are wired on both
  transports, including resume turns, with preflight failing closed
  rather than silently granting nothing. Live-proven end to end, not
  just unit-tested. (#240)

### Honest capability surface

- `permission_mode` is now reported per-backend rather than assumed
  uniform across adapters, and per-profile sandbox `network_access` is
  validated at load and surfaced in `sgt doctor` — a misconfigured
  profile is caught before a Work launches under it, not discovered
  mid-run. (#259, #260)

### Recoverable dirty patches

- A dirty working tree's patch is now captured byte-exact and verified
  with `git apply --check` before being trusted, closing a class of
  silent-corruption recovery failure. (#262)

### Commit imperative

- Shared workflow contexts now state the commit imperative explicitly,
  removing an ambiguity that previously left commit-or-not a per-actor
  judgment call in contexts where it should not have been one.

### Evidence and engine contract

- Per-stage artifact retention now keeps `tool_use` in transcripts
  rather than trimming it, and the engine contract gained a hard
  stage-output gate with `needs_input` recovery, a general
  promote/evidence finalize sweep, an opt-in branch-status wire for
  closing stages, and typed-ingest validation. (#234)

### Distro knowledge surface

- `sgt init`'s distro trees are now atomic and sgt-owned end to end: a
  managed `AGENTS.md` section, a `CLAUDE.md` symlink, a generated
  `.sergeant/index.md`, a route rewrite, and new `doc_routes` +
  edition doctor checks catch a malformed or hand-edited tree before it
  ships. (#232, #241, #261)

## [0.2.3] - 2026-08-23

Sergeant speaks Antigravity: a fourth native backend, registered the same
way Claude, Codex, and OpenCode are, with a new capability first this
release adds to the registry — typed native subagents — plus a launch-time
model-pin verification that beats every sibling adapter's own posture, and
two honestly-recorded refutations where the plan's own headline hopes did
not survive measurement.

### Agy backend

- **`sgt run --backend agy` and `sgt agy`** are real now — the agy adapter
  (`src/backend/agy.rs`) is registered in the `BackendRegistry` alongside
  Claude, Codex, and OpenCode. `sgt agy` is the exact mechanical mirror of
  the `goose` passthrough block (ADR 0006 D2) — origin-affinity routing has
  no origin to affine from without it.
- Two transports, chosen once per registration and never mixed mid-turn:
  `agy -p <prompt> --output-format stream-json` (one process per turn, the
  fallback under every capability below) and an adapter-driven
  `--input-format stream-json` input loop (one child for the whole
  execution, driven over plain stdio — no new crate, no port, no auth
  posture to carry, the cheapest second transport any adapter here has had).
- **Identity, the resolved model, and the effective permission mode all
  arrive on line 1** (the `init` event), before any model output — the
  strongest launch-time pin verification in the registry: claude verifies
  post-hoc from `modelUsage`, opencode post-hoc from `export`, and codex
  records substitution as undetectable, while agy's `verify_pin_from_init`
  refuses the LAUNCH itself the instant a substituted model is named, for
  zero wasted turns. Per-step usage rides every `step_update` too, so usage
  is known during a turn, not only at its end.
- **`native_subagents: true` on the loop transport — the first `true` for
  this flag anywhere in the registry.** A typed `subagent_info` record
  (`{type_name, role, initial_prompt, conversation_id, log_uri}`) names a
  child conversation distinct from the parent's, with its own transcript —
  admitted only on that typed evidence, never on assistant prose claiming a
  delegation happened.
- **Two refutations, recorded rather than left as silent gaps**: `ask` and
  `approval_flow` are **measured false**, not merely unmeasured, on both
  transports — sixteen candidate loop reply-event names were tried live and
  skipped, and a seventeenth, `control_request`, was refused outright, so a
  question that surfaces here has no channel to answer it on. OpenCode keeps
  the registry's only `true` on either flag. A SIGINT-based interrupt
  upgrade was tried and refuted too:
  on the loop transport it produced the same `ERROR`/"timeout waiting for
  response" terminal a plain deadline expiry gives, byte-for-byte
  indistinguishable — so `interrupt` stays `ProcessTreeTermination` on both
  transports, and `classify_terminal` gained an `InterruptedRunning`
  reading for that ambiguous terminal keyed on whether sergeant asked for
  the kill.
- **The soft-deny discrepancy, resolved by measurement and by transport.**
  Docs and an earlier probe pass both described a *hard* deny (typed
  `TOOL_ERROR`, terminal `ERROR`, exit 1); at the installed 1.1.19 this
  inverts on **print**: the tool step resolves `DONE` with no error and no
  output at all, the terminal is `CANCELED`, exit 0, and the *only*
  evidence anywhere is a plain-text stderr notice — classified fail-closed
  ambiguous rather than trusted as a clean completion. The **loop**
  transport instead does the opposite: the tool step carries the typed
  `TOOL_ERROR` verbatim, the terminal fails outright, and the child process
  itself exits, so a queued follow-up turn never runs and a subsequent send
  is refused rather than silently accepted. Two real shapes, both handled,
  neither guessed at.
- **No native OS sandbox to claim.** `nsjail` appears nowhere in the
  installed binary despite the docs' OS-native-mechanism claim, and
  `--sandbox`/`--add-dir` change nothing observable on the `init` line — so
  sandbox state is not launch-observable and this adapter does not pretend
  to report it. One paid probe found `proceed-in-sandbox` genuinely lifts
  the permission gate as a second, allow-rule-free channel, but the
  sandbox mechanism itself does not run on this host (a connection-reset
  failure at the tool layer) — evidenced, not claimed, and neither flag is
  composed by default on either transport. No NORTH-STAR amendment 4 entry
  is appended for this adapter, the identical posture ADR 0021 records for
  opencode: there is nothing here that qualifies as native enforcement to
  bind one to.
- **The measured permission-injection channel**: agy reads its settings
  from `$HOME/.gemini/antigravity-cli/settings.json`, so a per-run `HOME`
  override (`AgyConfig::settings_home`) is the lever — workspace-scoped
  settings files and a config-home environment variable were both measured
  absent. This wave wires the channel and synthesizes no policy: mapping a
  Work's declared mutation surface onto agy's `permissions` namespaces
  remains unbuilt, because inventing one here would be a security decision
  with no measurement behind it. Regardless of whether that channel is
  configured, the effective `permission_mode` is read off the `init` line
  and reported at launch — before any tool call, not discovered mid-run —
  so a tool-bearing Work launching under a denying posture is never a
  surprise.
- **Version churn, handled by provenance, not policy.** The installed build
  auto-updated `1.1.17` → `1.1.19` mid-sprint; `MEASURED_FLOOR` stays
  `1.1.17`, unconditionally provenance rather than a gate — every fixture
  here is a 1.1.19 capture, deliberately kept even though R1 would not have
  refused a build below its own floor either.
- **Two hardening fixes folded in during registration and finalize**: the
  zero-quota `/config` probe agy's own capability resolution calls during
  daemon registration is not always instant — an unauthenticated `agy`
  answers it with a blocking OAuth prompt — so the probe now carries its own
  5-second ceiling rather than risking the blocking-registration-call class
  of regression this project already tracks; and the loop transport's
  effective permission posture is now computed by the very reader thread
  that also composes the turn-end event, closing a race where a fast child
  could otherwise reach turn-end before the posture had been stored.
- **A recorded operational lesson, not a code defect**: composing
  `--disable-slash-commands` together with a slash-command prompt turns
  what looks like a zero-quota introspection call into an ordinary paid
  turn on the account's default model, not the pinned one — the one live
  probe this cost is reported rather than quietly absorbed, and this
  adapter's own zero-quota `/config` probe composes no such flag
  combination.

### Fake backend

- Five measured agy shapes, for contract-test authors who need to drive the
  engine against them deterministically: the print-transport soft-deny (a
  clean-looking tool completion whose turn never completes), the
  loop-transport denied-tool-kills-the-child shape (a typed tool failure
  after which the next send is refused), and the typed invalid-model
  refusal that mints no identity at all. Two more agy shapes reuse fake
  fidelity opencode's own wave already built rather than re-deriving it —
  init-first identity and death-without-terminal — each now proven against
  agy's own admission rows by a dedicated test. All additive: every
  existing fake-backend test is unchanged.

## [0.2.2] - 2026-08-23

Sergeant speaks OpenCode: a third native backend, registered the same
way Claude and Codex are, with two capability firsts this release adds
to the registry — a real approval round trip and a schema-distinguishable
actor question — plus a coverage-guard fix and a PATH addition.

### OpenCode backend

- **`sgt run --backend opencode` and `sgt opencode`** are real now —
  the opencode adapter (`src/backend/opencode.rs`, `opencode_serve.rs`)
  is registered in the `BackendRegistry` alongside Claude, Codex, and
  Docker (`docs/adr/0021-opencode-adapter.md`). Backend and harness
  selection were already separate, user-composable axes before this
  release (`sgt claude`/`codex`/`opencode`/`goose`, `--backend <name>`,
  `sergeant.toml`'s routing profiles); this release makes `opencode` a
  real option in that existing chain rather than an unregistered name.
- Two transports, chosen once per registration and journaled, never
  mixed mid-execution: `opencode run --format json` (one process per
  turn, server-minted session id, the simple fallback every capability
  below still works on) and an adapter-owned `opencode serve`
  HTTP+SSE child, one per execution, driven over the already-installed
  `reqwest` — no new crate, `Cargo.lock` byte-identical before and
  after the one feature-flag line this release needed.
- **`history: true`**, via token-free `opencode export` on run-json and
  `GET /session/{id}/message` on serve — the first `true` on this flag
  in the registry (both claude and codex report `false`). R4's "parity
  is the floor, not the ceiling" cashed in.
- **`approval_flow: true`** on the serve transport — the registry's
  first `true` on this flag anywhere. `permission.asked` parks the
  stage as `NeedsInput`; replies relay to the deprecated-but-live v1
  endpoint, measured to be the one that actually fires on 1.18.19
  despite the OpenAPI document naming a v2 as current.
- **`ask: true`** on the serve transport, through a genuinely distinct
  mechanism from approval_flow: opencode's own `question` tool carries
  a typed `question.asked` event naming the actor's own tool call —
  schema-distinguishable authorship, not guessed from narration.
  Measured end to end: an answer relayed to the reply endpoint resumes
  the session with no further client call.
- **Native turn interrupt** upgrades from a process-group kill
  (run-json) to `POST /session/{id}/abort` on serve, which kills the
  tool's own subprocess tree and leaves the session usable for a
  follow-up turn — a live-measured correction to this wave's own
  written spec: the synchronous abort response itself, not only a
  separate SSE frame, carries the abort signature.
- **No native OS sandbox to claim.** Unlike Codex's `workspace-write`
  enforcement, opencode exposes no OS-level sandbox mechanism at all —
  only permission config and per-tool disables, policy the model's own
  tool layer honors. Sergeant's observation layer stays the sole
  source of truth for this adapter, exactly as it already is for core;
  no NORTH-STAR amendment was needed, because there was nothing new to
  bind.
- **Every adapter's version-floor-as-provenance posture (R1)** applied
  here from this adapter's first commit: `opencode.rs` never had a
  refusal branch to strike, unlike claude/codex's own history with
  that rule.

### Coverage guard

- `src/backend/opencode.rs` and `src/backend/opencode_serve.rs` both
  cleared the repo's 90%-line coverage floor (Gate D) this release:
  91.21% and 91.22% respectively, up from a thin 82.91%/79.69% left by
  the wave that shipped the serve transport. The gap was almost
  entirely the *default* `Auto` transport path and its gate-failure
  fallback, never exercised because every existing serve test pinned a
  transport explicitly — now covered, alongside several
  previously-unreached terminal-classification and ask/permission
  reply-relay paths.

### Fake backend

- Four measured opencode failure shapes, for contract-test authors who
  need to drive the engine against them deterministically: a typed
  terminal error rendered identically to the real adapter's, a tool
  call the harness auto-rejected but the turn still completed around,
  a SIGKILL-with-no-terminal death, and an opt-in "the harness mints no
  native id until the first event" shape matching opencode's own
  server-minted session id. All additive — every existing fake-backend
  test is unchanged.

### PATH

- `~/.opencode/bin` added to `toolchain_path_dirs`
  (`src/harness.rs`) — measured absent from a non-interactive shell's
  PATH on Cerberus, the same failure class and the same one-line
  remedy that put `~/.cargo/bin`/`~/.local/bin` there.

## [0.2.1] - 2026-08-23

Content-only release: the distro content rebuild commissioned 2026-08-22
(`docs/proposals/distro-content-2026-08-22.md`; design record and every
ratified item at `sergeant-rs-workspace/knowledge/evidence/resources/
distro-content-series/design-proposal-2026-08-22.md`). Zero engine-behavior
change: every content byte in this release lives under `skills/`,
`.sergeant/workflows/`, `.sergeant/common/contexts/`, `.sergeant/index.md`,
or `AGENTS.md`; `git diff --stat main -- src/ .github/` is empty, and this
section's own version bump is the only `Cargo.toml`/`Cargo.lock` edit. The
embedded `software-change` default loop is untouched — see "Deferred"
below for why, and why this release is 0.2.1, not the sprint plan's
originally proposed 0.3.0.

### Workflow catalog rebuilt: 19 published workflows -> 7

Owner ruling (2026-08-22), from direct knowledge of the estate's own
history: "I never guided to select any other workflows because the ones we
had never applied or sucked. They're like one stage or fragile." Sixty of
the estate's own sixty-one dispatched Works ran the engine's embedded
default and never selected a published workflow; nine of the nineteen fail
the owner's own stated bar — genuinely multi-stage and robust — on
inspection. The rebuild is ratified; every surviving and new package
states a robustness argument: what each stage boundary buys under crash or
stall, which stage attacks the previous stage's output, and what happens
when a stage cannot complete.

- **`implement-change`** (9 stages) — the flagship. Orient, baseline,
  implement, validate, a 4-axis panel (spec-fidelity, invariants,
  simplicity, test-honesty) run as isolated sub-agent seats spawned in one
  message, per-axis refuters defaulting to refuted, fix on confirmed
  findings only, re-verify over the fix commits, close. Absorbs
  `implement` and `worker-mission`.
- **`fix-defect`** (8 stages) — reproduce-first, hard-gated: no edit until
  the defect reproduces; then hypothesize, instrument, fix with a
  regression test, and the same panel -> refute -> re-verify chain.
  Reshaped from `diagnose-bug` (6 stages, name changed); it remains the
  only workflow a live Work has ever bound.
- **`investigate`** (6 stages) — frame a bounded question with a stated
  stopping condition, fan out N isolated evidence seats, synthesize one
  cited document, challenge it with a refuter, record, close. Absorbs
  `research` (shipped as a single stage) and `wayfinder` minus its
  `00-name-destination` stage, deleted rather than carried forward:
  naming a destination needs a live interview, which R-NS-6 places in
  Captain, before dispatch, never in a dispatched stage.
- **`review-change`** (6 stages) — the standalone, read-only panel for a
  diff arriving from outside a Work: pin the revision, identify the spec
  source, run the 4-axis panel, refute, independently verify each
  survivor against current state, emit a typed finding set. Never fixes.
  Reshaped from `code-review` (4 stages, 2 axes).
- **`remediate-findings`** (6 stages, new) — consumes an approved typed
  finding set and accounts for every one: verify against current state,
  dispose (accept/reject/supersede, with a reason), fix accepted findings
  only, re-verify over the fix commits, and a disposition matrix proving
  nothing was silently dropped.
- **`author-document`** (6 stages, new) — a document as the deliverable,
  fidelity to the brief as the top-weighted review axis. Absorbs
  `record-decisions` as a named profile section — a section, not a
  construct, since sergeant has no profile mechanism yet (filed as an
  engine ask, not silently assumed).
- **`validate-and-ship`** (7 stages, `version: 3`) — kept and reshaped in
  place: the one shipping invariant in the catalog, and the ancestor of
  `review-change`'s never-edit rule.

### Eighteen retired

`code-review`, `cross-repo-work`, `deepen-module`, `diagnose-bug`,
`dispatch`, `implement`, `prototype`, `record-decisions`,
`recover-stalled-worker`, `repo-to-icm`, `research`,
`resolving-merge-conflicts`, `to-tickets`, `triage`, `validate-intent`,
`vet-external-skill`, `wayfinder`, `worker-mission`. Every disposition and
its reason is recorded in `.sergeant/index.md`'s retirement log; each
package's full content survives in git history. Two dispositions named
here because they cost something rather than merely tidying:

- **`dispatch`'s retirement removes the only place safety-sensitive
  keyword routing (auth/secrets/payments/migrations/production) lived.**
  This release does not replace it. Named as a live gap on the head PR's
  ratify list, not silently absorbed.
- **`repo-to-icm` relocates rather than retires** — to
  `sergeant-rs-workspace/.sergeant/local/workflows/`, this project's own
  doctrine-bootstrapping tool and never a product capability — and takes
  with it the library's only `kind = "execute"` stage. This library no
  longer exercises that stage kind anywhere.

### Captain's kernel: 5 published skills -> 14

Three ship byte-identical (`grilling`, `estate-navigation`,
`sergeant-help` — edition bump only). Eleven are new or reshaped:
`orient`, `brainstorm`, `clarify-intent`, `scope-intent`,
`define-acceptance` (absorbing `validate-intent`'s eight-dimension check
as an in-dialogue review — the cost is independence, stated rather than
hidden), `decide` (carrying the one-question-per-turn discipline),
`decompose`, **`select-workflow`** (reads `.sergeant/index.md` live, never
restates it, and leaves a dated selection record naming what was passed
over and why — the direct answer to the sixty unrecorded choices above),
**`plan-sprint`** (the three-sprint-proven method, codified with its
engine caveat stated honestly: its recon and panel seats are harness
sub-agents, not Sergeant Works, until child-workflow dispatch lands),
`review-outcome`, `retrospective`. `to-spec` and `grill-with-docs` retire
into their successors.

### AGENTS.md: the dispatch-time discipline

- The `## Trigger -> skill/workflow routing table`'s dispatch row now
  names **Captain**, not the `sgt run` command, as the workflow selector —
  correcting the cell the sixty unrecorded selections trace back to.
- New stable-operating-invariant text: "Before any `sgt run`, consult
  `.sergeant/index.md` and name the workflow you selected. Omitting
  `--workflow` binds the embedded default loop; that is a selection and
  must be stated as one... An unnamed default is not a selection."
- "**Captain owns ambiguity. Sergeant owns completion.**" stated beside
  `R-NS-6`, which remains the enforcing rule where the two could disagree.

### Contexts

`.sergeant/common/contexts/` gains nine reusable stage contexts (shared
panel/refute/fan-out-evidence/pin-fixed-point/identify-spec-source/
resolve-conflicts machinery, authored once instead of re-derived per
package) and four policy contexts: `test-first` (consolidating `tdd.md`
and `test-quality.md`, both deleted this release, closing a dangling
reference to the already-retired `tdd` package that had stood since
ICM-R3), `independent-review`, `evidence-requirements`, and
`model-assignment` — the last un-copy-pasted from three separate sprint
plans' identical paragraph.

### Fixed (docs)

- `docs/glossary.md`'s Placement Ladder citation pointed at
  `.sergeant/workflows/repo-to-icm/_config/icm-ladder.md`, a path this
  release's `repo-to-icm` relocation removes from this repository;
  repointed to the file's new home in `sergeant-rs-workspace`.
- `NORTH-STAR.md`'s cross-repo delivery-ordering gap named the now-retired
  `cross-repo-work` workflow as its current home; repointed to
  `scope-intent`'s `targets.dependency_order` field, the claim's actual
  owner since the dissolution (the underlying gap — no engine-side
  dependency contract — is unchanged and still open).
- `README.md`'s workflow-directory illustration and catalog summary named
  `software-change` as an on-disk package and six retired workflows
  (`code review`, `research`, `prototyping`, `cross-repository work`,
  `intent validation`, `decision recording`) as the shipped set; both now
  show the real seven, and the illustration uses `implement-change`, the
  package actually on disk.
- `edition` front matter aligned to `0.2.1` across all 21 shipped
  templates (was a mix of `0.2.0` and one `0.1.0` outlier). Cosmetic —
  `sgt init`/update always stamps the running binary's own version at
  write time regardless of the checked-in value (ADR 0016) — fixed
  because it was odd on sight, per W2's own ratify list.

### Deferred (ratified at kickoff, not executed this release)

The kickoff ruling retired the embedded `software-change` default outright
("an old way of ensuring there's at least one workflow"; unspecified
`--workflow` becomes bare intent execution) and granted the one
engine-code exception that requires. **This sprint does not execute that
retirement.** The owner's own scope ruling — "the only changes here should
be to workflows/, skills/, and AGENTS.md" — bounds this sprint's diff to
content, and a real `src/domain/workflow.rs` fallback change plus a
`src/workflows/software-change/` deletion do not fit inside that boundary
regardless of what the kickoff separately ratified. W2's spec deferred the
retirement to a future release that actually touches `src/`; it remains
ratified and outstanding, carried on this release's own ratify list (see
the head PR). **This release's version number reflects the deferral**:
`0.2.1`, a content-only patch — not the sprint plan's originally proposed
`0.3.0`, which priced in the retirement this release does not ship.

### Ratify-at-review (owner, at the head PR)

Carried forward from W2 §8 and W3, restated here for the release record:
the `tests/f_doctrine_skew.rs` edit (a fourth surface touched, under the
plan's existing test rider); the two homeless-policy homes
(`test-first`/`model-assignment` in shared contexts, AGENTS.md pointing
rather than restating); the safety-sensitive routing gap `dispatch`'s
retirement opens with no replacement; the `record-decisions` profile
section standing in for a profile construct sergeant does not have yet
(J.11); the `.sergeant/lib/` relocation and the finalize ruling (no
shipped stage invokes a helper `sgt init` does not write); the unmeasured
panel budget (four seats plus four refuters in one stage, against a
two-seat precedent) — this release's proof obligations are on the head
PR, not executed here (W4 is finalize, not proof; see the head PR's own
scope note); and this section's own version-number correction (0.2.1, not
the plan's 0.3.0) and the still-outstanding embedded-default retirement.

## [0.2.0] - 2026-08-22

Sergeant speaks Codex: a second native backend, selectable the same way
Claude always has been, plus a repo-wide fix to how every adapter reports
an old CLI.

### Codex backend

- **`sgt run --backend codex` and `sgt codex`** are real now — the codex
  adapter (`src/backend/codex.rs`, `codex_appserver.rs`) is registered in
  the `BackendRegistry` alongside Claude and Docker, closing deviation D6
  (`docs/adr/0020-codex-adapter.md`). Backend and harness selection were
  already separate, user-composable axes before this release
  (`sgt claude`/`codex`/`opencode`/`goose`, `--backend <name>`,
  `sergeant.toml`'s routing profiles); this release is what makes `codex`
  a real option in that existing chain rather than an unregistered name.
- Two transports, chosen once per registration and journaled, never mixed
  mid-execution: `codex exec` (one process per turn, the simple fallback
  every capability below still works on) and an adapter-owned
  `codex app-server --listen stdio://` child, one per execution, offering
  a native turn interrupt (a first-class `interrupted` terminal instead of
  an inferred process-kill), mid-turn token-usage events, and a typed
  auth-expiry error taxonomy exec never had.
- **Sandbox enforcement, on by default.** Codex turns run under
  `workspace-write`, scoped to the Work's own declared surfaces —
  enforcement the harness itself provides, on top of (never instead of)
  sergeant's own observation layer, which remains what sergeant charges
  dirty evidence from (`NORTH-STAR.md` amendment 4, dated 2026-08-21).
  `danger-full-access` is not offered by this adapter; an operator who
  wants it configures their own `~/.codex/config.toml` and opts out of
  sergeant's default via a launch profile.
- **Capabilities, reported honestly, negatives included.** `ask` is
  `false` on both transports: exec has no ask channel at all, and
  app-server has no `NeedsInput` mapping or answering path yet, so the
  capability cannot be admitted regardless of model behavior. A live,
  `#[ignore]`d admission test against the app-server's
  `request_user_input` method — the one place Codex's protocol could
  have exceeded Claude's — stays in the suite, opt-in via
  `SERGEANT_CODEX_TESTS=1`, for whoever next re-runs it; this release
  does not report a completed run of it against the pinned dev-tier
  model. `history` stays
  `false`: the durable transcript exists (a rollout file, a thread index)
  but reading it would be a private-format read this release does not
  attempt, and the capability's own whole-or-refuse rule forbids claiming
  it on anything less than a proven-complete read.
- **The narration hazard, named and tested.** Codex's cheap dev tier has
  been measured narrating a command's outcome with no corroborating tool
  record and no filesystem effect. The adapter's decoder has exactly one
  code path that can ever produce tool evidence, and both the codex
  contract suite and the deterministic fake backend now carry a test
  proving narration text alone never manufactures one.

### Every adapter's version floor is provenance now, not a gate

- **The version-floor refusal is struck.** `sgt`'s adapters used to refuse
  to launch below their own measured-floor CLI version. That refusal is
  gone: an old CLI is reported as available, with an honest
  unmeasured-provenance detail in `sgt doctor` and the probe record,
  instead of an outage the operator has no lever to fix. This is a
  usability fix, not a loosened guarantee — the two *other* conditions a
  version check used to bundle in (a required launch flag missing from
  `--help`; an unparseable version string) are untouched refusals, because
  neither is actually a version-policy question.

### Fake backend

- Five more measured-shape scripts, for contract-test authors who need to
  drive the engine against them without a live harness or a token spend:
  a deferred turn finish; a turn whose own signal never arrives, distinct
  from a hang; a harness-confirmed interrupt that kills nothing (distinct
  from the process-tree kill every earlier interrupt modelled); a turn
  answering an item queued before it was sent; and narration with no
  corroborating tool evidence. All five are additive — every existing
  fake-backend test is unchanged.

## [0.1.3] - 2026-08-21

Foundation close-out: the release pipeline proven end-to-end via a
from-branch dry run, and bounded retention — the daemon now prunes
terminal history past a declared cap so disk stays bounded and startup
stays fast, with every discard journaled and named, never silent.

### Release pipeline

- Gate A permits `mode=dry-run` dispatch from any ref, skipping the
  SHA-equals-`origin/main` assertion that mode alone needs; `publish`
  keeps the unchanged main-only + HEAD-current check — release authority
  is unaffected, only the pre-merge proof surface is (issue #17's owner
  commission: "the completed release pipeline that will work" needed
  proving before this release, not only after it).
- `gh release edit --latest` now runs as its own post-publish, by-tag
  step, superseding #215: `make_latest=true` riding the same PATCH as
  `draft=false` was silently dropped by GitHub because the release was
  still a draft at validation time — measured live on v0.1.2.
- R7 citations added at the four `gh api` sites with no by-ID or by-tag
  porcelain equivalent (comment-only; no behavior change).

### Retention

- **`[estate] retention = N` in `sergeant.toml`** (issue #17's Q1–Q10
  rulings record): how many terminal Works of history this estate keeps.
  Default 1000 (≈1.8 GB bounded total on this estate's own measured
  1.8 MB/Work basis), minimum 64. Older Works — and the blobs only they
  referenced — are pruned automatically, journaled, whole segments at a
  time; nothing is ever deleted silently. `sgt init` scaffolds the key
  **commented**, so an 0.1.2 binary is never handed a manifest it cannot
  parse.
- Startup replays only the newest 16 segments / 128 MiB plus a persisted
  summary cache, instead of walking the whole journal four times from
  scratch on every start — roughly 30% faster on this estate's own dogfood
  journal (1.30s → 0.89s; a journal with real retained depth wins more).
  `sgt daemon --rebuild-cache` forces a full rebuild on demand.
- `GET /v1/events` and the SSE stream now carry the journal's actual replay
  floor (`floor_seq`) instead of letting a client infer one of `1`; a
  `from=` below the floor is served from the floor, never an error.
  `sgt work show`/`sgt work transcript` answer a pruned Work by name
  ("pruned on `<date>` under policy") instead of a 404 indistinguishable
  from a Work that never existed.
- `sgt doctor` gains `journal_growth`: live segment count and bytes, the
  replay floor, retained-Work count against the cap, a stalled prune's
  blocking Work and age (reported, never overridden — Q7), and the last
  startup rebuild's duration (warn at 10s, fail at 30s, the owner-ruled
  trigger made mechanical).
- `docs/adr/0003-durability-promise-and-storage-preconditions.md` amended:
  durability while retained is total; durability forever was never the
  promise. New `docs/adr/0019-bounded-retention.md` records the mechanism.
  `docs/proposals/journal-archival-rule-c.md`'s compressed-archive design is
  superseded by the simpler count-based model above.
- A manual `sgt journal prune` verb is deliberately deferred (pruning must
  run inside the daemon process; the declared policy is already the whole
  authorization — a manual trigger would add no capability, only ceremony).

## [0.1.2] - 2026-08-20

Maintenance release: one day of backlog close-out rationalizing the base
feature set the estate-root contract (0.1.1) implied, followed by a CI/CD
hardening pass adopting one rule — float to discover, pin to build. Nothing
here is a new direction — these are the verbs, declarations, and honest
reports that should have always been there, plus the accumulated perf,
hygiene, and reproducibility debts paid down.

### CI/CD and supply chain

- The compiler is pinned: `rust-toolchain.toml` names 1.98.0 exactly (dev
  boxes and CI resolve the same toolchain from the same file; the
  dtolnay/rust-toolchain wrapper is gone), and MSRV is declared and
  CI-checked at the measured floor 1.89.0 — set by this crate's own
  `File::try_lock` usage, verified empirically, not inferred from metadata.
- Every cargo invocation in CI runs `--locked`, with `cargo metadata
  --locked` guards ahead of third-party subcommands and a
  `git diff --exit-code` no-mutation gate; runners are numbered
  (`ubuntu-24.04`, `macos-26`), helper tools exact-pinned
  (`cargo-llvm-cov@0.9.0`, `cargo-deny@0.20.2`, `cargo-cyclonedx@0.5.9`),
  and the one stray checkout v4.4.0 joined the repo-wide v7.0.1 SHA pin.
- A weekly `canary.yml` builds against floating `stable` Rust on floating
  `*-latest` runners — deliberately loud: no `continue-on-error`, and a red
  run maintains an `upstream-drift` tracking issue. Required CI stays
  deterministic; the canary is how the next upgrade PR gets discovered.
- The release path verifies what it executes and ships: `dist` is
  bootstrapped from its checksum-pinned, attestation-verified tarball
  (`scripts/release/install-dist.sh`) instead of `curl | sh`, and the
  generated shell installer now verifies archive SHA-256s before extraction
  — dist's per-target build manifests are wired into the global build so
  its own (previously starved) `verify_checksum()` path runs, gated in CI
  by positive and corrupted-archive smoke tests
  (`scripts/release/verify-installer-checksums.sh`). SBOM attestations
  moved from the deprecated `actions/attest-sbom` to unified
  `actions/attest` v4.2.2, keeping 1:1 target↔SBOM pairing. Each package
  leg records a build-environment evidence file into the release assets.
- The doctor probe image is minor-pinned (`alpine:3.24`), the direct
  reqwest dependency moved 0.12→0.13.4 collapsing a duplicate HTTP/TLS
  stack out of the binary, README install commands use the `/latest`
  release alias guarded by a new docs-consistency CI step, internal
  schema identifiers gained golden contract tests, and
  `docs/version-policy.md` records the pinning policy — including what is
  deliberately not pinned, and why.

### Fixed

- `release.yml` passes `make_latest=true` at publish and asserts
  `/releases/latest` resolves to the released tag, retrying eventual
  consistency — a stale Latest badge now fails the run loudly (#200).
- `scripts/probe-env.sh` no longer substitutes a sentinel string as a
  wrapped command's own output when `timeout(1)` is missing: it falls back
  to `gtimeout`, then runs unbounded and reports the unenforced bound as
  its own measured row; Cores falls back to `sysctl -n hw.ncpu` (#143).
- `sgt doctor` replays the journal once instead of three times; its cost no
  longer triples the journal-size term (#12).
- The `events` table gained an index on `(kind, work_id)`, roughly halving
  `blocked_time_per_work`'s cold-call cost at every measured mark (#10).
- A torn final journal line observed while the daemon is mid-append is now
  classified as a tolerable transient (`is_possible_torn_tail`), distinct
  from mid-file corruption which still fails closed; the test harness
  retries only the former (#169).
- Doctor's `data_dir` check names the winning resolution rung when an
  explicitly-set `$XDG_DATA_HOME` is outranked inside an estate, and
  ADR 0008's dangling precedence pointer was replaced with the adjudicated
  order: `--data-dir` > `SGT_DATA_DIR` > manifest `data_dir` (#80).

### Changed

- Terminal Work structs now live in a bounded cache (capacity 1024, Rule A
  pattern): `work list` keeps full history via an always-retained slim
  index, `work show` on an evicted Work re-derives from the journal, and an
  evicted list row carries `"evicted": true` with the effective integrity
  disposition — a stranded `completed_dirty` never reads as plain
  `completed` (#4).
- The perf harness scaffolds its own scratch estate and submits with
  explicit scope, so it runs under the estate contract (#8, #10, #12
  measurement support).
- `validate-and-ship`'s close-out stage may complete only once any external
  pipeline run it drove reached a terminal disposition, or the handover log
  records why one was deliberately left open (#124).
- Dispatch doctrine now matches the shipped engine: risk routing points at
  AGENTS.md's Captain intent discipline (the eight-dimension brief's one
  home), and the monitor stage describes the engine's own startup
  reconciliation instead of a shell-tool-era sync verb (#166, #167 —
  closed, no verb needed: reconciliation is engine-owned).
- NORTH-STAR and AGENTS.md state the ratified mutation-surface contract:
  per-Work worktrees with declared surfaces, violations journaled and
  charged as dirty evidence — observation with honest consequences, not
  prevention; shared-mount collision named as accepted risk (#180).

### Added

- `sgt work sweep` (#159): classifies every `sergeant/*` ref per mount —
  active / redundant (provably ancestor of the mount's default branch) /
  retained (unique content, commit count) / orphan (no journaled Work, the
  #172 fold-in) — plus prunable worktree registrations. Read-only by
  default; `--delete-redundant --yes` deletes only server-re-verified
  redundant refs under the repository gate, journaling each deleted tip
  SHA. Failed Works classify active and are never deletable.
- Forge-neutral `upstream = "<url>"` on `[[repo]]` (#112): recorded by
  `sgt repo add --upstream`, ensured as the mount's `upstream` remote at
  clone/repo-add (admission stays remote-agnostic), drift reported by
  doctor with the exact remedy. Any forge, or none — git is the
  assumption, not a CLI.
- `sgt run --intent-file <path>` (#166): the file's contents become the
  intent verbatim; mechanical guards only (symlink refusal, regular file,
  1 MiB cap, UTF-8) — ends the scratchpad-and-`cat` workaround for
  multi-paragraph intents.
- Two workflow packages: `validate-intent` (#201, optional pre-dispatch
  review of an intent across the eight dimensions — reports gaps, never
  fills them) and `record-decisions` (#88, transcribes already-made
  decisions with fidelity-first review; gaps are logged, never invented).
- `docs/proposals/journal-archival-rule-c.md` (#17): the gauntlet-evaluated
  Rule C archival design — ten open questions await a follow-up discussion;
  no behavior change in this release.

## [0.1.1] - 2026-08-20

### Added

- **Terminal Works now report an integrity disposition (#173).** At
  retirement, teardown reconciles what a Work's worktree *actually* held
  against what its binding claimed: the branch HEAD ended on and its tip,
  whether HEAD was detached, and whether the worktree and its source
  checkout still agree on one Git common directory. Divergences are
  recorded as a closed vocabulary of findings
  (`assigned_worktree_uncommitted`, `assigned_worktree_missing`,
  `assigned_branch_mismatch`, `assigned_head_detached_or_unreferenced`,
  `assigned_common_dir_mismatch`), and a terminal Work carries a
  `clean`/`dirty` integrity disposition beside — never inside — its state.
  A dirty completion reports as `completed_dirty`; `failed` and `canceled`
  keep their own state strings and carry the axis in the new `integrity`
  key, which `sgt work list`, `sgt work show`, and the TUI's work detail
  all render. Work state, the state machine, and the transition table are
  unchanged.

  Before this, a worktree that checked out a different branch and committed
  there was reported as a clean removal at the untouched base SHA —
  indistinguishable from a surface nothing ever touched — while removing
  the worktree destroyed the only record of where the output had gone.

- **Teardown no longer removes a worktree holding unreferenced commits.** A
  worktree whose HEAD is detached at a commit no named ref is proven to
  reach is retained (`retained_unreferenced`) rather than removed, and `sgt
  work reap` declines it with the remedy named. A removed worktree's HEAD
  stops being a garbage-collection root; retaining it is how commits
  nothing else points at survive.

- **Estate drift is observed at retirement.** One `git rev-parse HEAD` per
  bound repository mount, compared against the commit the Work was cut
  from. Reported with `attribution: unknown` and never used to make a Work
  dirty: a mount moving during the Work window is not evidence the Work
  moved it.

  Journal changes are additive. A `surface.torn_down` recorded before this
  release replays unchanged and reads as *not assessed* — never as clean.

- **Explicit Work scope is now required for a multi-repository estate
  (estate-root proposal §7).** Submitting `sgt run` with no `--repo`,
  `--group`, or `--all` used to silently expand to every declared
  repository; it is now refused with a structured 422 naming the repo
  count, the declared repositories and groups, and the three ways to
  select — `--repo <name>` (repeatable), `--group <name>`, or `--all`. A
  one-repository estate is unaffected: it still infers its sole repository
  on an empty scope. Group expansion is no longer CLI-side: the daemon
  resolves `--group`/`--repo`/`--all` against its own bound manifest, so
  every client submitting the same scope — CLI, TUI, or a direct API caller
  — reaches the identical resolution. The TUI's New Work form submits that
  same structured scope: its dead `workspace` field is replaced by a
  `group` field, and both it and `repositories` are forwarded unexpanded,
  so naming a group in the TUI resolves exactly as `sgt run --group` does.
  `sgt run --all` is new — an explicit, journaled selection of
  the whole estate. A submitted Work now records both the request form
  (`scope_request`: repos/group/all as submitted) and the resolved
  repository list, so a later manifest edit cannot rewrite what an
  already-journaled Work meant. Owner ruling (2026-08-20): `--all` combined
  with `--repo` and/or `--group` is refused (`conflicting_scope`, 422)
  instead of `--all` silently winning — clap rejects the combination
  locally, and the daemon's own `Engine::resolve_scope` is the authoritative
  check for a direct API caller.

- **`sgt -C <estate-root>`** names an estate explicitly instead of requiring
  a `cd` (gauntlet finding C10, approved by the owner 2026-08-20). It is a
  global flag on every verb, and it names an **exact** root: no search
  happens from it, and it is validated by exactly the rule the current
  directory is. The CLI is agent-first — an agent should not have to mutate
  its own working directory to address an estate.

- **`sgt doctor` gains an `estate_root` row.** It reports whether the
  directory it was run from is an estate root at all — `ok` naming the root
  when it is, `fail` carrying the remedy when it is not. `doctor` still
  works outside an estate, still never searches upward, and still never
  starts a daemon.

- **`sgt doctor` gains a cheap `git_surfaces` row (estate-root proposal
  §12.2).** One bounded summary — active works, active linked worktrees,
  retained worktrees, retained patches, retained artifact size, journaled
  Work branches, terminal dirty Works — derived from the journal plus
  retained-artifact filesystem metadata only, never a per-branch `git`
  walk. Silent (`ok`, "not an estate root") outside an estate; names `sgt
  work retained`/`sgt work show`/`sgt work reap` as the separate
  inspection/cleanup remedy when residue is nonzero. Classifies nothing as
  merged or redundant and deletes nothing — §12.3's expensive
  reconciliation stays future work.

- **Doctrine-skew tests (`tests/f_doctrine_skew.rs`, estate-root proposal
  §18).** New, focused tooling — no `validate-skew` or similar existed
  before this. Checks: AGENTS.md's "Session start" claims (the unscoped
  command set, the root-gate refusal wording, the `-C` flag) against the
  real binary; no `CONTEXT.md` under the embedded `.sergeant/workflows/`
  distro instructs a stage actor to run an estate-scoped `sgt` command from
  inside its own Work surface without a disclaiming note nearby; the
  estate-root proposal's own canonical manifest example (§13.1) still
  parses under the current schema; no shipped workflow/skill content
  quotes the removed `--workspace` flag (C12 regression pin); and the
  embedded `skills/` root/preflight remedies match the refusal text, the
  `--help` surface, and the preflight remedy strings the binary really
  emits.

### Changed

- **Every Work is now admitted against a complete Git preflight before it
  exists (estate-root proposal §8).** Submitting used to read each mount's
  HEAD with no judgment at all: a dirty mount, a detached HEAD, an
  unresolvable commit, an existing `sergeant/<work-id>` ref, an occupied or
  still-registered surface path, or a repository whose lock could not be
  taken all became a durable Work record first and a problem afterwards.
  Core now checks all eleven §8.1 facts for **every** selected repository
  before `work.submitted` is journaled and before any Git mutation, and
  refuses with a structured 422 when any of them is unresolved — so there
  is nothing to clean up, in the journal or in your checkouts.

  Each check has its own stable error code, the evidence observed, and a
  named remedy: `git_preflight_mount_missing`, `_mount_aliased`,
  `_top_level_mismatch`, `_linked_worktree_source`,
  `_common_dir_unlockable`, `_detached_head`, `_unresolvable_head`,
  `_dirty_mount`, `_work_branch_collision`, `_worktree_path_collision`,
  `_incomplete_plan`. A refusal reports every unresolved finding across the
  whole scope, so a multi-repository submission is fixed in one pass rather
  than one submission per repository; when part of a scope cannot be
  planned, the refusal says so explicitly and **no** repository is
  materialized, including the ones that were fine.

  The base a Work runs on is now an *admitted* fact: `base_branch` and
  `base_sha` are what preflight judged, not what the mount happens to say
  by the time `git worktree add` runs. Sergeant still performs no automatic
  network or branch-changing Git command on any admission path — no fetch,
  pull, push, rebase, switch, checkout or remote-default inference — which
  is now asserted against a recording Git binary across a whole real
  admission rather than only stated.

  **Upgrade note:** a mount with uncommitted changes, or on a detached
  HEAD, is refused where it previously ran. Commit or stash, check the
  mount out onto the branch the Work should be based on, or use the new
  bounded override below.

- **`sgt run --override-git-preflight`** waives exactly two of those
  cautions, and only because an exact commit can still be pinned in both
  (§8.3):

  - a **dirty** mount — the Work is based on the committed `HEAD`, the full
    `git status --porcelain` output is journaled as evidence, and the
    record states explicitly that the uncommitted changes are excluded from
    the Work base (they stay in your mount, untouched);
  - a **detached** mount — the exact `HEAD` is pinned and **no** named base
    branch is recorded.

  It never waives anything else: not an invalid estate, unresolved or
  unknown scope, an unknown or aliased repository, an unresolvable top
  level / common directory / commit, a lock conflict, an existing Work ref
  or surface path, a failed surface construction, or a
  backend/workflow/profile failure. An unresolvable `HEAD` says so
  explicitly — the override is unavailable there because no exact base can
  be pinned. Override may waive policy caution; it may not replace a
  missing fact or overcome mechanical impossibility.

  It is available **only** as a flag on `sgt run` (and the matching
  `override_git_preflight` request field). There is no configuration key,
  no `[estate]` key, no profile field and no run template that can set it:
  the operator types it for that submission or it is not set. The
  authorization and every waived finding are journaled with the Work.

- **A repository binding records no base branch when there is none.**
  `base_branch` on a `RepositoryBinding` (and on the `BindingSummary`
  backends receive) is now nullable, and a detached admission records an
  explicit `null` rather than the old `"(detached)"` sentinel — a value
  that read like a branch name in the field every consumer branches on. A
  binding journaled before this change replays exactly what it recorded,
  sentinel included.

- **An estate is now exactly the current directory (estate-root proposal
  §4).** Every estate-scoped command — `run`, `status`, `work *`,
  `respond`/`retry`/`extend`/`cancel`, `watch`, `analytics`, `tui`,
  `daemon`, `repo *`, `group *`, `workflow *`, and the `claude`/`codex`/
  `opencode`/`goose` harnesses — requires the working directory itself to
  contain a `sergeant.toml` that parses, declares `[estate]`, and satisfies
  the schema. **Sergeant no longer searches parent directories**, and no
  longer infers an estate from Git. Running `sgt run` from
  `repos/payments-api` used to find the estate above it; it now refuses,
  names the path it expected, explains that parents are not searched, and
  tells you to `cd` to the root (or `sgt init` here). Only bare `sgt`,
  `--help`, `--version`, `sgt init` and `sgt doctor` work outside a root.

  Validation happens before the data directory is resolved, before the
  runtime descriptor is read, before any daemon is spawned or contacted,
  before any repository is inspected, and before a harness is prepared or
  exec'd — so a directory mistake cannot attach to, or spawn, the wrong
  daemon.

  **Upgrade note:** if you have been running `sgt` from inside a repository
  mount, `cd` to the estate root or pass `-C <estate-root>`. If you relied
  on the zero-configuration single-repository mode — a plain git checkout
  with no `sergeant.toml` — run `sgt init` there once; a one-repository
  installation is now an estate with one declared repository.

- **A daemon belongs to one estate.** Daemon startup takes a canonical
  estate root and refuses to come up if it is not one. The runtime
  descriptor records `estate_root` and `manifest_path`, and every client
  verifies that root against its own before using the endpoint: a daemon
  bound to another estate is a named refusal listing both roots, never a
  connection and never a second daemon over the same data dir. The engine
  plans against that bound estate rather than rediscovering topology from
  each request's working directory, which removes the recursion hazard
  where a command launched from inside a Work surface rediscovered that
  linked worktree as a new workspace. `origin.cwd` is still recorded, as
  evidence only.

  The descriptor schema is `sergeant.runtime/v2`. There is no compatibility
  shim: a `v1` descriptor left by an older build carries no estate root, so
  a client cannot verify the binding at all and fails closed with the
  remedy — stop the old daemon and let a restarted one republish.

- **Repository mounts are derived, not configured (§6).** `[[repo]] path`
  is **removed** from `sergeant.toml`. Every repository is mounted at
  `<estate-root>/repos/<name>`, and that is the only place it can be. A
  manifest still declaring `path` is refused with a message naming the
  removal, not a generic unknown-field error. Mounts are validated on load:
  a missing mount, a symlinked or aliased one whose real Git top level is
  elsewhere, and a linked worktree offered as a repository source are each
  refused by name, reporting the expected derived path alongside the actual
  top level or common directory. Separate estates use separate clones, even
  for the same upstream repository.

  **Upgrade note:** delete every `path = "..."` line from your
  `sergeant.toml`'s `[[repo]]` entries. If a checkout is not already at
  `repos/<name>`, move or re-clone it there. `sgt repo add` writes the new
  shape and clones to exactly that path; `sgt repo remove` still undeclares
  without deleting the checkout.

- **`sgt <harness>` binds the estate explicitly.** It validates the exact
  root first, then exports `SGT_ESTATE_ROOT`, `SGT_DATA_DIR` and
  `SGT_ORIGIN_CLIENT` and starts the harness in the root. The environment
  helps later invocations name the correct root; it never waives the
  exact-root check — `cd` into a mount inside a bound session and `sgt run`
  still refuses, naming both roots and how to return.

### Removed

- **Upward estate discovery and the zero-configuration Git fallback are
  gone.** `Workspace::discover`, `discover_scoped`, the `find_estate_upward`
  ancestor walk and the `git rev-parse --show-toplevel` fallback beneath it
  are deleted outright, not merely left uncalled. R-MVP1-12 is superseded;
  ADR 0008 carries an amendment recording that the manifest keeps its
  storage-path authority while the *discovery* of the manifest becomes
  exact-root only.

- **`--workspace` is gone, from the CLI and the wire.** The daemon is bound
  to exactly one estate; a client-supplied workspace label had no role left
  to play. `Work.workspace` is no longer written by any new submission —
  `scope_request` and the resolved `repositories` list are its replacement
  — but the field itself still deserializes so a pre-existing journal (the
  live estate journals 150+ Works carrying it) keeps replaying unchanged.
  Analytics and the provenance graph now read the estate label off the
  plan-time `workflow.bound` event instead of the submission, and tolerate
  its absence for a Work that never reached a workspace at all.

### Documentation

- **Corrected: the shell installer does not verify downloads.** `dist`'s
  generated `sergeant-rs-installer.sh` (published from v0.1.0 onward)
  declares the local variables it would use to check a downloaded
  archive's checksum but never assigns them anywhere in the script, so its
  verification branch is structurally unreachable — every install via the
  `curl | sh` convenience one-liner prints "no checksums to verify" and
  installs unverified, confirmed by running the published v0.1.0 installer.
  This was previously undocumented; README.md's "Installing a released
  binary instead of building from source" section now states this plainly
  and gives a manual, deliberate verification path instead: download the
  archive and its `.sha256`, `sha256sum -c` it, then `gh attestation
  verify` it against this repo's build-provenance attestation, then
  extract and install by hand. `.github/workflows/release.yml`'s
  `package-installer` job comment is corrected to match — it previously
  read as though the shipped installer consumed the checksums `dist` also
  generates; it does not.

- **AGENTS.md gains a session-start invariant and an estate/Git model table
  (estate-root proposal §14.1/§14.2).** A new "Session start" section
  states the exact-root rule up front — no upward search, no Git fallback,
  the same remedy the real root-gate diagnostic gives — and names which
  four commands work outside an estate; a new "Estate and Git model" table
  fixes the vocabulary (estate root, `repos/<name>`, the surfaces
  directory, `sergeant/<work-id>`) before the routing table uses it. A new
  "ESTATE — Captain's estate discipline" section (§14.3/§14.4) states
  Captain's pre-Work checklist and what a worker never does (edit a mount,
  create a replacement branch, navigate into another Work's surface,
  expand its own scope, invoke an estate-scoped command from its own
  surface). "Captain captains" lands as emphasis, per the owner's
  2026-08-20 ruling (gauntlet finding C1): Captain's normal mode is
  dispatching Work and shaping intent, not writing code turn by turn — the
  existing ROUTING table's in-session allowances (BU-0004/BU-0009) are
  unchanged, not narrowed. `CAN — enforceable authority` gains three
  bullets for behavior already enforced: the root gate, the Git preflight,
  and durable branch retention.

- **README.md and docs/glossary.md corrected off the exact-root contract.**
  The data-dir precedence description no longer describes upward directory
  search; `sgt run`'s documented flags drop the removed `--workspace` and
  add `--all`/`--override-git-preflight`; "the workspace's own
  `software-change` workflow" is corrected to "the estate's own."
  `docs/glossary.md` gains the estate-root proposal's seven §14.7 terms:
  Estate, Repository Mount, Work Scope, Repository Binding, Work Surface,
  Integrity Disposition, Estate Drift.

- **Embedded distro swept for exact-root skew (C12 and §14.6).**
  `.sergeant/workflows/dispatch/05-classify-risk/CONTEXT.md`'s restated
  `sgt run --help` option list drops the removed `--workspace` flag (the
  one instance C12 named, confirmed the only one by a full sweep of
  `AGENTS.md`, `skills/`, `.sergeant/common/contexts/`, and
  `.sergeant/workflows/`) and adds `--all`/`--override-git-preflight`/the
  global `-C`/`--data-dir`/`--json` flags it was missing. Six sites that
  described a stage actor
  dispatching nested Work or delivering an escalation response as its own
  literal `sgt run`/`sgt respond` invocation from inside its own Work
  surface — `implement/30-review`, `implement`'s own `CONTEXT.md`,
  `worker-mission/20-implement`, `worker-mission`'s own `CONTEXT.md`, and
  `dispatch/80-monitor` plus `dispatch`'s own `CONTEXT.md` — gain an
  explicit note that the worker's own Work surface is not an estate root
  and the command would refuse from there today; the actual submission is
  Captain's, from the estate root.

- **Embedded skills rewritten for the exact-root front door (§14.6).**
  `skills/sergeant-help` gains the loud root and preflight remedies it was
  silent on: documentation-map rows for "which directory must this command
  run from" and "why was my submission refused for a dirty or detached
  mount", failure-behavior rows repeating the refusals' own remedies
  verbatim (`cd <estate-root>`, `sgt -C <estate-root> <command>`, `sgt
  init`; `git -C <mount> status`/`switch <branch>`, with
  `--override-git-preflight` described as the per-submission waiver of a
  dirty or detached mount and nothing else), and a must-not bullet against
  routing around either refusal. `skills/estate-navigation` now teaches the
  exact-root check itself — look for this directory's own `./sergeant.toml`,
  never walk upward, `-C` to name a root without moving — and its `sgt
  doctor` description is brought current with the `estate_root`,
  `workflows`, and `git_surfaces` rows.

- **ADR 0008's estate-root amendment verified, not redone.** Phase D
  already amended it ("Amended by the estate-root integration (C7a,
  2026-08-20)"): the manifest keeps storage-path authority, discovery
  becomes exact-root, and R-MVP1-12 is marked superseded. Confirmed
  current against this phase's own doctrine rewrite; no further change
  needed for C7a.

## [0.1.0] - 2026-08-19

First release. `sgt` is an AgentOS distro: instructions, skills, and
workflow templates embedded in the binary and written to your estate by
`sgt init`, turning a general-purpose coding harness (Claude Code today)
into an operator of your estate, carried by a durable intent-execution
engine that runs those intents to completion in isolated worktrees.

### Added

- **`sgt init` scaffolds an estate and embeds the distro.** A fresh
  `sgt init` writes `sergeant.toml`, `repos/`, `.gitignore`, and now also
  the full embedded distro — `AGENTS.md`, `skills/`,
  `.sergeant/common/contexts/`, and 17 workflow packages under
  `.sergeant/workflows/` — per-file idempotent, so re-running `sgt init`
  against an existing estate is a no-op rather than an overwrite (#179).
- **Local-shadows-stock workflow resolution and `sgt workflow fork`.** A
  workflow package you author locally under `.sergeant/workflows/` takes
  precedence over a stock package of the same name shipped in the distro;
  `sgt workflow fork` copies a stock package into your estate as a starting
  point for editing. The shipped packages are examples and defaults, not
  published procedure you're expected to follow as-is (ADR 0014 decision 3).
- **`sgt doctor`** checks estate health with named faults and named
  remedies, including: the estate-local data directory now resolves and is
  reported correctly on a brand-new estate instead of falling back to the
  pre-estate XDG/HOME default (#164), and a fresh or workflow-less estate
  now gets an explicit `workflows` check reporting zero packages and
  naming `sgt init`/`sgt workflow fork` as the remedy, instead of a bare
  422 the next time you try to dispatch a named workflow (#165).
- **Release pipeline** (`.github/workflows/release.yml`): `workflow_dispatch`
  only, `dry-run` default, no tag/push/schedule trigger. Runs Gates A-F
  (repository-state, `ci.yml` reuse, `matrix.yml` reuse, `coverage.yml`
  reuse, strict `cargo deny check`, and a documented Gate F no-op — see
  Known gaps below) before packaging, smoke-testing, generating a SHA-256
  manifest and a per-target CycloneDX SBOM, generating GitHub
  build-provenance and SBOM attestations, assembling a draft GitHub
  Release, verifying its manifest, and — only in `mode: publish` —
  publishing it. `.github/workflows/ci.yml` gained a `workflow_call`
  trigger so Gate B reuses CI's exact contract rather than redefining
  "tests passed."
- **Supply-chain posture:** every third-party GitHub Action referenced
  across `ci.yml`, `matrix.yml`, `coverage.yml`, and `release.yml` is
  pinned to a commit SHA, not a mutable tag; `cargo-deny` enforces
  bans/licenses/sources on every PR and the full advisory-inclusive check
  at release time (Gate E); CodeQL default setup scans Rust and Actions
  weekly; GitHub's `dependency-review-action` diffs dependency manifests
  on every pull request; Dependabot runs weekly, grouped updates for both
  Cargo and GitHub Actions dependencies; and release artifacts ship with a
  CycloneDX SBOM per target plus GitHub build-provenance and SBOM
  attestations.
- **Packaging configuration** (`[workspace.metadata.dist]` and
  `[profile.dist]` in `Cargo.toml`): `dist` (cargo-dist 0.32.0) covers the
  two target triples ADR 0001 names as release targets,
  `x86_64-unknown-linux-gnu` and `aarch64-apple-darwin`.

### Platform boundary (ADR 0001, ADR 0018)

- **`x86_64-unknown-linux-gnu` is measured**: built, packaged, and
  smoke-tested (`sgt --version`) on a Linux host, with `probe-env.sh` and a
  published-skip-count full suite run backing the "measured" label.
- **`aarch64-apple-darwin` is built and packaged, but NOT validated.**
  `dist build` succeeds there on a real `macos-latest` runner, so it ships
  in this release — a clean build is what ADR 0018 obliges from an
  unmeasured platform — but binary-equivalence and generated-installer
  checks were only run on `ubuntu-latest`. Do not read "packaged" as
  "measured" for macOS in this release; that label is earned separately,
  on the owner's own schedule.

### Known gaps in this release pipeline (recorded, not hidden)

- **No curl-pipe-sh installer is published yet.** `dist`'s generated
  shell installer is not part of this release's artifact set.
- **Co-versioning is real for `sgt init`, not yet for update semantics.**
  ADR 0014 decision 2 names one artifact identity: the `sgt` binary
  together with the embedded distro it writes. As of this release
  `sgt init` does embed and write that distro (closing the gap #165
  originally only made visible), but per ADR 0014 decision 2 the distro
  ships embedded *in the binary* — there is no separate distro artifact or
  update channel yet; getting a newer distro means installing a newer
  `sgt`.
- **Gate F (distro structural validator) does not run in this repo's CI.**
  It lives in `sergeant-rs-workspace`'s
  `.sergeant/local/workflows/validate-distro/` by deliberate placement
  (ADR 0014 decision 5) and this repo's CI cannot reach it. `release.yml`'s
  `gate-f-distro-validator` job is a labelled skip, not a passing check.
  Gap filed as issue #176; that property is instead proven continuously by
  `sergeant-rs-workspace`'s own CI against this repo's `main`.
- **GitHub-hosted-runner disk/cache bounds for `dist build` are
  unverified.** The measured Linux build ran on a workstation with ~780 GB
  free; GitHub-hosted runners start with a much smaller free-disk budget,
  and `coverage.yml` already has to reclaim disk before its own DuckDB
  build. Whether `dist build`'s cold build fits GitHub's runner disk budget
  without the same reclaim step was not measured here.
