Ok here’s how it’s being reviewed:

Pipeline’s running in your exact shape: four Sonnet critics in parallel (dependency/completeness, design-vs-source-reality, Ponytail-both-directions, and honesty/vision — that last one explicitly briefed to walk the happy path step by step and name the first step that breaks, plus a sycophancy check for owner suggestions the evidence contradicts). Then the Opus writer applies verified findings and rewrites the documents in place — with the discipline that rejected findings get recorded in a dispositions section, never silently dropped, and genuine forks escalate instead of being picked quietly. Then a fresh Sonnet sanity pass over the final documents only

I’d also like you to do a review here as well.  Something for you and I to discuss.  I just want to make sure this whole thing makes sense.  Spent a lot of time solutioning but never answered the problem statement.  Which is what this final stretch is about.  Why we needed a North Star.  It informed what the last bit of feature and platform work is and how to get there.  So this is our independent review of the plan.  


---------- RESPONSE ----------

# Independent verdict

**The plan is directionally right, and the North Star did its job.** It converted a large collection of sound solutions into a coherent product sequence:

```text
establish the durable operating substrate
→ make native actors honest
→ expose the complete CLI loop
→ prove unattended operation
→ teach the harness and human how to use it
→ invite colleagues
```

That is substantially better than continuing to add engine features, adapter sophistication, workflows, dashboards, and agent abstractions without deciding what experience they are supposed to create.

I would approve the **direction**. I would not yet call the roadmap contract-ready. There are several concrete contradictions and one or two unresolved product boundaries that should be ruled before implementation starts.

## The problem statement is still implicit

The North Star opens with a destination:

> Sergeant is an AgentOS distro carried by a durable intent-execution engine.

That is a strong solution statement and ownership model, but it is still a solution statement.

My independent formulation of the actual problem is:

> **General-purpose coding harnesses are effective conversational actors, but they are poor durable colleagues. A developer cannot yet give one an estate-scoped outcome, leave, and trust that intent, repository context, procedure, execution limits, outputs, and evidence will remain coherent without continued human supervision.**

Or even more plainly:

> **The human still has to stay present to keep agent work from falling apart.**

The dogfood run validates exactly that problem:

* one run spent much of its time reconstructing an environment and produced no deliverable;
* one produced a valuable artifact, but Sergeant added little visible value because output, transcript, and cost were not usable;
* one workflow whose purpose depended on human interaction completed autonomously with no place for the human’s answer to land.

Sergeant’s product promise then becomes:

> **Let a developer delegate bounded work across an estate, walk away, and return to either a trustworthy result or an honest, recoverable stop—with the branch, transcript, outputs, evidence, and execution bounds intact.**

That sentence should appear before “AgentOS distro.” It gives every roadmap item a falsifiable reason to exist.

# Does the MVP map to that problem?

Mostly, yes.

| Problem obligation                      | MVP answer                                        | Assessment                                                  |
| --------------------------------------- | ------------------------------------------------- | ----------------------------------------------------------- |
| Identify the developer’s working estate | manifest, `sgt init`, repo/group commands         | Correct direction; ownership and migration need rulings     |
| Turn conversation into bounded intent   | AGENTS.md, operator skills, structured intent     | Correct, but the “schema slot” is underdefined              |
| Execute safely without babysitting      | Work, worktrees, turn envelope, adapter contracts | Strong                                                      |
| Verify independently                    | Docker execute stages                             | Correctly retained in MVP                                   |
| Survive absence and failure             | journal, recovery, blocked exit test, soak        | Strong, provided the final soak matches the actual promise  |
| Return a usable result                  | output pointer, promote/finalize, transcript      | Directly supported by dogfood evidence                      |
| Avoid unbounded execution               | turn envelope plus adapter-reported cost          | Honest correction to the earlier dollar-cost promise        |
| Work across several repositories        | existing multi-repo surface plus estate groups    | Central value claim; instruction projection remains unclear |

The MVP is no longer an arbitrary feature pile. Nearly everything in it traces to the delegation problem or to a measured defect that prevents the delegation loop from being trusted. The post-MVP deferral of queued intents, full TUI, global Journal exploration, scheduler, callbacks, binaries, and platform breadth is mostly disciplined.

# Walking the happy path

I walked the stated path literally and found three different “first breaks.”

## 1. The first mechanical break: installation

The plan says:

```text
gh repo clone
→ cargo install sergeant
→ sgt init
```

The current Cargo package is named `sergeant-rs`, with `sgt` as its binary. The MVP buckets do not include publishing a crate under the name `sergeant`, and the release pipeline/prebuilt binary remains post-MVP.

