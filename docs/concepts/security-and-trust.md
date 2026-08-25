# Security and trust

Sergeant runs as the current user and inherits that user's filesystem access, credentials, environment, harness permissions, and Git authority. Native harnesses own authentication; profiles configure launches but do not broker credentials.

Worktrees define the intended mutation surface and let Sergeant attribute Git results. They are not a containment boundary. For stronger isolation, run Sergeant and harnesses in a VM, container, sandbox, or restricted account.

Execute stages declare read-only or read-write workspace access and currently require `network = "none"`. Workflow environment entries are plaintext versioned configuration and must not contain secrets.

The daemon API is loopback HTTP/SSE. `/v1/*` requires the daemon bearer token; `/healthz` does not. Protect the estate data directory because it contains runtime descriptors and durable evidence.
