//! Domain model: the core types describing work, execution, and workflow.

pub mod distro;
pub mod estate;
pub mod event;
pub mod execution;
pub mod manifest;
pub mod profile;
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
pub(crate) fn is_plain_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains('/')
        && !name.contains('\\')
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
