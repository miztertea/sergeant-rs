# 10-evaluate-liveness: evaluate liveness

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-snapshot/output/README.md | L4 | upstream artifact produced by `00-snapshot` |

## Purpose

Identity plus recent meaningful progress with a defined fallback chain; a stalled live worker records a non-terminal diagnostic, never an automatic kill.

Trigger (workflow-level): An operator or another workflow (dispatch's `80-monitor`) needs a live view of the fleet.

## What must become true here (durable outcome)

Identity plus recent meaningful progress with a defined fallback chain; a stalled live worker records a non-terminal diagnostic, never an automatic kill.

## Behavior contract

- **Worker health must never be equated with the in_progress status alone; it requires exact live-process identity plus recent, meaningful progress evidence, using a defined fallback chain (pane activity, then recorded progress timestamp, then file mtime only as a last resort), and once that evidence exceeds the grace window the worker stays in_progress but a nonterminal 'live worker stalled' diagnostic is recorded rather than an automatic failure or kill.**
  (trigger: an operator or sgt-watch --sync evaluates whether an in_progress worker is actually healthy; outcome: a stalled worker is distinguished from a healthy one using layered evidence, and the distinction is recorded durably as a nonterminal diagnostic rather than acted on automatically)
  — `BU-P8-072`, `reference/sergeant-upstream/docs/using-sergeant.md` (L161-172 (Worker states))

> **Read `pane`/`tmux` above as this project's durable execution/session identity, not literally.** Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4) — `BU-P8-072` carry a durable identity/liveness/ownership policy that survives the pane; the pane itself does not.

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
