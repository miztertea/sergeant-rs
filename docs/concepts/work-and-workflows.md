# Work, workflows, and durability

Work is durable accepted intent. It is not a terminal, process, chat session, backend turn, workflow stage, or worktree. Its identity, scope, pinned workflow, execution envelope, state, evidence, and Git outputs survive those mechanisms.

A workflow is versioned filesystem procedure. It contains ordered stages. Each actor stage is a fresh harness execution with its own context; an execute stage is a deterministic Docker invocation. Declared artifacts and repository state carry facts between stages instead of relying on one immortal conversation.

Sergeant journals meaningful transitions and reconstructs projections from that evidence. Recovery does not promise that a process never dies. It promises that retained Work can be reconstructed and either continued when evidence is sufficient or blocked with a reason when it is not.
