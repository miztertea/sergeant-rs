# A worked extraction example

Layer 3 (stable across runs), local to `20-harvest`. Illustrates the shape
in `../_config/evidence-policy.md`; it is not itself the rule — that file
is.

Source span (imagined `AGENTS.md`, lines 3-5):

```text
Before making changes to a repository, first verify that the
requested repository belongs to the currently loaded project. Log
every repository switch to the session's audit trail.
```

This span states **two** independently-triggerable behaviors bolted
together by "and": verifying membership before mutation, and logging
switches. Per evidence-policy's "one behavior per unit," that is two units,
not one — even though both can cite the same quoted span:

```json
{"id": "EX-0001", "statement": "Before changing a repository, verify that the requested repository belongs to the loaded project.", "source": {"path": "AGENTS.md", "locator": "AGENTS.md L3-5", "quote": "Before making changes to a repository, first verify that the\nrequested repository belongs to the currently loaded project. Log\nevery repository switch to the session's audit trail.", "quote_hash": "sha256:6e18bfe8efe643163295884a9af67a30474e884799d6add9b7fac0228e50c1f8"}, "scope": "cross-repository work", "trigger": "a work request names or implies a project repository", "outcome": "repository membership is established before mutation", "authority": "user-context actor", "confidence": "high", "notes": ""}
{"id": "EX-0002", "statement": "Every repository switch is logged to the session's audit trail.", "source": {"path": "AGENTS.md", "locator": "AGENTS.md L3-5", "quote": "Before making changes to a repository, first verify that the\nrequested repository belongs to the currently loaded project. Log\nevery repository switch to the session's audit trail.", "quote_hash": "sha256:6e18bfe8efe643163295884a9af67a30474e884799d6add9b7fac0228e50c1f8"}, "scope": "cross-repository work", "trigger": "a repository switch occurs", "outcome": "the switch is recorded in the audit trail", "authority": "the runtime", "confidence": "high", "notes": ""}
```

Note what did **not** happen: no `representation`, `workflow`, `stage`,
`rationale`, or `engine_gap` field was added — that is a later stage's
contract, not this one's. And note the mechanism/intent separation this
particular pair of examples does *not* need (there is no old-repo-specific
script or sentinel named in the span) — when a real source span does name
one (a script filename, a specific CLI flag, a sentinel file), keep that
detail in `notes`, and keep `statement` implementation-independent.
