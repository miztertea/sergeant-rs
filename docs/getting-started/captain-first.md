# Use Captain

Captain is the intended front door: a supported coding harness operating at the estate root with Sergeant's `AGENTS.md` and skills.

```sh
cd ~/estates/example
sgt claude
# or: sgt codex, sgt opencode, sgt goose, sgt agy
```

Describe the repositories that belong together, the outcome you want, constraints, acceptance evidence, and any decisions only you may make. Captain resolves ambiguity, inspects the live workflow catalog, names a workflow and scope, and dispatches accepted intent.

Captain owns conversation and intent shaping. Sergeant owns durable completion. A procedure that needs a live interview remains a Captain skill; a self-contained procedure that can reach a terminal outcome is a Sergeant workflow.

After dispatch, Captain can block on `sgt watch` rather than poll. If Work needs input, answer through `sgt respond`; if it blocks or fails, inspect evidence before choosing retry, extension, or cancellation.
