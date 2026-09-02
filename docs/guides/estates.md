# Estate guide

Choose an estate boundary around repositories that are commonly reasoned about or changed together. A single-repository estate is legitimate. A product estate often contains frontend, backend, services, infrastructure, and engineering knowledge. Use separate estates when clients, permissions, retention, or operating context should be independent.

Add and inspect repositories with `sgt repo add`, `sgt repo list`, and the corresponding remove operation shown by `sgt repo --help`. `origin` is the clone source; `upstream` models contributor-fork topology. Sergeant derives mounts at `repos/<name>` and refuses linked worktrees, aliases, or unexpected Git roots as base mounts.

Use groups for repeated scopes such as `product`, `payments`, or `platform`. Keep them compositional: do not encode a workflow choice into a group.

Knowledge repositories are ordinary repositories. Include one in Work when architecture notes, runbooks, or decisions are part of the outcome.

Estate hygiene:

1. Keep base mounts clean and on the expected branch.
2. Reconcile remotes before dispatch.
3. Run `sgt doctor` after topology changes.
4. Inspect retained Work surfaces and branches before manual cleanup.
5. Use multiple estates rather than weakening boundaries when permissions or ownership differ.

Common patterns are one repo; frontend + backend; service fleet plus shared libraries; product repos plus knowledge; fork plus upstream; one broad estate with overlapping groups; or several purpose-built estates. The primitive is unchanged in every case.
