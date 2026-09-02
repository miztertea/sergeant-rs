# Retention engine specification

This spec makes bounded retention real: prune old journal history so that
segments and blobs whose Works are no longer reachable get deleted safely.

## Requirements

- A prune horizon that can never bisect a Work, never delete a live
  reservation, and never remove journal history a running Work still needs.
- A journaled prune intent, followed by quarantine, then unlink, then a
  prune-completed marker — so a crash mid-prune resumes cleanly at the next
  start rather than leaving orphaned journal segments behind.
- Bounded retention means the journal history kept never exceeds the
  configured retention window; anything older is pruned, not archived.

## Consequences

A pruned Work's journal history is gone, not summarized elsewhere. Prune is
destructive by design: bounded retention is the whole point, and a system
that keeps every journal segment forever is not bounded at all.
