# Operations and troubleshooting

Start with `sgt doctor`. It checks installation, exact-root admission, manifest validity, repository mounts, runtime descriptors, journal growth, Docker requirements, and distro edition, and prints a remedy for failures.

Use `sgt tui` for Home, Workflows, Estate, fleet, and Work views. Attention states identify Work needing an operator; the journal remains authoritative when a projection or telemetry system disagrees.

One host daemon per user installation owns the journal and the DuckDB projection under a host runtime root — `--data-dir`, `SGT_DATA_DIR`, or the platform default — shared across every estate it has admitted. Estate-local material (`sergeant.toml`, repository mounts, Work surfaces) stays under the estate itself unless `[estate] surfaces_dir`/`SGT_SURFACES_DIR` selects another location. Default retention is 1,000 terminal Works per estate and the minimum configurable value is 64. Pruning is Work-aware, journaled, and partitioned per estate — one estate's own retention never condemns another's Work. Output branches remain outside automatic retention deletion.

Troubleshooting order:

1. Confirm the exact estate root or use `-C`.
2. Run `sgt doctor` and apply its named remedy.
3. Inspect `sgt status` and Work evidence.
4. Restart the daemon if needed; `sgt daemon stop` stops every estate it has admitted at once, not only the one you are working in, so a restart is felt host-wide. Do not delete journal or runtime files to force recovery.
5. Use retry/respond/extend only for the state that permits it.
6. Preserve unexpected behavior as evidence and report it separately from documentation changes.
