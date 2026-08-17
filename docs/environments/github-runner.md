# GitHub Actions hosted runner (CI)

Facts measured 2026-08-10 → 2026-08-11 via CI runs on `miztertea/sergeant-rs`.

**Measurement table moved.** The dated measurement table previously here
now lives in `sergeant-rs-workspace`'s knowledge library, at
`knowledge/evidence/host-measurements/github-runner.md` (ADR 0014 decision
18: the capability — `scripts/probe-env.sh` and the rule that a host is
measured before it is trusted — stays with the product; the measurements
themselves are workspace-shaped).

Fixture rule (docs/DEVELOPMENT.md testing section): shapes no hosted-runner user can
change (capabilities, kernel/FS enforcement) skip loudly; locally-fixable
preconditions stay hard failures.
