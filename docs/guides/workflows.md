# Workflow guide

Read the [catalog](../workflows/index.md) before submission. Choose the narrowest published workflow whose outcome matches the accepted intent. Omitting `--workflow` selects the embedded `software-change` loop; it is not a nameless routing fallback.

Fork a published package before editing it:

```sh
sgt workflow fork implement-change
```

The local package under `.sergeant/local/workflows/<name>/` shadows the complete stock package with the same name. Shadowing is whole-package, not file overlay.

To author procedure, create `workflow.toml`, root `CONTEXT.md`, ordered stage directories, each stage's `CONTEXT.md`, and optional output declarations. Keep interaction and unresolved ambiguity in Captain. Use actor stages for bounded judgment and execute stages for deterministic commands.

New generated candidates belong in `.sergeant/drafts/workflows/` and are not runnable. Human review promotes an accepted package into `.sergeant/workflows/`; its `index.md` becomes the human catalog authority. `sgt workflow` (see `sgt workflow --help`) only forks a stock package to `.sergeant/local/workflows/<name>/`; it does not validate one. Validate with the real loader by submitting through `sgt run --workflow <name>` — the daemon's whole-workflow preflight parses every stage before any Work or worktree exists.

See the [package schema](../reference/workflow-package.md).
