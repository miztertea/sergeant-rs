# Sergeant documentation

Sergeant turns accepted intent into durable, staged Work across one repository or a software estate. Start with a tutorial, then use the guides for tasks, concepts for the mental model, workflows for shipped procedure, and reference for exact contracts.

## Start here

- [Install Sergeant](getting-started/install.md)
- [Create your first estate](getting-started/first-estate.md)
- [Complete your first Work](getting-started/first-work.md)
- [Use Captain](getting-started/captain-first.md)
- [Use the CLI directly](getting-started/cli-first.md)

## Operate Sergeant

- [Estate guide](guides/estates.md)
- [Work operations](guides/work.md)
- [Workflow authoring](guides/workflows.md)
- [Harnesses and profiles](guides/harnesses.md)
- [Operations and troubleshooting](guides/operations.md)
- [Automation and observability](guides/automation.md)

## Understand the product

- [Captain and Sergeant](concepts/captain-and-sergeant.md)
- [Estates and Git surfaces](concepts/estates-and-git.md)
- [Host runtime and estates](concepts/host-runtime.md)
- [Hierarchical execution](concepts/hierarchical-execution.md)
- [Work, workflows, and durability](concepts/work-and-workflows.md)
- [Atlas and knowledge sources](concepts/atlas-and-knowledge.md)
- [Security and trust](concepts/security-and-trust.md)

## Exact contracts

- [CLI](reference/cli.md)
- [`sergeant.toml`](reference/sergeant-toml.md)
- [Workflow package and stage schema](reference/workflow-package.md)
- [Work states](reference/work-states.md)
- [Backends and profiles](reference/backends-and-profiles.md)
- [Captain skills](reference/captain-skills.md)
- [Machine interfaces](reference/machine-interfaces.md)
- [Data, environment, and telemetry](reference/runtime.md)
- [Glossary and support](reference/glossary-and-support.md)
- [Workflow catalog](workflows/index.md)
- [Documentation manifest](manifest.md)

The documentation on `main` describes development `main`. A release tag preserves the documentation that shipped with that release. Sergeant is pre-1.0; consult [versioning and support](reference/glossary-and-support.md#versioning-and-support) before automating a contract across upgrades.
