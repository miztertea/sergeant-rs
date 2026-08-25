# Complete your first Work

From an estate with at least one repository, submit a deterministic trial:

```sh
sgt run "inspect the repository and report the smallest useful improvement" --repo app --backend fake
sgt work list
sgt watch <work-id> --follow
sgt work show <work-id>
sgt work transcript <work-id>
```

Omitting `--workflow` deliberately selects the embedded `software-change` workflow. For real actor execution, choose a supported backend such as `claude`, `codex`, `opencode`, or `agy`, or use an estate profile.

The result belongs to the Work branch `sergeant/<work-id>` in every targeted repository. `sgt work show` reports the Work state, current stage, surfaces, output, integrity, and recent evidence. A terminal Work does not imply its branch was merged or deleted.
