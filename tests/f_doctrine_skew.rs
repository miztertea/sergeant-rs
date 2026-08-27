//! Phase F doctrine-skew tests (estate-root proposal §18's "Doctrine skew
//! tests" row, spec `phase-f.md` deliverable 4).
//!
//! No `validate-skew` tool or similar existed anywhere under `scripts/` at
//! the time this landed (checked by hand — grep found nothing), so this is
//! new, focused tooling rather than an extension of something existing.
//!
//! Three claims, each checked against the real thing rather than trusted:
//!
//! 1. `AGENTS.md`'s root-gate claims ("Session start") agree with the real
//!    binary's `--help` surface and its actual root-gate refusal text.
//! 2. No `CONTEXT.md` under the embedded `.sergeant/workflows/` distro
//!    source instructs a stage actor to invoke an estate-scoped `sgt`
//!    command as a literal action from inside its own Work surface, without
//!    a disclaiming note nearby (engine gap, Captain-side action, upstream
//!    "sgt-dispatch" provenance, etc).
//! 3. The estate-root proposal's own canonical manifest example (§13.1)
//!    still parses under the current manifest schema.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use sergeant_rs::domain::estate::Estate;

mod support;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Run `sgt` with `cwd` and `args`, no `--data-dir` — every check here
/// either never touches a data dir (`--help`/`--version`) or is refused
/// before one would be resolved (an estate-scoped command outside an
/// estate; §4.3's ordering).
fn run(cwd: &Path, args: &[&str]) -> Output {
    Command::new(SGT)
        .args(args)
        .current_dir(cwd)
        // Isolate from the real environment's own estate/data-dir state —
        // this suite asserts root-gate behavior, not this host's.
        .env_remove("SGT_DATA_DIR")
        .output()
        .expect("run sgt")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

// ------------------------------------------------------- 1. root-gate skew

/// A directory with no estate anywhere above it — a fresh subdirectory of
/// `TempDir`'s own root, which is never itself an estate in this suite.
fn bare_dir() -> tempfile::TempDir {
    tempfile::TempDir::new().expect("bare temp dir")
}

/// AGENTS.md's "Session start" section names exactly the unscoped-plus-
/// host-scoped command set `is_estate_scoped`/`is_host_scoped`
/// (`src/cli.rs`) compute, and the real refusal an estate-scoped command
/// gives outside an estate carries the exact wording AGENTS.md quotes for
/// it.
#[test]
fn agents_md_session_start_matches_the_real_root_gate() {
    let agents_md = std::fs::read_to_string(repo_root().join("AGENTS.md")).expect("read AGENTS.md");
    // Markdown prose in this file soft-wraps; collapse whitespace so a
    // sentence spanning a line break still matches a `contains` check on
    // its logical text rather than its accidental line layout.
    let agents_md_flat = agents_md.split_whitespace().collect::<Vec<_>>().join(" ");

    // The claim, quoted verbatim from `is_estate_scoped`/`is_host_scoped`'s
    // own doc comments (H1 sprint plan, W3 brief deliverable 2) in
    // AGENTS.md's own voice.
    assert!(
        agents_md_flat.contains(
            "`sgt --help`, `--version`, `sgt init`, `sgt doctor`, and the host-scoped bucket \
             — `sgt tui`, `sgt status`, `sgt work show`/`list`/`transcript`, `sgt watch`, and \
             every `sgt daemon` verb — all work outside an estate too"
        ),
        "AGENTS.md's Session start section no longer states the unscoped-plus-host-scoped \
         command set — update this test and the claim together, from is_estate_scoped's and \
         is_host_scoped's own doc comments"
    );

    // The refusal wording AGENTS.md quotes must be the real one
    // (`EstateRootError`'s `Display`, `src/domain/estate.rs`), not a
    // paraphrase that could drift from it.
    assert!(
        agents_md_flat.contains("Sergeant does not search parent directories for an estate"),
        "AGENTS.md must quote the real root-gate diagnostic wording"
    );

    let dir = bare_dir();

    // The four fully unscoped commands AGENTS.md claims work outside an
    // estate must actually run there — none of them the root-gate refusal.
    for args in [
        vec!["--help"],
        vec!["--version"],
        vec!["doctor"],
        vec!["init"],
    ] {
        let output = run(dir.path(), &args);
        assert!(
            !stderr(&output).contains("does not search parent directories"),
            "{args:?} must not hit the root gate outside an estate: {}",
            stderr(&output)
        );
    }

    // `sgt init` above already turned `dir` into an estate — re-run the
    // remaining checks from a second, still-bare directory so they test the
    // "no estate at all" case, not "estate exists but I'm not at its root".
    // `--data-dir` names a fresh, guaranteed-empty directory for every host-
    // scoped invocation below: none of them may hit the root gate, but
    // without an isolated data dir they could instead attach to whatever
    // real host daemon this machine happens to be running — the wrong
    // failure mode for a test that means to pin "no root gate", not "no
    // daemon either".
    let refused = bare_dir();

    // H1 §5 / brief deliverable 2: the host-scoped bucket. Each of these
    // must reach past the root gate — refusing, if at all, only on "no
    // daemon is running", never on "does not search parent directories".
    for args in [vec!["status"], vec!["work", "list"], vec!["watch"]] {
        let host_data_dir = bare_dir();
        let mut full = vec![
            "--data-dir".to_string(),
            host_data_dir.path().display().to_string(),
        ];
        full.extend(args.iter().map(|s| s.to_string()));
        let full: Vec<&str> = full.iter().map(String::as_str).collect();
        let output = run(refused.path(), &full);
        assert!(
            !output.status.success(),
            "{args:?} must refuse (no daemon at a fresh data dir): {}",
            stderr(&output)
        );
        assert!(
            !stderr(&output).contains("does not search parent directories for an estate"),
            "{args:?} is host-scoped and must never hit the root gate outside an estate: {}",
            stderr(&output)
        );
        assert!(
            stderr(&output).contains("no daemon is running for"),
            "{args:?} must refuse for the host-scoped reason (no daemon), not something else: {}",
            stderr(&output)
        );
    }

    // Estate-scoped verbs (H1 §11.3, unchanged) still refuse with the
    // root-gate diagnostic AGENTS.md quotes.
    for args in [vec!["analytics"], vec!["repo", "list"]] {
        let output = run(refused.path(), &args);
        assert!(
            !output.status.success(),
            "{args:?} must refuse outside an estate"
        );
        assert!(
            stderr(&output).contains("does not search parent directories for an estate"),
            "{args:?}'s refusal must carry the diagnostic AGENTS.md quotes: {}",
            stderr(&output)
        );
    }
}

/// AGENTS.md's estate/Git model table names `sgt -C <estate-root>` as the
/// way to address an estate without `cd`-ing to it (C10) — confirm that
/// actually works, from a directory that is not itself any estate.
#[test]
fn agents_md_dash_c_flag_actually_names_the_estate() {
    let agents_md = std::fs::read_to_string(repo_root().join("AGENTS.md")).expect("read AGENTS.md");
    assert!(
        agents_md.contains("sgt -C <estate-root>"),
        "AGENTS.md's Session start section must still name `-C` as the non-chdir remedy"
    );

    let estate = tempfile::TempDir::new().expect("estate dir");
    support::scaffold_solo_estate(estate.path(), "solo");
    let elsewhere = bare_dir();

    let output = run(
        elsewhere.path(),
        &["-C", &estate.path().display().to_string(), "doctor"],
    );
    assert!(
        stdout(&output).contains("is an estate root"),
        "sgt -C <estate-root> doctor must recognize the named root from elsewhere: {}",
        stdout(&output)
    );
}

/// The embedded operator skills the harness loads directly (`skills/`)
/// carry the same root-gate and Git-preflight remedies the binary actually
/// prints (§14.6's "loud root/preflight remedies"), and `estate-navigation`
/// teaches the exact-root check rather than an upward search — each claim
/// cross-checked against the real refusal, the real `--help`, or the real
/// remedy string, never against a paraphrase of it.
#[test]
fn embedded_skills_carry_the_real_root_and_preflight_remedies() {
    let help_skill = std::fs::read_to_string(repo_root().join("skills/sergeant-help/SKILL.md"))
        .expect("read skills/sergeant-help/SKILL.md");
    let navigation_skill =
        std::fs::read_to_string(repo_root().join("skills/estate-navigation/SKILL.md"))
            .expect("read skills/estate-navigation/SKILL.md");

    // 1. The root-gate refusal a real estate-scoped command gives outside an
    //    estate — every remedy it names must be one `sergeant-help` names
    //    too. `analytics`, not `status`: H1 (sprint plan D6, brief
    //    deliverable 2) moved `status` into the host-scoped bucket, so it no
    //    longer hits this gate at all — `analytics` stays estate-scoped
    //    (H1 §11.3).
    let refused = bare_dir();
    let refusal = stderr(&run(refused.path(), &["analytics"]));
    for quoted in [
        "no estate found in",
        "does not search parent directories",
        "cd <estate-root>",
        "sgt init",
    ] {
        assert!(
            refusal.contains(quoted),
            "the real root-gate refusal no longer contains {quoted:?} — update the skill and \
             this test together: {refusal}"
        );
        assert!(
            help_skill.contains(quoted),
            "sergeant-help must repeat the real root-gate remedy {quoted:?}"
        );
    }
    // The descendant half of the same gate, and `-C`: both live in the
    // binary rather than in that one refusal's text.
    assert!(
        help_skill.contains("this command must be run from the estate root"),
        "sergeant-help must quote the descendant refusal's own wording"
    );
    assert!(
        stdout(&run(refused.path(), &["--help"])).contains("-C <ESTATE_ROOT>"),
        "`sgt --help` no longer documents the global -C flag the skills route to"
    );
    for skill in [&help_skill, &navigation_skill] {
        assert!(
            skill.contains("sgt -C <estate-root>"),
            "both skills must name `-C` as the remedy that needs no `cd`"
        );
    }

    // 2. The Git preflight: the flag exists on `sgt run`, and the mount
    //    remedies the skill quotes are the ones preflight actually emits.
    assert!(
        stdout(&run(refused.path(), &["run", "--help"])).contains("--override-git-preflight"),
        "`sgt run --help` no longer carries the override flag the skills describe"
    );
    // Rust string literals in that file are line-continued mid-sentence, so
    // drop the `\` continuations and collapse whitespace: match the remedy's
    // logical text rather than its accidental line layout.
    let preflight_src = std::fs::read_to_string(repo_root().join("src/runtime/preflight.rs"))
        .expect("read src/runtime/preflight.rs")
        .replace("\\\n", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for remedy in ["git -C <mount> status", "git -C <mount> switch <branch>"] {
        assert!(
            preflight_src.contains(remedy),
            "preflight no longer emits the remedy {remedy:?} sergeant-help quotes"
        );
        assert!(
            help_skill.contains(remedy),
            "sergeant-help must quote preflight's own remedy {remedy:?}"
        );
    }

    // 3. `estate-navigation` teaches the exact-root check, not an upward one.
    assert!(
        navigation_skill.contains("./sergeant.toml"),
        "estate-navigation must teach checking this directory's own ./sergeant.toml"
    );
    assert!(
        navigation_skill.contains("never walk upward"),
        "estate-navigation must forbid an upward search outright"
    );
}

/// #232: a literal-reading actor that finished a correct fix retired
/// `completed_dirty` because no shared closing-stage context ever told it
/// to commit — the `BindingSummary` names `work_branch`, but nothing said
/// to land the work there. `@@fix-confirmed` is the stage context that
/// instructs an actor to *do* fix work (as opposed to `@@close` /
/// `@@evidence-requirements`, which only report on fixes already made),
/// so it is the one this test pins the commit imperative in.
#[test]
fn fix_confirmed_context_states_the_commit_imperative() {
    let fix_confirmed =
        std::fs::read_to_string(repo_root().join(".sergeant/common/contexts/fix-confirmed.md"))
            .expect("read fix-confirmed.md");
    assert!(
        fix_confirmed.contains("git commit") && fix_confirmed.contains("git add"),
        "fix-confirmed.md must literally instruct the actor to `git add`/`git commit` its \
         fix — the gap #232 found, where a literal-reading actor finishes a correct fix and \
         retires it uncommitted because nothing ever said to commit"
    );
    assert!(
        fix_confirmed.contains("work_branch"),
        "fix-confirmed.md's commit imperative must name `work_branch` — the field the \
         BindingSummary already hands every actor — so the instruction says *where* to commit, \
         not just that a commit must happen"
    );
}

/// W1's engine-level nesting (host-atlas r3 ratification ruling 2, J3)
/// makes icm-policy.md §4 rule 1's "true nested workflows do not exist
/// yet" clause stale: the rule's actual point — a `@@name` reference is
/// context composition, never workflow composition, and must not be read
/// as "run this as a sub-workflow" — still holds and must not be
/// softened, but the dead-end "engine-gap claim" pointer is replaced with
/// the two real primitives W1 ships (a stage directory carrying its own
/// `workflow.toml` for engine-level recursion; child Work for a
/// separately-durable need). This is a net-new pin — no test covered this
/// text before S2 (confirmed by grep for "icm-policy" and "nested
/// workflows do not exist" over `tests/`, both empty).
///
/// Written first against the pre-amendment text (the stale "do not exist
/// yet; ... engine-gap claim" clause) to confirm it fails on that exact
/// wording, then against the amended text to confirm it passes.
#[test]
fn icm_policy_rule_one_points_to_the_real_nesting_primitives() {
    let icm_policy =
        std::fs::read_to_string(repo_root().join(".sergeant/common/contexts/icm-policy.md"))
            .expect("read icm-policy.md");
    let icm_policy_flat = icm_policy.split_whitespace().collect::<Vec<_>>().join(" ");

    // The core prohibition — unsoftened — still stands.
    assert!(
        icm_policy_flat.contains(
            "A `@@name` reference used to imply \"and then run that other procedure as a \
             sub-workflow\" is a violation of scope"
        ),
        "icm-policy.md §4 rule 1 must still forbid reading `@@name` as workflow composition — \
         W1 replaces only the stale 'do not exist yet' clause, not this prohibition"
    );

    // The amended clause names both real primitives.
    assert!(
        icm_policy_flat.contains("its own `workflow.toml`"),
        "icm-policy.md §4 rule 1 must point an author who wants real nested execution at a \
         stage directory carrying its own `workflow.toml`"
    );
    assert!(
        icm_policy_flat.contains("child Work"),
        "icm-policy.md §4 rule 1 must name child Work as the separately-durable alternative to \
         a `@@name` reference"
    );

    // Regression guard: the stale claim is gone, not merely amended-around.
    assert!(
        !icm_policy_flat.contains("do not exist yet")
            && !icm_policy_flat.contains("engine-gap claim"),
        "icm-policy.md §4 rule 1 must not still carry the stale 'true nested workflows do not \
         exist yet' / 'engine-gap claim' text now that W1 ships the real primitives"
    );
}

// -------------------------------------------------- 2. embedded distro skew

/// No `CONTEXT.md` under the embedded `.sergeant/workflows/` distro source
/// instructs a stage actor to invoke an estate-scoped `sgt` command as a
/// literal present-tense action from inside its own Work surface, without a
/// disclaiming note nearby.
///
/// Heuristic, not a parser: flags a paragraph naming an estate-scoped verb
/// cue (`sgt run`, `sgt respond`, …) unless a disclaiming term (engine gap,
/// Captain, context composition, estate root, the disclaimed upstream
/// "sgt-dispatch" name, …) appears in that paragraph or an adjacent one —
/// prose in this corpus routinely puts the caveat in the very next
/// paragraph rather than the same one. A false negative here is possible;
/// a false positive on real, already-disclaimed content is not, because
/// every current file is asserted to pass below.
#[test]
fn no_embedded_workflow_instructs_a_stage_actor_to_run_estate_scoped_commands_from_its_surface() {
    const VERB_CUES: &[&str] = &[
        "sgt run",
        "sgt respond",
        "sgt retry",
        "sgt extend",
        "sgt cancel",
        "sgt watch",
        "sgt status",
        "sgt work ",
        "sgt repo ",
        "sgt group ",
        "sgt daemon",
        "sgt tui",
        "sgt analytics",
    ];
    const DISCLAIMERS: &[&str] = &[
        "engine gap",
        "engine-gap",
        "captain",
        "context composition",
        "estate root",
        "estate-root",
        "sgt-dispatch",
        "upstream",
        "disclaim",
    ];

    let workflows_root = repo_root().join(".sergeant/workflows");
    let mut findings = Vec::new();

    for entry in walk(&workflows_root) {
        if entry.file_name().and_then(|n| n.to_str()) != Some("CONTEXT.md") {
            continue;
        }
        let text = std::fs::read_to_string(&entry).expect("read CONTEXT.md");
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        for (i, para) in paragraphs.iter().enumerate() {
            let low = para.to_lowercase();
            if !VERB_CUES.iter().any(|v| low.contains(v)) {
                continue;
            }
            let window_start = i.saturating_sub(1);
            let window_end = (i + 2).min(paragraphs.len());
            let window = paragraphs[window_start..window_end]
                .join(" ")
                .to_lowercase();
            if !DISCLAIMERS.iter().any(|d| window.contains(d)) {
                findings.push(format!(
                    "{}: paragraph #{i} names an estate-scoped verb with no disclaimer \
                     nearby: {:.200}",
                    entry.display(),
                    para.trim()
                ));
            }
        }
    }

    assert!(
        findings.is_empty(),
        "undisclaimed estate-scoped command instruction(s) in the embedded distro:\n{}",
        findings.join("\n")
    );
}

/// Recursively list files under `dir` (no external walker dependency —
/// R2/R3: the tree is small and this is a handful of lines).
fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out
}

/// C12 regression pin: the one known `--workspace` site the phase-f spec
/// named by path stays purged, and no sibling reintroduces it.
#[test]
fn no_shipped_workflow_or_skill_quotes_the_removed_workspace_flag() {
    let mut offenders = Vec::new();
    for root in [".sergeant/workflows", "skills", ".sergeant/common/contexts"] {
        for path in walk(&repo_root().join(root)) {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if text.contains("--workspace") {
                offenders.push(path.display().to_string());
            }
        }
    }
    let agents_md = repo_root().join("AGENTS.md");
    if std::fs::read_to_string(&agents_md)
        .map(|t| t.contains("--workspace"))
        .unwrap_or(false)
    {
        offenders.push(agents_md.display().to_string());
    }
    assert!(
        offenders.is_empty(),
        "shipped estate content still quotes the removed --workspace flag: {offenders:?}"
    );
}

// ------------------------------------------------- 3. manifest example skew

/// §13.1's canonical manifest shape, carried verbatim from
/// `sergeant-rs-workspace's knowledge/evidence/reference/estate-root-git.md` §13.1 as of 2026-08-24, ahead of
/// that file's relocation to the workspace knowledge library
/// (split-hardening series, sprint-plan-2026-08-24.md, W1). Fixture, not a
/// live read, so this test no longer depends on the doc surviving in this
/// repo.
const PROPOSAL_MANIFEST_EXAMPLE: &str = r#"[estate]
name = "payments"
data_dir = ".sergeant/data"
surfaces_dir = ".sergeant/data/surfaces"

[[repo]]
name = "payments-api"
origin = "git@github.com:company/payments-api.git"
instructions = "suppress"

[[repo]]
name = "auth"
origin = "git@github.com:company/auth.git"
instructions = "suppress"

[[repo]]
name = "payments-knowledge"
origin = "git@github.com:company/payments-knowledge.git"
instructions = "suppress"

[group.payments]
repos = ["payments-api", "auth", "payments-knowledge"]
brief = "Payment authorization, settlement, and governing team knowledge."

[[profile]]
name = "sonnet"
backend = "claude"
default_model = "sonnet""#;

/// The estate-root proposal's own canonical manifest shape (§13.1) still
/// parses under the current schema — structurally
/// (`Estate::from_config_structural`, which validates
/// names/duplicates/groups/profiles without requiring `repos/<name>` to be
/// real git checkouts on disk, exactly right for a doc example that names
/// no real repository).
#[test]
fn the_proposals_canonical_manifest_example_parses_under_the_current_schema() {
    let toml_text = PROPOSAL_MANIFEST_EXAMPLE;

    let dir = tempfile::TempDir::new().expect("scratch dir");
    let manifest_path = dir.path().join("sergeant.toml");
    std::fs::write(&manifest_path, toml_text).expect("write manifest");

    let estate = Estate::from_config_structural(&manifest_path)
        .unwrap_or_else(|e| panic!("§13.1's example manifest no longer parses: {e}"));
    assert_eq!(estate.name, "payments");
    assert_eq!(estate.repositories.len(), 3);
    assert!(estate.groups.contains_key("payments"));
    assert_eq!(estate.profiles.len(), 1);
}

// --------------------------------------- 4. close-out completion boundary

/// The `60-close-out` stage's Completion boundary section (#124) states the
/// same terminality applies to any external pipeline run the stage drove:
/// completion requires that run to reach a terminal disposition, or the
/// handover log to explicitly record it was deliberately left open and why.
/// Quoted verbatim so this test tracks the doctrine text, not a paraphrase
/// of it.
#[test]
fn close_out_completion_boundary_covers_external_pipeline_runs() {
    let context_md = std::fs::read_to_string(
        repo_root().join(".sergeant/workflows/validate-and-ship/60-close-out/CONTEXT.md"),
    )
    .expect("read 60-close-out/CONTEXT.md");

    let anchor = context_md
        .find("### Completion boundary")
        .expect("60-close-out/CONTEXT.md must still have a '### Completion boundary' section");
    let section = &context_md[anchor..];

    assert!(
        section.contains(
            "The same terminality binds any external pipeline run this stage drove: \
             completion requires that run to have reached a terminal disposition, or \
             the handover log to explicitly record that it was deliberately left open \
             and why — an untracked open run is silence by another name."
        ),
        "60-close-out/CONTEXT.md's Completion boundary section no longer states the \
         external-pipeline-run terminality clause (#124) — update this test and the \
         doctrine text together"
    );
}

// ------------------------------------------------- 5. dispatch doctrine (#166/#167)
//
// `classify_risk_states_the_real_intent_file_mechanics` and
// `monitor_states_the_real_reconciliation_mechanism` lived here, pinning
// `.sergeant/workflows/dispatch/05-classify-risk/CONTEXT.md` and
// `.../80-monitor/CONTEXT.md` byte-for-byte: --intent-file's real
// pure-content-transport mechanics (#166), the pointer to AGENTS.md's
// INTENT section for the eight-dimension risk brief rather than a
// restatement of it, the real automatic-at-startup reconciliation
// mechanism with no on-demand sync verb (#167), and the absence of any
// leftover "Engine gap" note. Both tests are removed here because the
// package they pinned, `dispatch`, was retired by the 2026-08-22 distro
// content rebuild (sergeant-rs-workspace's knowledge/evidence/reference/distro-content-2026-08-22.md, W2;
// design proposal §4.1) — the kickoff ruling cut it outright rather than
// keeping it alive as a transitional package, and neither file exists
// under `.sergeant/workflows/` any more for a test to read.
//
// The doctrine these tests pinned is not thereby licensed to drift: the
// `--intent-file` mechanics they asserted are still true of the binary
// and are still stated in AGENTS.md's INTENT section. The third
// assertion `classify_risk_states_the_real_intent_file_mechanics` made —
// that AGENTS.md carries that section at all — is preserved below as its
// own standalone test rather than lost with the package that pointed at
// it.

/// AGENTS.md still carries the `### INTENT — Captain's intent discipline`
/// section — the one home for the eight-dimension risk brief, which used
/// to be pinned as a side effect of `classify_risk_states_the_real_intent_
/// file_mechanics` (removed above, with `dispatch`). Lifted into its own
/// test so retiring that package's pinning test does not silently drop
/// this half of the claim.
#[test]
fn agents_md_carries_the_intent_section() {
    let agents_md = std::fs::read_to_string(repo_root().join("AGENTS.md")).expect("read AGENTS.md");
    assert!(
        agents_md.contains("### INTENT — Captain's intent discipline"),
        "AGENTS.md no longer carries the INTENT section that named the eight-dimension \
         risk brief for sensitive-territory Work"
    );
}

// --------------------------------------------------------- 6. #180 wording

/// AGENTS.md's mirror sentence states the ratified #180 contract — a
/// declared mutation surface, observed and journaled, never an enforced
/// one. Quoted verbatim, same house pattern as above.
///
/// This test used to also pin the North Star ruling's own copy of the same
/// sentence; that document left this repo for the sergeant-rs-workspace
/// knowledge library under the split-hardening series (W1 relocated it,
/// W2c removed the local file), so AGENTS.md is now the sole doctrine
/// surface this repo's build and tests depend on.
#[test]
fn agents_md_states_the_ratified_mutation_surface_contract() {
    let agents_md = std::fs::read_to_string(repo_root().join("AGENTS.md")).expect("read AGENTS.md");
    // AGENTS.md soft-wraps; collapse whitespace so a sentence spanning a
    // line break still matches a `contains` check on its logical text
    // rather than its accidental line layout (same trick
    // `agents_md_session_start_matches_the_real_root_gate` uses above).
    let agents_md_flat = agents_md.split_whitespace().collect::<Vec<_>>().join(" ");

    assert!(
        agents_md_flat.contains(
            "It is carried by `sgt`, a durable intent-execution engine that gives every Work \
             its own git worktree and a declared mutation surface — authorization, not a \
             seal — and runs submitted intents to completion against it, journaling what it \
             can prove happened outside that surface as dirty evidence at retirement rather \
             than silently absorbing it"
        ),
        "AGENTS.md's mirror sentence no longer states the ratified #180 contract — update this \
         test and the doctrine text together"
    );

    // Factual-not-exhortative framing (MUTATION_SURFACE_HEADER's own rule,
    // src/backend/claude.rs): the destination text must never claim the
    // mutation surface itself is enforced.
    assert!(
        !agents_md_flat.contains("enforces the mutation surface")
            && !agents_md_flat.contains("enforced mutation surface")
            && !agents_md_flat.contains("mutation surface is enforced"),
        "AGENTS.md must state the mutation surface as declared and observed, never enforced"
    );
}

/// W1's worker-supersession ratification (host-atlas-r3-ratification
/// ruling 2, J3): the ESTATE section's fifth worker bullet is superseded
/// "for this one sanctioned path only" — every other item in that list
/// stands untouched. This pins the amended sentence naming the sanctioned
/// `-C "$SERGEANT_ESTATE_ROOT" run` path AND, as a regression guard, that
/// the other four bullets are still present verbatim — an edit that
/// widens the amendment into removing one of those four would pass a pin
/// on the new sentence alone but must not pass this one.
///
/// Written first against the pre-W1 text (the blanket "invoke an
/// estate-scoped `sgt` command from its own surface" prohibition, no
/// exception) to confirm it goes red on exactly that wording, then
/// against the amended text to confirm green — the red-then-green
/// discipline the wave brief requires or this pin is decorative rather
/// than load-bearing.
#[test]
fn agents_md_estate_bullets_state_the_sanctioned_child_work_path() {
    let agents_md = std::fs::read_to_string(repo_root().join("AGENTS.md")).expect("read AGENTS.md");
    let agents_md_flat = agents_md.split_whitespace().collect::<Vec<_>>().join(" ");

    // The other four bullets (295-298 at the integration ref) stand
    // verbatim — the ratification is explicit only the fifth is narrowed.
    for bullet in [
        "edit a `repos/` mount;",
        "create a replacement branch;",
        "navigate into another Work's surface;",
        "expand its own repository scope;",
    ] {
        assert!(
            agents_md_flat.contains(bullet),
            "AGENTS.md's ESTATE worker bullets must keep this untouched bullet verbatim: {bullet}"
        );
    }

    // The amended fifth bullet names the one sanctioned path and the
    // journal-validation the ratification requires before it narrows the
    // prohibition.
    assert!(
        agents_md_flat.contains("sgt -C \"$SERGEANT_ESTATE_ROOT\" run"),
        "AGENTS.md's amended worker bullet must name the sanctioned `-C \
         \"$SERGEANT_ESTATE_ROOT\" run` path — the ratified exception, not a blanket refusal"
    );
    assert!(
        agents_md_flat.contains("validates") && agents_md_flat.contains("journal"),
        "AGENTS.md's amended worker bullet must state that the daemon validates the claimed \
         causation against its own journal before recording the relation"
    );

    // The narrowing framing itself: a worker still must not silently
    // become a nested Captain — preserved by admission/addressing/
    // validated-causation now, not by a blanket refusal (ratification
    // ruling 2's own words).
    assert!(
        agents_md_flat.contains("nested Captain"),
        "AGENTS.md's amendment must restate the 'not a nested Captain' preservation clause so \
         the change reads as a narrowing, not a removal of the worker-surface prohibition"
    );

    // Regression guard: the pre-W1 blanket prohibition text must be gone,
    // not merely amended-around — a copy-paste-drifted edit that left the
    // old unconditional sentence sitting beside the new exception would
    // otherwise pass every assertion above.
    assert!(
        !agents_md_flat.contains(
            "invoke an estate-scoped `sgt` command from its own surface — no `sergeant.toml` \
             lives there, and Session start's refusal applies even from inside it."
        ),
        "AGENTS.md must not still carry the old unconditional worker-surface prohibition \
         alongside the new sanctioned-path exception"
    );
}

/// Aria-seat review (S2, adjudicated by the captain against host-atlas r3
/// ratification ruling 2, the amended-E8 validation ruling, and the
/// single-user trust ruling — J3) found the amended fifth bullet above
/// left three things ambiguous: whether "possession of the triple" itself
/// was the sanction (rather than the engine's act of injecting it), what
/// the ellipsis after `sgt -C "$SERGEANT_ESTATE_ROOT" run` stood for, and
/// whether "rather than by refusal" repealed Session start's root gate.
/// This pins the resolving language.
#[test]
fn agents_md_estate_bullet_resolves_possession_vs_injection_and_the_root_gate() {
    let agents_md = std::fs::read_to_string(repo_root().join("AGENTS.md")).expect("read AGENTS.md");
    let agents_md_flat = agents_md.split_whitespace().collect::<Vec<_>>().join(" ");

    // (a) the sanction covers a managed execution acting on the
    // engine-injected identity; a self-fabricated triple is not
    // sanctioned and lands as journaled, adjudicable evidence.
    assert!(
        agents_md_flat.contains("the causation triple the ENGINE itself injected"),
        "AGENTS.md must state the sanctioned triple is the one the ENGINE injected, not \
         merely possessed by the worker"
    );
    assert!(
        agents_md_flat.contains("a self-fabricated triple is not sanctioned"),
        "AGENTS.md must state that a self-fabricated triple is not sanctioned"
    );
    assert!(
        agents_md_flat.contains("causation_unverified")
            && agents_md_flat.contains("reported and adjudicable, never silently honored"),
        "AGENTS.md must state a self-fabricated claim lands as journaled \
         `causation_unverified` evidence, reported and adjudicable, never silently honored"
    );

    // (b) the purpose clause is restated explicitly, not left as an
    // ellipsis after the run command.
    assert!(
        agents_md_flat.contains("sgt -C \"$SERGEANT_ESTATE_ROOT\" run` to create child Work"),
        "AGENTS.md must restate the purpose clause 'to create child Work' explicitly rather \
         than eliding it after the run command"
    );

    // (c) the refusal statement is unambiguous: Session start's root gate
    // still applies to bare invocation, the sanctioned path proceeds only
    // via explicit `-C` addressing, and "rather than by refusal" is a
    // statement about the daemon's treatment of the sanctioned path's
    // claims, not a repeal of the root gate.
    assert!(
        agents_md_flat.contains(
            "Session start's exact-root refusal still applies to any bare invocation from a \
             surface"
        ),
        "AGENTS.md must state Session start's exact-root refusal still applies to any bare \
         invocation from a surface"
    );
    assert!(
        agents_md_flat.contains("proceeds only via explicit `-C`"),
        "AGENTS.md must state the sanctioned path proceeds only via explicit `-C` addressing"
    );
    assert!(
        agents_md_flat.contains("not a repeal of Session start's root gate"),
        "AGENTS.md must state 'rather than by refusal' describes the daemon's treatment of \
         the sanctioned path's claims, not a repeal of Session start's root gate"
    );
}

// ------------------------------------------------ 7. citation-integrity skew
//
// split-hardening W2c made this repo product-documents-only: NORTH-STAR.md,
// GAUNTLET.md, LESSONS.md, and the internal-process docs/ tree that had
// accumulated (ADRs-as-working-notes, proposals, perf baselines, coverage
// dumps, ICM convention drafts, handoff checklists — except the handful of
// items re-homed into assets/ and CONTRIBUTING.md) are gone, their content
// already verified and pushed to the sergeant-rs-workspace knowledge
// library. A leftover path-like reference to any of them is a dangling
// citation, not live doctrine.
//
// docs/ itself has since been reintroduced as the product manual (a
// getting-started/concepts/guides/reference tree, not a development-process
// archive) — a bare `docs/` needle would now misfire on every legitimate
// citation to it, so this checks the specific removed-content subpaths
// instead, mirroring `no_embedded_skill_or_workflow_file_cites_a_removed_
// or_workspace_only_path`'s already-narrow needle set below. Excludes
// `CHANGELOG.md` (an append-only ledger this wave does not edit).
#[test]
fn no_readme_contributing_src_test_or_workflow_file_cites_a_removed_path() {
    const NEEDLES: &[&str] = &[
        "NORTH-STAR.md",
        "GAUNTLET.md",
        "LESSONS.md",
        "docs/icm/",
        "docs/adr/",
        "docs/DEVELOPMENT.md",
        "docs/environments/",
        "docs/proposals/",
        "docs/perf/",
        "docs/coverage/",
        "docs/handoff/",
        "docs/measure-dist",
        "docs/gauntlet/",
        "docs/glossary.md",
        "docs/version-policy.md",
    ];

    let mut roots = vec![
        repo_root().join("README.md"),
        repo_root().join("CONTRIBUTING.md"),
    ];
    for dir in ["src", "tests", ".github"] {
        roots.extend(walk(&repo_root().join(dir)));
    }

    let self_path = repo_root().join("tests/f_doctrine_skew.rs");
    let mut offenders = Vec::new();
    for path in roots {
        // This test's own source necessarily spells out the needles it
        // checks for (and describes, in its doc comment, what it excludes)
        // — comparing it against itself would be a tautological failure.
        if !path.is_file() || path == self_path {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for needle in NEEDLES {
            if text.contains(needle) {
                offenders.push(format!("{}: still cites {needle:?}", path.display()));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "dangling citation(s) to a path removed by split-hardening W2c:\n{}",
        offenders.join("\n")
    );
}

/// split-hardening W3 (#261) extended the sweep above onto `skills/` and
/// `.sergeant/` — the embedded distro content `sgt init` actually ships —
/// now that those stale routes are fixed. This is a separate test rather
/// than folding into the one above because the embedded corpus
/// legitimately uses a bare `docs/` as a placeholder for *a consumer
/// repository's own* docs directory (e.g. `review-change/10-identify-spec-
/// source/CONTEXT.md`'s "a PRD/spec file under `docs/`, `specs/`, or
/// `.scratch/`"), so the needle set here is the specific stale-route
/// fragments split-hardening actually removed or re-pointed, not a bare
/// `docs/` substring that would misfire on that legitimate generic usage.
#[test]
fn no_embedded_skill_or_workflow_file_cites_a_removed_or_workspace_only_path() {
    const NEEDLES: &[&str] = &[
        "NORTH-STAR.md",
        "GAUNTLET.md",
        "LESSONS.md",
        "docs/icm/",
        "docs/adr/",
        "docs/DEVELOPMENT.md",
        "docs/environments/",
        "docs/proposals/",
        "sergeant-rs-workspace",
    ];

    let mut roots = walk(&repo_root().join("skills"));
    roots.extend(walk(&repo_root().join(".sergeant")));
    roots.push(repo_root().join("AGENTS.md"));

    let mut offenders = Vec::new();
    for path in roots {
        if !path.is_file() {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for needle in NEEDLES {
            if text.contains(needle) {
                offenders.push(format!("{}: still cites {needle:?}", path.display()));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "dangling citation(s) to a path removed by split-hardening W2c, or to the private \
         workspace repo, in the embedded distro:\n{}",
        offenders.join("\n")
    );
}