The current cold source build also compiles bundled DuckDB and is documented as taking roughly ten minutes, so it cannot satisfy the North Star’s eventual “stranger reaches the end in under five minutes” acceptance path. That is okay because stranger onboarding is gated—but the roadmap currently mixes that eventual path with the colleague MVP path.

There should be two explicit acceptance personas:

### MVP colleague path

```text
gh repo clone miztertea/sergeant-rs
cd sergeant-rs
cargo install --path . --bin sgt
sgt init
```

A cold build longer than five minutes is acceptable if stated.

### Later stranger path

```text
install packaged sgt
sgt init
```

Under five minutes, no Rust toolchain or DuckDB compilation required.

That distinction is already conceptually present in the North Star, but the happy-path command needs to stop pretending the post-MVP distribution surface exists during MVP.

## 2. The first product break: how repository instructions reach the actor

The path says:

```text
actors work in worktrees
consuming each repo’s own AGENTS.md
(manifest policy)
```

This is load-bearing. It is how the general-purpose harness becomes an operator of the actual estate instead of a generic model dropped into a directory.

The current engine can already materialize multiple repositories. For one repository, execution runs in that worktree; for multiple repositories, execution runs at the shared surface root containing the child worktrees.

But the MVP plan does not yet state the complete instruction projection contract:

* Which instruction files are selected?
* Does each repository contribute `AGENTS.md`, `CLAUDE.md`, both, or a manifest-selected file?
* In what order are instructions composed for multi-repository Work?
* How are conflicting instructions handled?
* Is the instruction policy pinned at bind so a later file edit cannot rewrite a running Work?
* What does Claude receive through `--setting-sources`?
* What will another adapter receive?
* Does an actor running at the multi-repo surface root naturally discover any child repository’s instructions, or must Sergeant explicitly project them?

The plan mentions a manifest `instructions` policy and the Claude `--setting-sources` boundary, but this needs to become a first-class MVP acceptance item—not an adapter implementation detail.

The rule should probably be:

> **The manifest declares instruction-selection policy; the core resolves and pins the selected instruction identities at bind; each adapter translates that pinned policy into its native mechanism without redefining it.**

That matches the North Star’s ownership boundary: the estate owns repository instructions, the core owns the pinned execution contract, and adapters only translate.

Until this is explicit, the estate claim is not actually proven.

## 3. The first architectural break: self-hosting with the in-estate data directory

This is the most concrete source-level contradiction I found.

The roadmap moves the default data directory into:

```text
<estate>/.sergeant/data/
```

The current surface implementation puts worktrees under:

```text
<data-dir>/surfaces/<work-id>/<repo>
```

It also deliberately rejects a surface located inside the repository it is meant to modify.

For ordinary estate members, this is fine:

```text
estate/.sergeant/data/surfaces/...     # worktree
estate/repos/payments-api              # source repo
```

The paths are siblings.

But self-hosting Sergeant against the estate root produces:

```text
sergeant-rs/.sergeant/data/surfaces/...  # worktree
sergeant-rs/                              # source repo
```

The worktree is inside its own source checkout, so the current safety check must refuse it.

That directly conflicts with:

* clone-is-distro;
* the estate-local data-dir ruling;
* the recursion/self-hosting proof;
* the idea that Sergeant development itself runs through Sergeant.

This needs a design ruling before the data-dir flip lands. The realistic options are:

1. **Separate `data_dir` from `surfaces_root`.** Keep journal/blobs/projections in the estate, but place disposable worktrees in an external or sibling runtime directory.
2. **Declare the estate root untargetable.** Self-host through a separate registered clone or worktree under `repos/`.
3. **Retain an external data directory until clean-distro extraction.**

Option 1 best preserves the North Star, but it is a real design decision, not a CLI detail.

# Five findings I would settle before contracts

## Finding 1 — the manifest has three unresolved identities

The plan proposes:

```toml
[estate]
[[repo]]
[group]
```

The current checked-in `sergeant.toml` parser expects:

```toml
[workspace]
[[repository]]
[[profile]]
```

and deliberately rejects unknown fields. It already supports multiple repositories and `--repo` selection.

The new plan therefore needs to decide:

### Schema migration

Is the estate format an extension, a replacement, or a second file?

A hidden rename from `[workspace]` to `[estate]` will break every current fixture, example, and repository config. That may be acceptable before external release, but it should be explicit and mechanically migrated.

### Ownership

