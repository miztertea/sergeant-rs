//! The distro — `AGENTS.md`, `skills/`, `.sergeant/common/contexts/`, and
//! `.sergeant/workflows/` — embedded in the `sgt` binary at compile time and
//! written into a fresh estate by `sgt init` (issue #165, ADR 0014 decision
//! 1: "ships embedded in the `sgt` binary and is written to disk by `sgt
//! init`. It is not a cloned directory.").
//!
//! Ponytail R5: no dependency already in `Cargo.lock` does whole-directory
//! compile-time embedding — `include_str!` (stdlib, R3) is already used for
//! the built-in `software-change` workflow
//! (`domain::workflow::EMBEDDED_WORKFLOW_TOML`/`EMBEDDED_CONTEXTS`), but that
//! precedent hand-lists five known literal paths; it does not generalize to
//! 200+ files across an arbitrarily nested tree without either hand-listing
//! every path (silently stale the moment a file is added and the list
//! isn't) or a directory-walking build script (more code than the dependency
//! it would replace, and re-solves a problem `include_dir` already solves
//! narrowly). `include_dir` was chosen over `rust-embed`: `rust-embed`
//! re-reads files from disk at runtime in debug builds (a `Cow`-based API
//! and a `cfg`-branched code path this use case has no reason to carry —
//! the estate the binary writes into is never the same tree the binary was
//! built from), where `include_dir` gives exactly the shape needed —
//! a `Dir<'static>` tree of `&'static [u8]` file contents, nothing else —
//! for a two-crate addition (`include_dir` + its proc-macro half
//! `include_dir_macros`, confirmed via `cargo add --dry-run` and `cargo
//! fetch`; no transitive crates beyond those two). Measured cost of this
//! addition, recorded in the commit message: release binary size delta and
//! `cargo build --release` wall time, before vs. after.

use std::io;
use std::path::{Path, PathBuf};

use include_dir::{Dir, include_dir};

use crate::runtime::fsutil::{create_dir_all_durable, write_atomic};

static AGENTS_MD: &str = include_str!("../../AGENTS.md");
static SKILLS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/skills");
static CONTEXTS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/.sergeant/common/contexts");
static WORKFLOWS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/.sergeant/workflows");

/// Which paths this call actually wrote vs. left untouched because they
/// already existed. `written.is_empty()` on a second `sgt init` is the
/// no-op signal the AGENTS.md guardrail requires ("re-running `sgt init` on
/// an already-initialized estate is a no-op, not a reset").
#[derive(Debug, Default, Clone)]
pub struct DistroWriteOutcome {
    /// Paths created by this call, relative to the estate root.
    pub written: Vec<PathBuf>,
    /// Paths that already existed and were left alone, relative to the
    /// estate root.
    pub skipped: Vec<PathBuf>,
}

impl DistroWriteOutcome {
    /// Whether this call wrote anything at all.
    pub fn changed(&self) -> bool {
        !self.written.is_empty()
    }
}

/// Write the embedded distro into `estate_root`.
///
/// Per-file idempotent: a file that already exists at its target path is
/// never overwritten, so a second `sgt init` on an already-initialized
/// estate writes nothing (hard constraint: re-running init is a no-op, not
/// a reset). Writes only under `estate_root` — `AGENTS.md`, `skills/`,
/// `.sergeant/common/contexts/`, `.sergeant/workflows/` — and never touches
/// `.sergeant/local/`, which is the user's own and which Phase 3's
/// local-shadows-stock resolution (`domain::workflow::WorkflowDefinition::
/// resolve`) depends on staying separate from stock.
///
/// `index.md` and `SKILL.md` front matter's `edition` field is rewritten to
/// this binary's own version (`env!("CARGO_PKG_VERSION")`) as each file is
/// written, rather than trusted verbatim from the embedded content — the
/// mechanism ADR 0016 requires ("`sgt init`/update sets `edition` to the
/// current distro version whenever it writes a stock copy") without relying
/// on every future release remembering to hand-bump the checked-in
/// front matter to match `Cargo.toml` before cutting the release.
pub fn write_distro(estate_root: &Path) -> io::Result<DistroWriteOutcome> {
    let mut outcome = DistroWriteOutcome::default();

    write_file(
        estate_root,
        Path::new("AGENTS.md"),
        AGENTS_MD.as_bytes(),
        &mut outcome,
    )?;
    write_dir(estate_root, Path::new("skills"), &SKILLS, &mut outcome)?;
    write_dir(
        estate_root,
        Path::new(".sergeant/common/contexts"),
        &CONTEXTS,
        &mut outcome,
    )?;
    write_dir(
        estate_root,
        Path::new(".sergeant/workflows"),
        &WORKFLOWS,
        &mut outcome,
    )?;

    Ok(outcome)
}

/// Recurse `dir` (an `include_dir!` tree), writing every file it contains
/// under `estate_root/prefix/<file's own path within dir>` — `File::path()`
/// already carries the full path relative to the tree's own root, so no
/// manual path accumulation is needed as this recurses into subdirectories.
fn write_dir(
    estate_root: &Path,
    prefix: &Path,
    dir: &Dir<'static>,
    outcome: &mut DistroWriteOutcome,
) -> io::Result<()> {
    for file in dir.files() {
        write_file(
            estate_root,
            &prefix.join(file.path()),
            file.contents(),
            outcome,
        )?;
    }
    for sub in dir.dirs() {
        write_dir(estate_root, prefix, sub, outcome)?;
    }
    Ok(())
}

