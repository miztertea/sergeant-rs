# Install Sergeant

Sergeant supports x86_64 Linux, Apple Silicon macOS, and Windows through WSL2. You need Git. Install Docker only when a workflow uses an `execute` stage, and install at least one supported harness to run actor stages.

```sh
curl -fsSL https://github.com/miztertea/sergeant-rs/releases/latest/download/sergeant-rs-installer.sh | sh
sgt --version
sgt doctor
```

The installer verifies the selected archive against published checksums. Release assets can also be verified with `gh attestation verify`.

To upgrade, run the installer again. Before crossing releases, read the changelog for configuration or machine-contract changes. To uninstall, remove the installed `sgt` binary; removing it does not remove estates, repositories, Work branches, or estate data. Delete those separately only after inspecting them.

There is one Sergeant daemon per user installation, not one per estate. Run `sgt daemon install-service` to register it as a native per-user service — a systemd user unit on Linux/WSL, a LaunchAgent on macOS — so it survives logout of the terminal that started it and restarts if it crashes. `--print` shows what would be written without touching anything. Where no native user service manager is reachable, `sgt doctor` names that missing prerequisite and the foreground `sgt daemon` mode remains the development/diagnostic fallback. `sgt daemon stop` stops that one daemon and every estate it has admitted at once — there is no per-estate stop.

An installation carrying estate-local daemon state from before this host-daemon model — `sgt doctor`'s `legacy_estate_runtime` row names it — needs that state reconciled or abandoned before host mode is trustworthy for it: finish or cancel any non-terminal Work under the old install, then remove its estate-local runtime files; the doctor row names exactly which.

Building from source is contributor setup, documented in [CONTRIBUTING.md](../../CONTRIBUTING.md).