The current file is checked-in declarative topology. The new CLI intends to mutate it through:

```text
sgt repo add/remove
sgt group add/remove
```

That changes it from human-authored configuration to a jointly edited control surface.

### Portability

Does the manifest record:

* expected logical repositories and relative mount names, which are portable and tracked; or
* machine-specific paths and clone state, which should be gitignored?

The phrase “harness interviews, writes the manifest, `sgt repo add` populates the working set” is particularly dangerous. The harness should **not** write the file and then ask Sergeant to update it. The harness should gather the answer and drive `sgt repo add` or `sgt group add`; Sergeant should be the one writer applying validation, locking, and atomic replacement.

A clean boundary could be:

```text
tracked manifest:
  logical repo name
  expected relative mount
  source URL where appropriate
  group membership
  instruction policy

gitignored local state:
  actual resolved path
  clone/worktree/symlink kind
  health and synchronization observations
```

That may or may not be the lowest rung, but the ownership distinction has to be settled.

## Finding 2 — “structured-intent schema slot” is too vague

This item is at risk in both Ponytail directions.

It could be overbuilt into a generalized future WorkPacket schema nobody uses during MVP.

It could also be underbuilt as an untyped `serde_json::Value` placeholder that permanently avoids deciding what intent means.

The happy path already requires the harness to “shape intent,” so this is not merely post-MVP scaffolding. Define the smallest fields the MVP genuinely uses, perhaps:

```text
objective
repositories or group
acceptance
exclusions
workflow
```

Keep the existing free-text intent legal. Let structured fields be optional progressive elaboration. Show them in `work show`, journal them with the Work, and pin them before execution.

That earns the schema now without building queued intents.

## Finding 3 — the fake backend correction is sequenced too late

MVP-1 adds:

* the turn-count envelope;
* blocked exit-door behavior;
* promote/finalize semantics;
* multi-repo binding;
* terminal eviction.

MVP-2 then fixes fake-backend deferred-finish and timing fidelity.

That test instrument previously allowed hundreds of green tests to coexist with a real 45-minute execution stall because fake turns settled too early. The plan itself says the fidelity work is required for MVP-4’s results to be believable.

I would not move the entire adapter milestone ahead of Core. I would move the **minimum fake lifecycle/timing correction** to the start of MVP-1 or make it a precondition for any core contract whose behavior depends on a turn boundary.

Otherwise the first milestone builds new safety semantics with an instrument already known to misrepresent the relevant lifecycle.

## Finding 4 — `promote`/finalize is the remaining genuine design fork

The output pointer is straightforward:

* the branch already survives;
* Work should state the branch, repository, and declared output location;
* the human should not decode teardown events to find it.

The current surface implementation explicitly retains the branch while removing disposable worktrees.

But “`promote`/finalize disposition executes” can mean several materially different things:

1. the engine understands output declarations and performs disposition;
2. a deterministic execute stage runs a finalize helper;
3. a closing actor stage invokes the existing helper;
4. Sergeant merely reports declared disposition and branch location.

Those differ in engine vocabulary, event model, N4 dependency, failure behavior, and who owns correctness.

The roadmap puts disposition execution in MVP-1 Core while putting Docker execute stages in MVP-2. It also says terminal eviction “rides N4” while listing it under MVP-1. Those phrases reveal that the dependency boundary is not fully settled.

Before an MVP-1 contract, define:

> What exact inputs declare an output, what `promote` and `evidence` cause, which component performs the operation, when it occurs relative to terminal state and teardown, and what happens when it fails.

This is the one place where a wrong shortcut could create a second artifact system or bake workflow convention into the engine prematurely.

## Finding 5 — stabilization occurs before the most behaviorally important layer

The sequence is:

```text
MVP-4 stabilize
→ MVP-5 content
→ send to colleagues
```

But MVP-5 changes:

* the canonical AGENTS.md;
* the harness routing loop;
* operator skills;
* worker bundles;
* workflow library dispositions;
* the intent-elaboration behavior;
* the walkthrough.

That content is the user-facing operating system. It can break the product even when every Rust test is green. The dogfood run already demonstrated that procedure content can turn a sound engine into an unusable product: one workflow silently lost its defining human checkpoint, and another added ceremony without visible value.

MVP-5 therefore needs a **final assembled-product ship gate**, not merely a demo script.

The gate should start from a fresh checkout and exercise the literal colleague path:

