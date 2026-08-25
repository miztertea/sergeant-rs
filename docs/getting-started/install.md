# Install Sergeant

Sergeant supports x86_64 Linux, Apple Silicon macOS, and Windows through WSL2. You need Git. Install Docker only when a workflow uses an `execute` stage, and install at least one supported harness to run actor stages.

```sh
curl -fsSL https://github.com/miztertea/sergeant-rs/releases/latest/download/sergeant-rs-installer.sh | sh
sgt --version
sgt doctor
```

The installer verifies the selected archive against published checksums. Release assets can also be verified with `gh attestation verify`.

To upgrade, run the installer again. Before crossing releases, read the changelog for configuration or machine-contract changes. To uninstall, remove the installed `sgt` binary; removing it does not remove estates, repositories, Work branches, or estate data. Delete those separately only after inspecting them.

Building from source is contributor setup, documented in [CONTRIBUTING.md](../../CONTRIBUTING.md).
