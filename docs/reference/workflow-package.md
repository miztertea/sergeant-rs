# Workflow package and stage schema

A named package lives at `.sergeant/workflows/<name>/`; an estate-local fork lives at `.sergeant/local/workflows/<name>/` and shadows the stock package completely.

```text
<name>/
  workflow.toml
  CONTEXT.md
  index.md
  00-stage/CONTEXT.md
  10-stage/CONTEXT.md
```

`workflow.toml` declares `[workflow]` name, version, and ordered stages. A stage directory name is its ID. The pinned workflow identity is a hash over `name`, `version`, and each stage's execution-relevant fields (kind, harness, profile, its own `CONTEXT.md`, `requires_ask`, `receives_branch_status`, execute config) in order; the package's root-level `CONTEXT.md` and `index.md` are human documentation only and are not hashed.

Actor stages are the default. A stage may declare a harness, profile, `requires_ask`, and `receives_branch_status`. Omitted routing values inherit the Work decision and are resolved and pinned before execution.

Execute stages require an image, argv command (no implicit shell), workdir, explicit `workspace_access = "read_only"|"read_write"`, `network = "none"`, and optional plaintext environment map. The resolved immutable image identity is recorded at launch. Secrets do not belong in workflow TOML.

A stage may declare an expected output artifact in `<stage>/output/README.md`: a `` **Expected artifact:** `<file>` — description. `` line (a bare filename, no path separators) gates stage completion, an optional `**Required columns:**` line names columns the produced artifact's table must carry, and `**Disposition:**` (`` `promote` `` or absent) decides whether the finalize sweep ships the artifact in the Work branch or retains it as evidence and removes it. Silence on disposition means evidence, never promotion.

`index.md` owns the human trigger, use/avoid guidance, expected outcome, and stage summary. Draft packages are reviewable but not admitted or runnable.

A stage directory may itself hold a valid `workflow.toml`, making that stage a container whose implementation is another workflow package reusing this same grammar recursively — see [hierarchical execution](../concepts/hierarchical-execution.md) for the nesting, flattening, and cycle-guard rules, and for child Work, the separately-durable alternative to nesting.