```text
install by the documented MVP method
sgt init
register at least two repositories
open a fresh Claude context
say “I’d like to work on the payments API”
let AGENTS.md drive the interview and CLI
shape and submit the intent
run an actor stage
run deterministic Docker verification
detach the client
exercise restart or absence
return through status/work show/transcript
find the retained branch and outputs
verify turn-envelope behavior
```

No hand-editing the manifest.
No manual journal decoding.
No undocumented profile knowledge.
No orchestrator rescuing the path.

That gate is where the roadmap finally answers the original problem rather than validating its component solutions.

# The self-hosting proof needs to remain in MVP

The North Star’s argument record required the first relevant fix to run as Sergeant Work, because self-hosting is evidence rather than branding. The revised MVP note instead calls P2-JOURNAL the “first self-hosted milestone.”

Those statements can coexist only if individual MVP changes are self-hosted while P2 is the first completely self-hosted milestone. The roadmap should say that explicitly.

I would require:

* at least one MVP-1 change executed through current Sergeant;
* at least one MVP-3 CLI/content integration run through the assembled version;
* MVP-4’s real soak launched and inspected through documented Sergeant surfaces;
* the final MVP-5 happy-path gate run from a fresh harness context.

Otherwise the plan ships to colleagues before closing the North Star’s own “recursion proof” gap.

# What I think the plan gets exactly right

## Full TUI is post-MVP

Despite all the excellent TUI design work, this is the correct ruling.

The CLI and harness path is already first-class. A TUI can dramatically improve usability, but it should not mask missing outputs, transcript, estate setup, instruction projection, execution bounds, or soak evidence. The North Star’s rule that surfaces add usability rather than functionality is the right one.

## Global Journal exploration is post-MVP

Also correct. `sgt work transcript` is required for the MVP because it closes an immediate human legibility defect. P2-JOURNAL is a richer historical and analytical capability. Splitting them keeps the MVP honest.

## Docker remains MVP

I agree with keeping it. Without a deterministic verification stage, “walk away and return to finished work” remains “trust the same probabilistic actor that made the change to declare itself done.” Docker execute stages make verification a separate, inspectable operation.

## Cost is no longer overpromised

The plan correctly separates:

```text
core-owned turn envelope
adapter-reported usage and cost
```

A turn limit can be enforced before launch. A dollar limit based on usage reported after a turn can only stop the next turn and may overshoot by the largest single turn. Calling that an adapter capability rather than universal hard cost control is honest.

## Estate and cross-repository work remain MVP

Also correct. Removing estate/multi-repo work would make the MVP easier, but it would reduce Sergeant to a durable single-repo harness runner. That is not the North Star.

# One post-MVP choice is not evidence-settled

The plan hard-orders:

```text
P2-JOURNAL
→ T-series minimal slice
```

I do not think current evidence has earned that ordering yet.

The case for P2 first is good:

* exercises the data plane;
* becomes a self-hosted milestone;
* provides the million-event measurement;
* advances retention Rule C;
* compounds future analysis.

The case for T-minimal first is also good:

* colleagues will interact with current Work far more often than global history;
* attention, thread legibility, and responding are direct operator pain;
* the T-minimal slice is likely smaller;
* colleague use may immediately expose it as the adoption bottleneck.

The dogfood evidence directly demanded transcript legibility, not global journal search. It also did not exercise the proposed TUI. So I would make this a **post-MVP pilot ruling**, not freeze it now:

```text
colleague pilot says current-operation usability is the bottleneck
    → T-minimal first

colleague pilot says historical investigation and evidence discovery block use
or 1M measurement/retention trigger becomes urgent
    → P2 first
```

This is a genuine fork, not a current blocker.

# My bottom line

The roadmap now has a coherent answer:

> **Build the smallest complete delegation loop first. Do not optimize the interface, historical intelligence, automation queue, distribution, or platform breadth until a colleague can give Sergeant real estate-scoped work, leave, and return to a bounded, inspectable result.**

I think that is the right plan.

Before converting it into milestone contracts, I would require five rulings:

1. **Write the non-product problem statement explicitly.**
2. **Separate the colleague MVP install path from the later stranger path.**
3. **Resolve estate-local data versus self-hosted work surfaces.**
4. **Settle manifest ownership, schema migration, repository-add semantics, and per-repository instruction projection.**
5. **Add a fresh-clone, post-content, assembled-product ship gate.**

Then clarify promote/finalize ownership and pull minimum fake timing fidelity ahead of the core tests that depend on it.

With those settled, I would be comfortable saying the plan is no longer solutioning in search of a problem. It becomes a direct program for proving one claim:

> **A developer can hand Sergeant meaningful work and stop babysitting it.**

