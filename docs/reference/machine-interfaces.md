# Machine interfaces

## JSON

Place global `--json` before or after the subcommand as accepted by Clap. Nonstreaming commands emit one JSON value. Consumers must key by documented fields and tolerate additive fields within a release line.

## Watch

The schema is `sergeant.watch/v1`. `sgt watch <id>` is scoped; omitting the ID observes the estate. `--follow` continues across notices. JSON mode is JSONL. A notice contains a reason, trigger provenance, and a fresh snapshot. Attention states are waiting, needs-input, blocked, failed, completed, and canceled; terminal behavior follows the current command flags.

## API

The v1 service is loopback HTTP/JSON plus SSE. `/healthz` is unauthenticated; `/v1/*` requires bearer authentication. Mutations journal accepted or rejected outcomes and are idempotent by caller command ID. Errors are structured JSON. SSE sends domain events plus named floor and stream-error control frames and periodic keepalive.

The API, CLI JSON, and watch protocol are pre-1.0 contracts. Bind automation to a release and verify fixtures on upgrade.
