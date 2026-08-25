# Operations and troubleshooting

Start with `sgt doctor`. It checks installation, exact-root admission, manifest validity, repository mounts, runtime descriptors, journal growth, Docker requirements, and distro edition, and prints a remedy for failures.

Use `sgt tui` for Home, Workflows, Estate, fleet, and Work views. Attention states identify Work needing an operator; the journal remains authoritative when a projection or telemetry system disagrees.

Each estate normally stores runtime state under `.sergeant/data`, unless `sergeant.toml`, global `--data-dir`, or `SGT_DATA_DIR` selects another location. Default retention is 1,000 terminal Works and the minimum configurable value is 64. Pruning is Work-aware and journaled. Output branches remain outside automatic retention deletion.

Troubleshooting order:

1. Confirm the exact estate root or use `-C`.
2. Run `sgt doctor` and apply its named remedy.
3. Inspect `sgt status` and Work evidence.
4. Restart the daemon; do not delete journal or runtime files to force recovery.
5. Use retry/respond/extend only for the state that permits it.
6. Preserve unexpected behavior as evidence and report it separately from documentation changes.
