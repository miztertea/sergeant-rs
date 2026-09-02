# Work states

Stage status is orthogonal to Work state.

| State | Meaning | Normal operator door |
|---|---|---|
| `pending` | accepted, not executing | cancel |
| `active` | current execution running | cancel |
| `waiting` | parked on an external condition | retry or cancel |
| `needs_input` | waiting for a human answer | respond or cancel |
| `blocked` | dependency, policy, or ambiguous recovery prevents progress | retry or cancel |
| `completed` | successful and terminal | inspect |
| `failed` | unsuccessful terminal run, explicitly retryable | retry or cancel |
| `canceled` | explicitly stopped and terminal | inspect |

`completed` and `canceled` are absorbing. `failed` can move to `active` only through explicit retry. Extending an exhausted envelope grants turns; it does not itself promise stage re-entry, so inspect and retry as directed.
