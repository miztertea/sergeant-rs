//! Domain model: the core types describing work, execution, and workflow.

pub mod distro;
pub mod estate;
pub mod event;
pub mod execution;
pub mod manifest;
pub mod profile;
pub mod source;
pub mod work;
pub mod workflow;

use std::path::Path;

/// Whether `name` is safe to join directly onto a filesystem path: a single,
/// non-empty path component with no separators and no `.`/`..`.
///
/// Three user-controlled names are joined straight onto a root directory —
/// a workflow name onto `.sergeant/workflows/` ([`workflow::WorkflowDefinition::resolve`]),
/// a stage id onto its workflow directory, and a repository name onto
/// `<data-dir>/surfaces/<work-id>/` ([`estate::Estate::from_config`]).
/// That is one rule, so it lives in one function: a path-traversal guard
/// copied per module is a guard that can be fixed in one copy and left broken
/// in the others.
///
/// Also rejects a Windows drive-letter/UNC-style component (e.g. `C:`, or
/// any component containing `:` at all) **independent of the host OS**: two
/// container-adapter callers ([`crate::runtime::atlas::mail`]'s attachment
/// filenames, [`crate::runtime::atlas::worker`]'s `enclosed_relative_path`)
/// reuse this per-`/`-component check and document it as `enclosed_name`
/// semantics — "a relative path with no absolute component" — but
/// `std::path::Path` only recognizes a drive-letter prefix as a `Prefix`
/// component on Windows itself; on the Unix host this build actually runs
/// on, `Path::new("C:").components().count()` is `1`, an ordinary Normal
/// component, so a bare `contains('\\')`/`contains('/')` check alone let a
/// component like `C:` — the first segment of `C:/Windows/System32/evil.zip`
/// — through as "plain". Rejecting any `:` closes that gap without making
/// this function's behavior depend on which OS it happens to run on.
pub(crate) fn is_plain_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains(':')
        && name != "."
        && name != ".."
        && Path::new(name).components().count() == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The one rule, stated once: anything that could denote somewhere other
    /// than "a child of this directory" is refused.
    #[test]
    fn only_a_single_child_component_is_a_plain_name() {
        for ok in ["software-change", "00-prepare", "api", ".git", "-x", "a.b"] {
            assert!(is_plain_name(ok), "{ok:?} is a plain directory name");
        }
        for bad in [
            "", ".", "..", "/", "/etc", "a/b", "../etc", "..\\etc", "a\\b", "./a", "a/",
        ] {
            assert!(!is_plain_name(bad), "{bad:?} must not be a plain name");
        }
    }

    /// A review finding: `Path::new("C:").components().count() == 1` on the
    /// Unix host this build runs on — no `Prefix` component recognition
    /// off-Windows — so a bare separator/`.`/`..` check alone let a
    /// Windows drive-letter-absolute filename's first component through as
    /// "plain". Pinned per-component AND as the composed enclosed-relative-path
    /// shape a container-adapter attachment name actually arrives in (mail.rs's
    /// `filename.split('/').all(is_plain_name)`, worker.rs's
    /// `enclosed_relative_path`).
    #[test]
    fn a_windows_drive_letter_component_is_never_a_plain_name() {
        for bad in ["C:", "c:", "C:evil.txt", "AB:"] {
            assert!(!is_plain_name(bad), "{bad:?} must not be a plain name");
        }
        let attacker_path = "C:/Windows/System32/evil.zip";
        assert!(
            !attacker_path.split('/').all(is_plain_name),
            "a drive-letter-absolute attachment/entry name must be refused, not admitted \
             component-by-component"
        );
    }

    /// W1 §3: a nested workflow package's stage id is a `/`-joined path of
    /// components (`10-investigate/00-lead`). This guard stays
    /// per-*component*: the composite is deliberately never a plain name, so
    /// a caller that passed a whole hierarchical stage id here would refuse
    /// every valid nested stage rather than validate one. Only
    /// [`workflow::WorkflowDefinition::load_dir`] joins components, and only
    /// after checking each one through here.
    #[test]
    fn a_composed_hierarchical_stage_id_is_never_a_plain_name() {
        let composed = "10-investigate/00-lead";
        assert!(
            !is_plain_name(composed),
            "the composite must not pass the per-component guard"
        );
        for component in composed.split('/') {
            assert!(is_plain_name(component), "{component:?} is a plain name");
        }
    }
}
