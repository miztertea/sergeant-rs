#!/usr/bin/env python3
"""A11: rewrite statements that still name old-Sergeant mechanisms (tmux,
pane, sentinel, specific sgt-* script filenames, dotfile artifacts) where the
mechanism is not itself the behavior. Mechanism nouns -> role nouns in
`statement`; the mechanism name is preserved/added in `notes`. ids and
citations (source.*) are untouched. `representation: obsolete-mechanism`
units are deliberately excluded: for those, naming the mechanism *is* the
recorded behavior (what is being retired), not incidental phrasing.
"""
import json

CORPUS = "reference-corpus/behavior-units/P1.ndjson"

# id -> (new_statement, note_addendum_or_None)
REWRITES = {
"BU-P1-001": (
    "Before acting on a named project, resolve its repositories, roles, inherited instructions, and configured paths via a context-resolution step rather than inferring ownership from the current working directory.",
    None,  # notes already names sgt-context as the mechanism
),
"BU-P1-005": (
    "In dispatch mode, create one worker per owning repository via the dispatch procedure.",
    "Old mechanism: sgt-dispatch (bin/sgt-dispatch); the durable behavior is one isolated worker per owning repository, not the specific script.",
),
"BU-P1-008": (
    "In direct mode, resolve project context and the owning task's context before making any edit.",
    "Old mechanism: sgt-context <project> and td context <id>; the durable behavior is establishing both project and task context before mutation.",
),
"BU-P1-026": (
    "Load context: resolve project context and identify the owning repository or repositories, inherited instructions, configured paths, and cross-repository dependencies before selecting an execution mode.",
    "Old mechanism: sgt-context <project>.",
),
"BU-P1-027": (
    "Check the queue: query the task tracker for an existing task and reuse a matching one in direct or dispatch mode; create a task only when no canonical task exists.",
    "Old mechanism: sgt-td-list <project>.",
),
"BU-P1-029": (
    "Reconcile existing state: reconcile fleet state, then inspect active workers, branches, worktrees, retained gates, and handoffs before starting; resume or take over preserved work rather than creating duplicates.",
    "Old mechanism: sgt-watch --sync-all.",
),
"BU-P1-031": (
    "Execute: in direct mode, start the task and implement through tests, review, and delivery; in dispatch mode, launch the dispatch procedure with a repository list or a task id.",
    "Old mechanism: sgt-dispatch <project> --repos <list> or sgt-dispatch <project> --td <id>.",
),
"BU-P1-032": (
    "Monitor real progress: require recent meaningful events or an active child operation plus exact process/execution-context identity — parent-process liveness alone is insufficient; in OpenCode use a managed background watch and verify it started, falling back to bounded one-shot status checks rather than a blocking watch call when unavailable.",
    "'Pane' in the source names old Sergeant's tmux-pane identity; the durable requirement is a verified execution-context identity, not specifically a tmux pane. 'OpenCode' and 'managed background watch' name a supported harness capability, not an obsolete internal mechanism, and are kept.",
),
"BU-P1-036": (
    "A waiting worker may remain alive or may exit after a durable handoff; a deferred wait publishes a durable wake condition and resumes through an automated wake path, while a human decision still resumes through a response-delivery path.",
    "Old mechanism: the wake condition is published as .sergeant-wake-condition; resumption uses sgt-wake for deferred waits and sgt-respond for human decisions.",
),
"BU-P1-038": (
    "Use the response-delivery, wake, or supported recovery paths only after reconciling status, response generation, execution-context identity, and handoff evidence.",
    "Old mechanism: sgt-respond, sgt-wake; 'pane identity' names old Sergeant's tmux-pane identity, generalized here to execution-context identity.",
),
"BU-P1-039": (
    "Every dispatched implementation, independent review, PR description, successor, recovery, and final shipping gate must use the same canonical intent revision from the canonical intent record.",
    "Old mechanism: the intent record is the file .sergeant-intent.md.",
),
"BU-P1-041": (
    "After readiness, the coordinator launches the single coordinator-owned validation-only boundary.",
    "The durable rule is 'exactly one coordinator-owned validation boundary'; 'sgt-validate' and 'split pane of the worker's tmux window' are old Sergeant's mechanism and belong in obsolete-mechanisms.md, not the durable procedure.",
),
"BU-P1-042": (
    "The validation boundary's default medium profile skips the redundant no-mistakes review and document stages.",
    "Old mechanism: sgt-validate's default 'medium' profile.",
),
"BU-P1-047": (
    "Do not call a worker active solely because its process or execution context exists; require recent meaningful progress evidence.",
    None,  # notes already cross-reference the canonical Worker definition; add pane clarification inline below
),
"BU-P1-053": (
    "Context resolution resolves instructions in order: defaults.agent_instructions, then group instructions, then repository instructions; later layers override earlier ones for the same repository.",
    "Old mechanism: sgt-context performs this resolution.",
),
"BU-P1-057": (
    "The dispatch invocation's agent selector (flag or environment variable) may select opencode, oc, goose, claude, or an equivalent path whose basename is one of those names; dispatch uses only persistent interactive sessions and rejects every other agent and all non-interactive launch modes before creating worker state.",
    "Old mechanism: SERGEANT_AGENT env var or sgt-dispatch --agent flag.",
),
"BU-P1-058": (
    "The dispatch invocation's model selector (flag or environment variable) pins the harness model as provider/model[:variant]; the agent selector and model selector are orthogonal; precedence is the explicit flag, then the environment variable, then the harness's ambient default, and an unpinned dispatch is recorded as unpinned rather than left blank.",
    "Old mechanism: sgt-dispatch --model flag or SERGEANT_MODEL env var; --agent/SERGEANT_AGENT for the executable.",
),
"BU-P1-080": (
    "The no-mistakes run is validation-only and must not fix findings; actionable findings are routed into separate, deduplicated owning-repository task-tracker work via the finding-routing mechanism.",
    "Old mechanism: sgt-no-mistakes-finding.",
),
"BU-P1-085": (
    "Generated worker briefs require one independent review per axis named in the shared review-axes definition — Standards, Spec, and Readiness always, plus a conditional Accessibility axis whenever frontend/UI/visual/interaction/accessibility/user-facing language appears in the mission, repo role, or repo group — and that one definition drives both the brief's requirement and the review-finding router's accepted axes so the two cannot drift.",
    "Old mechanism: the shared definition lives in SGT_REVIEW_AXES_REQUIRED (bin/_sgt-review-axes.sh); the router is sgt-review-findings.",
),
"BU-P1-086": (
    "Dispatched workers pass each axis's strict JSON finding artifact to the review-finding router, which creates or updates one owning-repository task per actionable finding, preserves active task state on reruns, and publishes blocking task IDs and remediation guidance through the worker's status/message channel and a notification mechanism; cosmetic and false-positive dispositions create no cards.",
    "Old mechanism: the router is sgt-review-findings; the status/message channel is .sergeant-message and .sergeant-status; the notification mechanism is sgt-notify.",
),
"BU-P1-090": (
    "When a route fails after parsing, sanitized findings are retained under a review-artifacts directory scoped to the worktree, axis, and source, holding only post-redaction fields, the blocked message names the exact retry command, a retry re-validates every field and recomputes each content digest, and a route refuses to overwrite an artifact nobody has retried yet.",
    "Old mechanism: the directory is <worktree>/.sergeant-review-artifacts/<axis>-<source>/.",
),
"BU-P1-099": (
    "Each installation owns its own project configuration, local Git credentials and GitHub account selection, worktrees or Treehouse leases, fleet worker state, local execution sessions and agent processes, and local wiki captures and knowledge graphs.",
    "Old mechanism: 'local execution sessions' names old Sergeant's tmux panes; D2 already establishes the sergeant-rs daemon has no TTY/pane, so the durable fact is that per-installation session/process state is never shared, not the tmux mechanism specifically.",
),
"BU-P1-105": (
    "A Worker is an agent running in an isolated worktree and execution session; a live process is not proof of progress, and recent meaningful progress evidence is required.",
    "Old mechanism: 'execution session' names old Sergeant's tmux pane, per the same D2 supersession noted on the adjacent worker-lifecycle units.",
),
}


def append_note(existing, addendum):
    if addendum is None:
        return existing
    if existing is None or existing == "":
        return addendum
    return existing + " " + addendum


def main():
    with open(CORPUS, "r", encoding="utf-8") as f:
        units = [json.loads(l) for l in f if l.strip()]

    applied = 0
    for u in units:
        if u["id"] not in REWRITES:
            continue
        assert u["representation"] != "obsolete-mechanism", (
            f"{u['id']} is obsolete-mechanism; A11 rewrite should not apply"
        )
        new_stmt, note_add = REWRITES[u["id"]]
        u["statement"] = new_stmt
        u["notes"] = append_note(u["notes"], note_add)
        applied += 1

    with open(CORPUS, "w", encoding="utf-8") as f:
        for u in units:
            f.write(json.dumps(u, ensure_ascii=False) + "\n")

    print(f"Rewrote {applied} statements for A11.")


if __name__ == "__main__":
    main()
