# 30-confirm: read the Docker verify stage's evidence and confirm it

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-docker-check/output/check.txt | L4 | the execute stage's captured evidence to confirm |
| ../../../notes/status.md | L1 | the file `10-touch` edited and the execute stage checked |

## Purpose

Close the actor -> execute -> actor loop: read
`../20-docker-check/output/check.txt` (written by the Docker container,
not by an actor) and confirm, in this stage's own words, that it shows the
expected line count and the `checked-ok` marker. This is the second half
of the N4 proof shape — the execute stage's output evidence is available
to this actor without sergeant interpreting it for you.

## What must become true here (durable outcome)

Append one line to `notes/status.md`, under an `## Confirmed` heading
(create it if absent), stating that the Docker check for this round passed
and citing the line count `check.txt` reported.

## Judgment required

Actually read `check.txt` — do not assume it says what you expect. If it
is missing or does not contain `checked-ok`, say so plainly instead of
confirming.

## Output

Declared in `output/README.md` (Layer 4).
