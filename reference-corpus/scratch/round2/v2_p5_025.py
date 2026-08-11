import sys
sys.path.insert(0, '/home/user/sergeant-rs/reference-corpus/scratch/round2')
from util import load, dump, by_id

path = '/home/user/sergeant-rs/reference-corpus/behavior-units/P5.ndjson'
records = load(path)
idx = by_id(records)
r = idx['BU-P5-025']

assert r['representation'] == 'engine-gap'
r['representation'] = 'stage-context'
r['engine_gap'] = None
r['rationale'] = (
    "A sequence of user-confirmation checkpoints embedded in the "
    "30-project-interview stage's own flow, representable today as repeated "
    "needs_input/retry round-trips within that stage rather than a new engine "
    "capability: `src/domain/work.rs`'s `WorkState::can_transition` already "
    "allows `NeedsInput -> Active`, and `src/api.rs`'s `POST /v1/work/{id}/input` "
    "(work.respond) composed with `POST /v1/work/{id}/retry` (re-enter the "
    "current stage) already form the per-question ask/wait/resume loop this "
    "unit's statement requires -- stage-context, not an engine gap (adjudication "
    "A13 overturns X8's engine-gap framing for this unit; see "
    "classification-ledger.md X8)."
)
r['notes'] = (
    (r.get('notes') or '').rstrip()
    + (" " if r.get('notes') else "")
    + "Reclassified off engine-gap per adjudication-round1.md A13 "
      "(ruling on conflict X8): BU-P4-053's 'representable today via the "
      "existing needs_input state, not an engine gap' reading was held to "
      "stand for this unit too, since the shipped needs_input/retry "
      "round-trip already composes into the missing capability engine-pressure.md "
      "G5 had claimed as absent. Representation moves to stage-context of "
      "this unit's owning workflow (sergeant-setup / 30-project-interview); "
      "the retired engine_gap analysis (unbounded question rounds, "
      "why-each-fails reasoning) is preserved in this note's history via "
      "the ledger, not restated here."
)

r['alternatives_considered'] = [
    "one actor stage per phase with the whole interview in one CONTEXT.md turn "
    "(rejected: a bounded headless `claude -p` turn cannot suspend after "
    "question 1 and resume the same reasoning context later, per the "
    "measured Claude adapter's turn model)",
    "one coarse per-phase needs_input round-trip collecting all answers "
    "together (rejected: cannot express the required early-stop behavior "
    "when an early answer ends the interview)",
    "one stage per individual question, statically enumerated (rejected: "
    "expressible only for the fixed-count questions, not for the "
    "per-repository/per-group loops whose round count is not known until "
    "runtime)",
]

dump(path, records)
print("wrote", path)
