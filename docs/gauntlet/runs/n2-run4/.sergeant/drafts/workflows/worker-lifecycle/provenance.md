# provenance — worker-lifecycle

Maps every stage (and the workflow as a whole) to the `behavior_id`(s) from `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`. A stage with no direct source evidence is marked as a justified design inference with a one-line reason, never left silent and never given an invented citation.

## Workflow as a whole

- Evidenced by this candidate's member stages' own citations below (no single record states the workflow-as-a-whole; the aggregate is the union of its stages, per `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).

**Workflow-level helpers** (`stage=null`, apply throughout):

- `BU-0503`
- `BU-0504`
- `BU-0505`
- `BU-0506`
- `BU-0507`
- `BU-0508`
- `BU-0917`
- `BU-0918`

## Stages

### `01-resume-model-pin-reverification`

- Primary behavior_id: `BU-0073` (`README.md (README.md L227-229)`)
- Stage-context attachments: `BU-0344`

### `02-drain-admission-lock`

- Primary behavior_id: `BU-0105` (`README.md (README.md L353-358)`)
- Stage-context attachments: `BU-0521`, `BU-0528`
- Helper attachments: `BU-0543`, `BU-0544`, `BU-0545`, `BU-0546`, `BU-0547`, `BU-0548`, `BU-0549`, `BU-0550`, `BU-0551`, `BU-0552`, `BU-0553`, `BU-0554`, `BU-0555`, `BU-0556`, `BU-0557`, `BU-0558`, `BU-0559`, `BU-0560`, `BU-0561`

### `03-deliver-mission`

- Primary behavior_id: `BU-0137` (`docs/using-sergeant.md (docs/using-sergeant.md L83-86)`)
- Stage-context attachments: `BU-0305`, `BU-0911`, `BU-0912`, `BU-0913`
- Helper attachments: `BU-0907`, `BU-0908`

### `04-bulk-reconcile-fleet-state`

- Primary behavior_id: `BU-0141` (`docs/using-sergeant.md (docs/using-sergeant.md L149-155)`)
- Stage-context attachments: `BU-0142`, `BU-0603`
- Helper attachments: `BU-0607`

### `05-stalled-worker-recovery`

- Primary behavior_id: `BU-0146` (`docs/using-sergeant.md (docs/using-sergeant.md L169-172)`)
- Stage-context attachments: `BU-0159`, `BU-0175`, `BU-0483`, `BU-0484`, `BU-0485`, `BU-0486`, `BU-0487`, `BU-0488`, `BU-0490`, `BU-0492`, `BU-0499`
- Helper attachments: `BU-0480`, `BU-0481`, `BU-0482`

### `06-recover-orphaned-worker`

- Primary behavior_id: `BU-0147` (`docs/using-sergeant.md (docs/using-sergeant.md L182)`)
- Stage-context attachments: `BU-0176`, `BU-0178`, `BU-0306`, `BU-0357`, `BU-0596`, `BU-0597`, `BU-0599`, `BU-0600`

### `07-enter-waiting-state`

- Primary behavior_id: `BU-0148` (`docs/using-sergeant.md (docs/using-sergeant.md L188-190)`)
- Stage-context attachments: `BU-0307`, `BU-0308`

### `08-evaluate-and-resume-wait`

- Primary behavior_id: `BU-0149` (`docs/using-sergeant.md (docs/using-sergeant.md L196-199)`)
- Stage-context attachments: `BU-0150`, `BU-0151`, `BU-0152`, `BU-0453`, `BU-0454`, `BU-0455`, `BU-0456`, `BU-0458`, `BU-0459`, `BU-0460`, `BU-0461`, `BU-0462`, `BU-0463`, `BU-0468`, `BU-0469`, `BU-0470`, `BU-0471`, `BU-0472`, `BU-0473`, `BU-0474`, `BU-0475`, `BU-0476`, `BU-0477`, `BU-0478`, `BU-0479`
- Helper attachments: `BU-0450`, `BU-0451`, `BU-0452`, `BU-0457`, `BU-0464`, `BU-0465`, `BU-0466`, `BU-0467`

### `09-drain-fleet-admission`

- Primary behavior_id: `BU-0153` (`docs/using-sergeant.md (docs/using-sergeant.md L237-243)`)
- Stage-context attachments: `BU-0154`, `BU-0348`, `BU-0423`, `BU-0424`, `BU-0489`, `BU-0515`, `BU-0516`, `BU-0517`, `BU-0519`, `BU-0522`, `BU-0523`, `BU-0524`, `BU-0525`, `BU-0526`, `BU-0527`
- Helper attachments: `BU-0518`, `BU-0520`, `BU-0529`, `BU-0539`, `BU-0540`, `BU-0541`, `BU-0542`, `BU-0562`, `BU-0563`

### `10-respond-to-worker`

- Primary behavior_id: `BU-0155` (`docs/using-sergeant.md (docs/using-sergeant.md L255-262)`)
- Stage-context attachments: `BU-0156`, `BU-0157`, `BU-0177`, `BU-0179`, `BU-0275`, `BU-0405`, `BU-0412`, `BU-0413`, `BU-0414`, `BU-0415`, `BU-0416`, `BU-0417`, `BU-0418`, `BU-0419`, `BU-0420`, `BU-0421`, `BU-0422`
- Helper attachments: `BU-0397`, `BU-0398`, `BU-0399`, `BU-0400`, `BU-0401`, `BU-0402`, `BU-0403`, `BU-0404`

### `11-acknowledge-response`

- Primary behavior_id: `BU-0158` (`docs/using-sergeant.md (docs/using-sergeant.md L275-281)`)
- Stage-context attachments: `BU-0186`, `BU-0436`, `BU-0437`, `BU-0438`, `BU-0439`, `BU-0441`, `BU-0442`, `BU-0443`, `BU-0444`, `BU-0445`, `BU-0446`, `BU-0447`, `BU-0448`, `BU-0449`
- Helper attachments: `BU-0433`, `BU-0434`, `BU-0435`, `BU-0440`

### `12-cleanup-fleet-task`

- Primary behavior_id: `BU-0171` (`docs/using-sergeant.md (docs/using-sergeant.md L403-408)`)
- Stage-context attachments: `BU-0185`, `BU-0191`, `BU-0230`, `BU-0231`, `BU-0611`, `BU-0612`, `BU-0613`, `BU-0614`, `BU-0615`, `BU-0616`, `BU-0617`, `BU-0618`, `BU-0619`, `BU-0620`, `BU-0631`, `BU-0632`, `BU-0633`, `BU-0634`, `BU-0635`, `BU-0636`, `BU-0637`, `BU-0638`, `BU-0639`, `BU-0640`, `BU-0641`, `BU-0642`, `BU-0643`, `BU-0644`, `BU-0645`, `BU-0646`, `BU-0647`, `BU-0648`, `BU-0649`, `BU-0650`, `BU-0651`, `BU-0652`, `BU-0653`, `BU-0654`, `BU-0655`, `BU-0656`, `BU-0657`, `BU-0658`, `BU-0659`, `BU-0660`, `BU-0661`, `BU-0665`, `BU-0666`, `BU-0667`, `BU-0668`, `BU-0669`, `BU-0670`, `BU-0671`, `BU-0672`, `BU-0673`, `BU-0674`, `BU-0675`, `BU-0678`, `BU-0679`, `BU-0805`
- Helper attachments: `BU-0608`, `BU-0609`, `BU-0610`

### `13-retire-response-handshake`

- Primary behavior_id: `BU-0187` (`docs/troubleshooting.md (docs/troubleshooting.md L173-180)`)
- Stage-context attachments: `BU-0188`, `BU-0189`, `BU-0190`, `BU-0624`, `BU-0625`, `BU-0626`, `BU-0627`, `BU-0628`, `BU-0629`, `BU-0630`
- Helper attachments: `BU-0501`

### `14-seal-before-deletion`

- Primary behavior_id: `BU-0232` (`docs/callbacks.md (docs/callbacks.md L176-178)`)
- Stage-context attachments: `BU-0233`, `BU-0676`, `BU-0677`, `BU-0779`, `BU-0806`, `BU-0810`, `BU-0811`, `BU-0812`

### `15-terminate-worker-process`

- Primary behavior_id: `BU-0347` (`bin/sgt-interactive-worker (bin/sgt-interactive-worker L174-182)`)
- Stage-context attachments: `BU-0349`, `BU-0350`, `BU-0351`, `BU-0491`, `BU-0564`, `BU-0565`, `BU-0566`

### `16-worker-exit-cleanup`

- Primary behavior_id: `BU-0354` (`bin/sgt-interactive-worker (bin/sgt-interactive-worker L458-476)`)
- Stage-context attachments: `BU-0355`, `BU-0356`, `BU-0358`

### `17-claim-action-lease`

- Primary behavior_id: `BU-0359` (`bin/sgt-interactive-worker (bin/sgt-interactive-worker L608-626)`)
- Stage-context attachments: `BU-0425`, `BU-0426`, `BU-0493`, `BU-0909`, `BU-0910`
- Helper attachments: `BU-0500`, `BU-0502`, `BU-0509`, `BU-0510`, `BU-0511`, `BU-0512`, `BU-0513`, `BU-0514`

### `18-migrate-legacy-response-state`

- Primary behavior_id: `BU-0406` (`bin/sgt-respond (bin/sgt-respond L70-73)`)
- Stage-context attachments: `BU-0407`, `BU-0408`, `BU-0409`, `BU-0410`, `BU-0411`

### `19-relaunch-superseded-worker`

- Primary behavior_id: `BU-0427` (`bin/sgt-respond (bin/sgt-respond L469-480)`)
- Stage-context attachments: `BU-0428`, `BU-0429`, `BU-0430`, `BU-0431`, `BU-0432`, `BU-0494`, `BU-0495`, `BU-0496`, `BU-0497`, `BU-0498`, `BU-0567`

### `20-force-stop-worker`

- Primary behavior_id: `BU-0530` (`bin/sgt-drain-force (bin/sgt-drain-force L48-56)`)
- Stage-context attachments: `BU-0531`, `BU-0532`, `BU-0533`, `BU-0534`, `BU-0535`, `BU-0536`, `BU-0537`, `BU-0538`

### `21-recycle-terminal-worker-pane`

- Primary behavior_id: `BU-0581` (`bin/sgt-watch (bin/sgt-watch L282-298)`)
- Stage-context attachments: `BU-0582`, `BU-0583`, `BU-0584`, `BU-0585`, `BU-0586`, `BU-0587`, `BU-0588`, `BU-0598`, `BU-0605`

### `22-classify-stalled-worker`

- Primary behavior_id: `BU-0591` (`bin/sgt-watch (bin/sgt-watch L416-426)`)
- Stage-context attachments: `BU-0590`, `BU-0592`, `BU-0593`

### `23-reconcile-incomplete-dispatch`

- Primary behavior_id: `BU-0595` (`bin/sgt-watch (bin/sgt-watch L484-509)`)
- Stage-context attachments: `BU-0589`, `BU-0594`

### `24-stop-validation-pane`

- Primary behavior_id: `BU-0621` (`bin/sgt-cleanup (bin/sgt-cleanup L314-318)`)
- Stage-context attachments: `BU-0622`, `BU-0623`, `BU-0662`, `BU-0663`, `BU-0664`

### `25-start-background-monitor`

- Primary behavior_id: `BU-0919` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L1085-1091 (_sgt_background_watch))`)
- Stage-context attachments: `BU-0920`, `BU-0921`, `BU-0922`

### `26-stop-background-monitor`

- Primary behavior_id: `BU-0923` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L1182-1185 (_sgt_stop_background_monitor))`)
- Stage-context attachments: `BU-0924`

