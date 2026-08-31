# Documentation manifest

This is the implementation manifest for the repository-native product manual. The proposal's page-level questions are intentionally consolidated where one page can answer adjacent questions without duplicating product facts.

| Family | Pages | Owns | Verification |
|---|---:|---|---|
| home and tutorials | 6 | routing, install, first estate, first Work, Captain-first, CLI-first | link and example review |
| concepts | 7 | Captain/Sergeant, estate/Git, host runtime, hierarchical execution, Work/workflow/durability, Atlas/knowledge sources, trust | source and doctrine review |
| guides | 6 | estates, Work, workflows, harnesses, operations, automation | scenario review |
| workflow docs | 1 | published and embedded procedure selection | catalog completeness test |
| reference | 9 | CLI, `sergeant.toml`, workflow schema, states, backends/profiles/skills, machine/runtime contracts, glossary/support | command graph, parser/source, and link tests |

Documentation ownership is narrow:

- `README.md` is the product front door.
- `docs/` is the human manual.
- `AGENTS.md` is Captain's always-on constitution.
- `skills/` is interactive procedure.
- `.sergeant/workflows/` is executable durable procedure.
- `.sergeant/index.md` is the present-tense executable catalog.
- `CONTRIBUTING.md` is the complete public contributor contract.

Release tags version documentation. Public pages may be informed by development history, but install, use, operation, troubleshooting, extension, automation, and contribution must be fully understandable from this repository.
