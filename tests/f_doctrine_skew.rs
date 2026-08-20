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

/// AGENTS.md's "Session start" section names exactly the unscoped command
/// set `is_estate_scoped` (`src/cli.rs`) computes, and the real refusal an
/// estate-scoped command gives outside an estate carries the exact wording
/// AGENTS.md quotes for it.
#[test]
fn agents_md_session_start_matches_the_real_root_gate() {
    let agents_md = std::fs::read_to_string(repo_root().join("AGENTS.md")).expect("read AGENTS.md");
    // Markdown prose in this file soft-wraps; collapse whitespace so a
    // sentence spanning a line break still matches a `contains` check on
    // its logical text rather than its accidental line layout.
    let agents_md_flat = agents_md.split_whitespace().collect::<Vec<_>>().join(" ");

    // The claim, quoted verbatim from `is_estate_scoped`'s own doc comment
    // ("Unscoped: bare `sgt`, `--help`, `--version`, `init`, `doctor` —
    // nothing else") in AGENTS.md's own voice.
    assert!(
        agents_md_flat.contains(
            "Only `sgt --help`, `--version`, `sgt init`, and `sgt doctor` work outside an \
             estate at all."
        ),
        "AGENTS.md's Session start section no longer states the unscoped command set — \
         update this test and the claim together, from `is_estate_scoped`'s own doc comment"
    );

    // The refusal wording AGENTS.md quotes must be the real one
    // (`EstateRootError`'s `Display`, `src/domain/estate.rs`), not a
    // paraphrase that could drift from it.
    assert!(
        agents_md_flat.contains("Sergeant does not search parent directories for an estate"),
        "AGENTS.md must quote the real root-gate diagnostic wording"
    );

    let dir = bare_dir();

    // The four commands AGENTS.md claims work outside an estate must
    // actually run there — none of them the root-gate refusal.
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
    // refusal checks from a second, still-bare directory so they test the
    // "no estate at all" case, not "estate exists but I'm not at its root".
    let refused = bare_dir();
    for args in [
        vec!["status"],
        vec!["work", "list"],
        vec!["watch"],
        vec!["analytics"],
        vec!["repo", "list"],
    ] {
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
    //    estate — every remedy it names must be one `sergeant-help` names too.
    let refused = bare_dir();
    let refusal = stderr(&run(refused.path(), &["status"]));
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
/// paragraph rather than the same one (`dispatch/80-monitor/CONTEXT.md` is
/// exactly this shape). A false negative here is possible; a false positive
/// on real, already-disclaimed content is not, because every current file
/// is asserted to pass below.
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

/// The estate-root proposal's own canonical manifest shape (§13.1,
/// `docs/proposals/estate-root-git.md`) still parses under the current
/// schema — structurally (`Estate::from_config_structural`, which validates
/// names/duplicates/groups/profiles without requiring `repos/<name>` to be
/// real git checkouts on disk, exactly right for a doc example that names
/// no real repository).
#[test]
fn the_proposals_canonical_manifest_example_parses_under_the_current_schema() {
    let proposal = std::fs::read_to_string(repo_root().join("docs/proposals/estate-root-git.md"))
        .expect("read estate-root-git.md");

    let anchor = proposal
        .find("13.1 Proposed manifest shape")
        .expect("proposal must still have a §13.1 'Proposed manifest shape' heading");
    let after_anchor = &proposal[anchor..];
    let fence_start = after_anchor
        .find("```toml")
        .expect("§13.1 must be followed by a ```toml fence");
    let block_start = fence_start + "```toml".len();
    let block_end = after_anchor[block_start..]
        .find("```")
        .expect("the §13.1 toml fence must close");
    let toml_text = after_anchor[block_start..block_start + block_end].trim();

    // Structural sanity: this is the specific example this test means to
    // pin, not whatever now happens to follow the same heading text.
    assert!(
        toml_text.contains("[estate]"),
        "unexpected block: {toml_text}"
    );
    assert!(
        toml_text.contains("[[repo]]"),
        "unexpected block: {toml_text}"
    );
    assert!(
        toml_text.contains("[group."),
        "unexpected block: {toml_text}"
    );
    assert!(
        toml_text.contains("[[profile]]"),
        "unexpected block: {toml_text}"
    );

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
