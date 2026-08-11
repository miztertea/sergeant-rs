# Partition ledger — run 01KZQ32J2BAD4P8WJA9SWXRMZ9

Per `references/partition-checkpoint-protocol.md`. Seeded from
`../10-inventory/output/inventory.md`'s 21 named partitions (decompose
files only), in that file's recorded order. `Status` is exactly `done` or
`pending`, never a third state.

| Partition | Status | Unit id range | Notes |
|---|---|---|---|
| P1: Root agent policy | done | BU-0001–BU-0060 | AGENTS.md only; dense directive-per-bullet file, high unit count. |
| P2: Product overview, documentation index & help | done | BU-0061–BU-0128 | 6 files: README.md, docs/README.md, docs/what-is-sergeant.md, docs/skills.md, docs/repo-scoped-skills.md, skills/sergeant-help/SKILL.md. |
| P3: Installation, usage, troubleshooting & config schema | done | BU-0129–BU-0213 | 6 files: docs/getting-started.md, docs/using-sergeant.md, docs/troubleshooting.md, docs/schema.md, schema/project.yaml.example, mise.toml. |
| P4: Durable callback protocol | done | BU-0214–BU-0234 | docs/callbacks.md only. |
| P5: Project resolution, status, sync, td-query & graphify | done | BU-0235–BU-0266 | 7 files: bin/sgt-list, bin/sgt-context, bin/sgt-status, bin/sgt-sync, bin/sgt-td-list, bin/sgt-graphify, skills/load-project/SKILL.md. |
| P6: Cross-repo planning & dispatch | done | BU-0267–BU-0312 | 7 files: skills/cross-repo-work/SKILL.md, skills/dispatch/SKILL.md, bin/sgt-dispatch, bin/sgt-td-create, bin/sgt-treehouse-init, bin/_sgt-review-axes.sh, templates/worker-brief.md. |
| P7: Worker lifecycle: interactive session & validation | pending | | |
| P8: Response, wake & recovery | pending | | |
| P9: Drain control | pending | | |
| P10: Fleet monitoring & cleanup | pending | | |
| P11: Escalation & finding routing | pending | | |
| P12: Wiki capture & digest | pending | | |
| P13: DAG-driven dispatch (dagr integration) | pending | | |
| P14: Shared bash foundation | pending | | |
| P15: Vendored single-doc engineering skills (mattpocock/skills) | pending | | |
| P16: Vendored multi-doc skill: codebase-design | pending | | |
| P17: Vendored multi-doc skill: domain-modeling | pending | | |
| P18: Vendored multi-doc skill: prototype | pending | | |
| P19: Vendored multi-doc skill: tdd | pending | | |
| P20: Vendored multi-doc skill: triage | pending | | |
| P21: Sergeant-authored operational skills | pending | | |
