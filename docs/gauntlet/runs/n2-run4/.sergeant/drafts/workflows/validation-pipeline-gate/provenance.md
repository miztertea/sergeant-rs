# provenance — validation-pipeline-gate

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

**Workflow-level helpers** (`stage=null`, apply throughout):

- `BU-0396`
- `BU-1259`

## Stages

### `01-launch-validation-run`

- Primary behavior_id: `BU-0042` (`AGENTS.md (AGENTS.md L150-157)`)
- Stage-context attachments: `BU-0043`, `BU-0079`, `BU-0080`, `BU-0081`, `BU-0082`, `BU-0161`, `BU-0170`, `BU-0181`, `BU-0182`, `BU-0376`, `BU-0382`, `BU-0383`, `BU-1197`, `BU-1198`, `BU-1199`, `BU-1200`, `BU-1201`, `BU-1202`, `BU-1203`, `BU-1204`, `BU-1205`, `BU-1206`, `BU-1207`, `BU-1208`, `BU-1209`, `BU-1210`, `BU-1211`, `BU-1212`, `BU-1213`, `BU-1214`, `BU-1215`, `BU-1216`, `BU-1217`, `BU-1218`, `BU-1219`

### `02-drive-gate-findings`

- Primary behavior_id: `BU-0084` (`README.md (README.md L283-287)`)
- Stage-context attachments: `BU-0083`, `BU-0085`, `BU-0086`, `BU-0180`, `BU-0310`, `BU-1226`, `BU-1227`, `BU-1228`, `BU-1229`, `BU-1230`, `BU-1231`, `BU-1232`, `BU-1233`, `BU-1234`, `BU-1235`, `BU-1236`, `BU-1237`, `BU-1238`, `BU-1239`, `BU-1240`, `BU-1250`, `BU-1253`, `BU-1254`, `BU-1255`, `BU-1256`, `BU-1257`, `BU-1258`

### `03-recover-from-interrupted-run`

- Primary behavior_id: `BU-0087` (`README.md (README.md L295-298)`)
- Stage-context attachments: `BU-0088`, `BU-1241`, `BU-1242`, `BU-1243`, `BU-1244`, `BU-1245`, `BU-1246`, `BU-1247`, `BU-1248`, `BU-1249`, `BU-1251`, `BU-1252`

### `04-declare-readiness`

- Primary behavior_id: `BU-0160` (`docs/using-sergeant.md (docs/using-sergeant.md L312-316)`)
- Stage-context attachments: `BU-0309`, `BU-0373`, `BU-0374`, `BU-0914`

### `05-acquire-launch-reservation`

- Primary behavior_id: `BU-0162` (`docs/using-sergeant.md (docs/using-sergeant.md L328-331)`)
- Stage-context attachments: `BU-0377`, `BU-0387`

### `06-choose-intent-transport`

- Primary behavior_id: `BU-0163` (`docs/using-sergeant.md (docs/using-sergeant.md L335-338)`)
- Stage-context attachments: `BU-0164`, `BU-0165`, `BU-0166`, `BU-0324`, `BU-0325`, `BU-0375`, `BU-0384`, `BU-0389`, `BU-0394`

### `07-transfer-ownership`

- Primary behavior_id: `BU-0167` (`docs/using-sergeant.md (docs/using-sergeant.md L359-374)`)
- Stage-context attachments: `BU-0168`, `BU-0369`, `BU-0370`, `BU-0371`

### `08-rollback-on-launch-failure`

- Primary behavior_id: `BU-0169` (`docs/using-sergeant.md (docs/using-sergeant.md L384-390)`)
- Stage-context attachments: `BU-0385`, `BU-0386`

### `09-verify-intent-consistency`

- Primary behavior_id: `BU-0326` (`bin/_sgt-intent.sh (bin/_sgt-intent.sh L112-127)`)
- Stage-context attachments: `BU-0372`, `BU-0388`, `BU-0393`

### `10-reset-retryable-state`

- Primary behavior_id: `BU-0378` (`bin/sgt-validate (bin/sgt-validate L630-641)`)
- Stage-context attachments: `BU-0379`

### `11-create-isolated-snapshot`

- Primary behavior_id: `BU-0380` (`bin/sgt-validate (bin/sgt-validate L833-836)`)
- Stage-context attachments: `BU-0381`, `BU-0392`

### `12-check-coordinator-liveness`

- Primary behavior_id: `BU-0390` (`bin/sgt-validation-worker (bin/sgt-validation-worker L73-82)`)
- Stage-context attachments: `BU-0395`

### `13-publish-worker-readiness-handshake`

- Primary behavior_id: `BU-0391` (`bin/sgt-validation-worker (bin/sgt-validation-worker L91-103)`)

### `14-monitor-active-run`

- Primary behavior_id: `BU-1222` (`.agents/skills/no-mistakes/SKILL.md (SKILL.md L107-110)`)
- Stage-context attachments: `BU-1220`, `BU-1221`, `BU-1223`, `BU-1225`
- Helper attachments: `BU-1224`

