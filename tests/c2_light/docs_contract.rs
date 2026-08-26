use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(path).expect("read docs") {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
                files.push(path);
            }
        }
    }
    files
}

#[test]
fn links_and_public_boundary_are_sound() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = markdown_files(&root.join("docs"));
    files.extend([
        root.join("README.md"),
        root.join("AGENTS.md"),
        root.join("CONTRIBUTING.md"),
        root.join(".sergeant/index.md"),
    ]);
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        assert!(
            !text.contains("sergeant-rs-workspace"),
            "{} names private workspace",
            file.display()
        );
        for part in text.split("](").skip(1) {
            let target = part
                .split(')')
                .next()
                .unwrap_or("")
                .split('#')
                .next()
                .unwrap_or("");
            if target.is_empty() || target.contains("://") || target.starts_with('#') {
                continue;
            }
            assert!(
                file.parent().unwrap().join(target).exists(),
                "broken link in {}: {}",
                file.display(),
                target
            );
        }
    }
}

#[test]
fn catalogs_cover_published_workflows() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let packages: BTreeSet<_> = fs::read_dir(root.join(".sergeant/workflows"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    for catalog in [".sergeant/index.md", "docs/workflows/index.md"] {
        let text = fs::read_to_string(root.join(catalog)).unwrap();
        for package in &packages {
            assert!(
                text.contains(&format!("`{package}`")),
                "{catalog} omits {package}"
            );
        }
    }
}

#[test]
fn captain_skill_reference_covers_shipped_skills() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let reference = fs::read_to_string(root.join("docs/reference/captain-skills.md")).unwrap();
    for entry in fs::read_dir(root.join("skills"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
    {
        let name = entry.file_name().to_string_lossy().into_owned();
        assert!(
            reference.contains(&format!("`{name}`")),
            "skill reference omits {name}"
        );
    }
}

#[test]
fn cli_reference_covers_real_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_sgt"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    let reference =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/reference/cli.md"))
            .unwrap();
    let commands = help
        .split("Commands:")
        .nth(1)
        .unwrap_or("")
        .split("Options:")
        .next()
        .unwrap_or("");
    for line in commands.lines() {
        let name = line.split_whitespace().next().unwrap_or("");
        if name.is_empty() || name == "help" {
            continue;
        }
        assert!(
            reference.contains(&format!("`{name}")),
            "CLI reference omits {name}"
        );
    }
}
