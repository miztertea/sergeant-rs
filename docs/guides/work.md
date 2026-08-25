# Work operations

Submit with one scope selector (`--repo`, `--group`, or `--all`), an explicit workflow when the catalog fits, and a backend/profile. Monitor with the TUI, `sgt work show`, or one blocking `sgt watch --follow`; do not build a tight polling loop.

Operator actions:

| Situation | Action |
|---|---|
| `needs_input` | `sgt respond <id> "answer"` |
| `waiting`, `blocked`, or `failed` after its cause is resolved | `sgt retry <id>` |
| turn envelope exhausted | `sgt extend <id> <turns>`, then retry if still required |
| no safe continuation is wanted | `sgt cancel <id>` |

Before acting, inspect `sgt work show <id>`, `sgt work transcript <id>`, and `sgt work show <id> --graph`. Retry re-enters the current stage under the pinned execution decision; it does not silently select a new workflow or backend.

After interruption, restart or address the estate and inspect status. Sergeant replays the journal and recovers unambiguous Work. A blocked recovery is evidence that operator judgment is required, not permission to delete state.

Dirty or otherwise retained surfaces remain inspectable. Use the `work retained`, `work reap`, and `work sweep` families only after reading their command help. Reaping removes retained runtime surfaces; sweep classifies Work branches using Git ancestry. Work branches are never automatically deleted.
