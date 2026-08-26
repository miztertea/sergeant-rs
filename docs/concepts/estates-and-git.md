# Estates and Git surfaces

An estate is an exact-root topology boundary. It provides a light-monorepo view without combining repository histories. One repository, many services, knowledge repositories, and contributor forks are all ordinary estate members.

Three Git locations must not be conflated:

| Location | Owner | Purpose |
|---|---|---|
| `repos/<name>` | estate | clean base mount used to admit and prepare Work |
| `<surfaces-dir>/<work-id>/<repo>` | worker | linked worktree where an actor may mutate files |
| `sergeant/<work-id>` | Work | durable output branch retained after terminal outcomes |

Groups are named sets of repositories. They express topology and convenient scope, not workflow policy. A repository may belong to several groups.

The declared surface is an authorization and attribution boundary, not an OS sandbox. Sergeant detects and records integrity problems; it cannot prevent a process running as you from reaching paths your account can reach.

An estate's own topology — mounts, groups, surfaces — is unaffected by which daemon process serves it. One host daemon per user installation admits many estates by their exact roots; see [host runtime and estates](host-runtime.md) for how that admission works and what it preserves from the single-estate model.