/// Write one file at `estate_root/rel_path` if and only if nothing is there
/// yet. `rel_path`'s leaf name decides whether `edition` front matter gets
/// rewritten to this binary's version before the bytes hit disk.
fn write_file(
    estate_root: &Path,
    rel_path: &Path,
    contents: &[u8],
    outcome: &mut DistroWriteOutcome,
) -> io::Result<()> {
    let full_path = estate_root.join(rel_path);
    if full_path.exists() {
        outcome.skipped.push(rel_path.to_path_buf());
        return Ok(());
    }
    if let Some(parent) = full_path.parent() {
        create_dir_all_durable(parent)?;
    }
    let carries_edition = matches!(
        rel_path.file_name().and_then(|n| n.to_str()),
        Some("index.md") | Some("SKILL.md")
    );
    if carries_edition && let Ok(text) = std::str::from_utf8(contents) {
        write_atomic(&full_path, rewrite_edition(text).as_bytes())?;
        outcome.written.push(rel_path.to_path_buf());
        return Ok(());
    }
    write_atomic(&full_path, contents)?;
    outcome.written.push(rel_path.to_path_buf());
    Ok(())
}

/// Replace the value of an `edition:` line inside the leading `---`
/// front-matter block with this binary's own version. A file with no such
/// line, or no front-matter block at all, is returned unchanged — this is
/// deliberately narrower than a YAML parser (matching
/// `domain::workflow::parse_index_front_matter`'s own "tiny local
/// composition" rather than a general parser, Ponytail R6), scoped to
/// exactly the shape `sergeant-rs-workspace's knowledge/rulings/icm/record-shapes.md` §1 fixes.
fn rewrite_edition(text: &str) -> String {
    let mut lines = text.lines();
    let Some(first) = lines.next() else {
        return text.to_string();
    };
    if first.trim() != "---" {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len() + 8);
    out.push_str(first);
    out.push('\n');
    let mut in_front_matter = true;
    for line in lines {
        if in_front_matter && line.trim() == "---" {
            in_front_matter = false;
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if in_front_matter && line.trim_start().starts_with("edition:") {
            out.push_str("edition: ");
            out.push_str(env!("CARGO_PKG_VERSION"));
            out.push('\n');
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_edition_replaces_the_value_inside_front_matter_only() {
        let text = "---\nkind: workflow\nedition: 0.0.1\nname: x\n---\n\nedition: 0.0.1 in the body must survive\n";
        let rewritten = rewrite_edition(text);
        assert!(rewritten.contains(&format!("edition: {}", env!("CARGO_PKG_VERSION"))));
        assert!(rewritten.contains("edition: 0.0.1 in the body must survive"));
    }

    #[test]
    fn rewrite_edition_is_a_no_op_without_front_matter() {
        let text = "no front matter here\nedition: 0.0.1\n";
        assert_eq!(rewrite_edition(text), text);
    }

    #[test]
    fn write_distro_is_per_file_idempotent() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let first = write_distro(tmp.path()).expect("first write");
        assert!(first.changed());
        assert!(tmp.path().join("AGENTS.md").is_file());
        assert!(tmp.path().join(".sergeant/workflows").is_dir());

        let sentinel = tmp.path().join("AGENTS.md");
        let edited = "user-edited content, must survive a second call\n";
        std::fs::write(&sentinel, edited).expect("simulate a user edit");

        let second = write_distro(tmp.path()).expect("second write");
        assert!(
            !second.changed(),
            "a second call must write nothing, got {:?}",
            second.written
        );
        assert_eq!(
            std::fs::read_to_string(&sentinel).expect("read AGENTS.md"),
            edited,
            "a second call must never clobber an existing file"
        );
    }

    #[test]
    fn write_distro_never_touches_local() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        write_distro(tmp.path()).expect("write");
        assert!(!tmp.path().join(".sergeant/local").exists());
    }

    #[test]
    fn write_distro_stamps_the_binary_edition() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        write_distro(tmp.path()).expect("write");

        let mut checked_a_workflow = false;
        for entry in std::fs::read_dir(tmp.path().join(".sergeant/workflows")).expect("read_dir") {
            let entry = entry.expect("entry");
            let index = entry.path().join("index.md");
            if index.is_file() {
                let text = std::fs::read_to_string(&index).expect("read index.md");
                assert!(
                    text.contains(&format!("edition: {}", env!("CARGO_PKG_VERSION"))),
                    "{} must carry the binary's edition, got:\n{text}",
                    index.display()
                );
                checked_a_workflow = true;
            }
        }
        assert!(checked_a_workflow, "expected at least one stock index.md");

        for entry in std::fs::read_dir(tmp.path().join("skills")).expect("read_dir") {
            let entry = entry.expect("entry");
            let skill_md = entry.path().join("SKILL.md");
            if skill_md.is_file() {
                let text = std::fs::read_to_string(&skill_md).expect("read SKILL.md");
                assert!(
                    text.contains(&format!("edition: {}", env!("CARGO_PKG_VERSION"))),
                    "{} must carry the binary's edition, got:\n{text}",
                    skill_md.display()
                );
            }
        }
    }
}
