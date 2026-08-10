#!/usr/bin/env python3
"""P6 fixer: targeted overrides on top of the generic first-range extraction.

Handles:
  (a) A2 span-choice overrides for units where the *first* cited range is a
      thin/generic anchor and a *later* cited range (or, for BU-P6-039, a
      range adjacent to what was cited but omitted) is the real supporting
      evidence for the statement's operative claim.
  (b) A10: BU-P6-129 demoted + narrowed to what its own quote supports, with
      a new split-off unit (BU-P6-143) carrying the ownership-claim
      sub-behavior on its own strong citation. BU-P6-082 narrowed similarly
      (finding N1-BH-08's "equally weak" citation).
  (c) A11: statement rewrites removing mechanism nouns (tmux/pane) in favor
      of role nouns, for units whose representation is a live rung (not
      obsolete-mechanism, where naming the mechanism IS the point).

Same span convention as extract_quotes_p6.py: first char of start line to
last char of end line, no trailing newline, sha256 over exact UTF-8 bytes.
"""
import json
import hashlib

CORPUS = "reference-corpus/behavior-units/P6.ndjson"


def load_lines(path, cache={}):
    if path not in cache:
        with open(path, "r", encoding="utf-8", newline="") as f:
            cache[path] = f.read().splitlines(keepends=True)
    return cache[path]


def span_for(path, start, end):
    lines = load_lines(path)
    span = "".join(lines[start - 1:end])
    if span.endswith("\n"):
        span = span[:-1]
    if span.endswith("\r"):
        span = span[:-1]
    return span


def set_quote(src, path, start, end):
    span = span_for(path, start, end)
    span_bytes = span.encode("utf-8")
    digest = "sha256:" + hashlib.sha256(span_bytes).hexdigest()
    if len(span) <= 500:
        src["quote"] = span
        src.pop("span_bytes", None)
    else:
        src["quote"] = span[:500]
        src["span_bytes"] = len(span_bytes)
    src["quote_hash"] = digest
    return span


def main():
    with open(CORPUS, "r", encoding="utf-8") as f:
        units = [json.loads(l) for l in f if l.strip()]
    by_id = {u["id"]: u for u in units}

    # ---- (a) A2 span-choice overrides ----

    # BU-P6-025: first range is just the marker-format assignment; the
    # statement's decisive claim ("more than one match -> refuse rather than
    # pick one arbitrarily") is the second cited range.
    u = by_id["BU-P6-025"]
    path = u["source"]["path"]
    set_quote(u["source"], path, 160, 162)
    u["notes"] = (u.get("notes") or "")
    u["notes"] = ("Locator cites two spans (L107 for the marker format, L160-162 for "
                  "the refuse-on-ambiguous-match behavior); the quoted span is the "
                  "latter since it is the statement's decisive, more heavily-weighted "
                  "claim. The marker format (L107) is corroborating context, not "
                  "separately quoted.") if not u["notes"] else u["notes"] + (
                  " Locator cites two spans; the quoted span (L160-162) is the "
                  "refuse-on-ambiguous-match behavior, the statement's decisive claim.")

    # BU-P6-039: none of the three originally-cited ranges (L1-4 script
    # header, L45-46 flag-presence check, L58-62 confirmation gate) actually
    # evidences the statement's lead clause ("refused unless a cooperative
    # drain is already active"). That check exists in the file at L48-55,
    # immediately adjacent to the cited L45-46. Locator corrected to cite it.
    u = by_id["BU-P6-039"]
    path = u["source"]["path"]
    set_quote(u["source"], path, 48, 55)
    u["source"]["locator"] = "L48-55, L58-62"
    u["notes"] = (
        "Original locator (L1-4, L45-46, L58-62) omitted the actual "
        "drain-is-active check; corrected to L48-55 (the check itself) plus "
        "L58-62 (the --yes/--dry-run confirmation gate), dropping the script "
        "header and the unrelated --global/--project presence check that were "
        "cited but did not support the statement."
    )

    # BU-P6-136: first range explains *why* exit code 2 exists (rationale);
    # the second range is the function itself and its docstring, which is a
    # near-verbatim source for the statement's core claim.
    u = by_id["BU-P6-136"]
    path = u["source"]["path"]
    set_quote(u["source"], path, 1028, 1042)
    u["notes"] = (u.get("notes") or "")
    extra = (" Locator cites two spans; the quoted span (L1028-1042) is "
             "_require_owning_td_task_closed's own definition and docstring, "
             "which states the not-closed-vs-unlookupable distinction directly. "
             "L988-992 (kept in the locator) is the exit-code-2 rationale, "
             "corroborating context rather than the primary quote.")
    u["notes"] = extra.strip() if not u["notes"] else u["notes"] + extra

    # ---- (b) A10: BU-P6-129 split + BU-P6-082 narrowing ----

    # BU-P6-129 as originally written cited only its own 83-char header
    # comment (L2) for a four-part compound claim. Each of the four parts
    # already has its own, far stronger, independently-cited unit elsewhere
    # in this partition:
    #   (1) readiness marker proving commit+axes  -> BU-P6-130 (L236-269)
    #   (2) isolated validation code snapshot      -> BU-P6-133 (L829-858)
    #   (3) ownership (mechanism half)              -> BU-P6-131 (L83-86)
    #   (3) ownership (durable rule half)            -> NEW BU-P6-143 (L282-308)
    #   (4) "every precondition checked before any state is committed" is an
    #       emergent property of (1)+(2)+(3) each independently gating in
    #       sequence before the tool ever runs -- not itself a separate
    #       quotable line, so it is not split into its own unit; it is
    #       recorded here as a note rather than re-asserted as a citable fact.
    # BU-P6-129 itself is demoted to only the coarse trigger+outcome claim
    # its actual (thin) citation supports, per A10 / finding N1-BH-08.
    u = by_id["BU-P6-129"]
    u["statement"] = (
        "Launching the shipping-gate validator (sgt-validate) is its own "
        "bounded, independently invocable top-level procedure -- a "
        "coordinator-owned validation run launched beside an interactive "
        "worker's task -- distinct from the worker's own mission; the specific "
        "preconditions that gate it (readiness marker, isolated snapshot, "
        "resolved ownership) are separately recorded, each on its own "
        "stronger citation, as sibling units of this same workflow."
    )
    u["confidence"] = "low"
    u["notes"] = (
        "N1 adjudication A10 (finding N1-BH-08): the original statement "
        "asserted four requirements (readiness marker, isolated snapshot, "
        "ownership, preconditions-before-commit) from a single 83-character "
        "header-comment quote (L2) and carried confidence: high -- an "
        "over-claim the citation could not support. Demoted to confidence: low "
        "and narrowed to only the coarse workflow-boundary claim the header "
        "comment actually states. The four original sub-claims now rest on "
        "their own, stronger citations: BU-P6-130 (readiness marker), "
        "BU-P6-133 (isolated snapshot), BU-P6-131 + BU-P6-143 (ownership, "
        "mechanism and durable-rule halves), and this unit's own rationale "
        "(preconditions-before-commit, an emergent property of those three "
        "gates running in sequence before sgt-validate ever invokes "
        "no-mistakes -- not itself an independently quotable line, so it is "
        "not re-asserted as a separate citable fact)."
    )
    u["rationale"] = (
        "Recognizable trigger (worker's code is ready for shipping-gate "
        "validation) and bounded, independently invocable outcome (a "
        "validation run either launches under a single accountable owner or "
        "is refused) still matches the §6.2 workflow test at the coarse "
        "grain -- but the record now claims only what its own citation "
        "supports, not the compound detail that belongs to its sibling "
        "stage/stage-context units."
    )
    u["alternatives_considered"] = [
        "stage of a larger validate-and-ship workflow (rejected: this is "
        "already the top-level launch procedure the validate-and-ship "
        "package's own stages compose around, not a step inside one)",
        "fold into dispatch (rejected: validation launches on its own "
        "trigger -- worker readiness -- independent of and later than the "
        "original dispatch)",
    ]

    # New split-off unit: the durable ownership rule BU-P6-129 gestured at,
    # now on its own real, strong citation (distinct from BU-P6-131, which
    # covers only the process-chain *mechanism* half of ownership proof).
    src_path = "reference/sergeant-upstream/bin/sgt-validate"
    new_src = {"path": src_path, "locator": "L282-308"}
    span = set_quote(new_src, src_path, 282, 308)
    new_unit = {
        "id": "BU-P6-143",
        "statement": (
            "A validation run's ownership can only pass from the original "
            "dispatching coordinator to a different claimant through an "
            "explicit ownership-claim request; resuming from the original "
            "owning coordinator re-asserts ownership automatically, but any "
            "other caller is refused validation unless it explicitly claims "
            "ownership, and a caller that is neither the original owner nor "
            "an explicit claimant is refused outright even if the original "
            "owner still appears live."
        ),
        "source": new_src,
        "scope": "shipping-gate launch / ownership",
        "trigger": "a validation run is about to launch and its coordinator ownership must be resolved",
        "outcome": "exactly one accountable coordinator ever owns a validation run, and only ever by original dispatch or explicit claim, never by default or inference",
        "authority": "coordinator",
        "confidence": "high",
        "notes": (
            "Split off BU-P6-129 (N1 adjudication A10 / finding N1-BH-08): "
            "this is the durable ownership *rule* half; BU-P6-131 records "
            "the (obsolete-mechanism) process-chain *proof* used to verify a "
            "claim, which is a different, mechanism-specific fact."
        ),
        "representation": "stage",
        "workflow": "no-mistakes",
        "stage": "resolve-ownership",
        "rationale": (
            "A durable checkpoint (who owns this validation run, and how did "
            "ownership arrive there) whose only-original-or-explicit-claim "
            "semantics would matter under any implementation -- matches "
            "§6.3's stage test, distinct from the process-chain verification "
            "mechanism (BU-P6-131) used to check a claim's legitimacy."
        ),
        "alternatives_considered": [
            "stage-context of the launch-worker checkpoint (rejected: this is "
            "load-bearing enough, and gates independently enough, to be its "
            "own durable checkpoint rather than a detail of another one)",
            "agents-invariant (rejected: it is specific to the validation-launch "
            "procedure's own ownership handshake, not a stage-independent rule "
            "applying uniformly across every procedure)",
        ],
        "engine_gap": None,
    }
    units.append(new_unit)

    # BU-P6-082: same failure shape as BU-P6-129 (finding N1-BH-08) -- a
    # four-part compound claim resting on its own 78-char header comment
    # (L2). The parse/sanitize, retain-before-side-effect, and dedup-routing
    # sub-claims are already independently and more strongly evidenced by
    # BU-P6-087, BU-P6-084, and BU-P6-085/086 respectively. What is NOT
    # independently covered is the sequencing guarantee ("only once every
    # finding has reached tracked work does the gate get decided") -- and
    # that guarantee has real, strong, contiguous evidence at L672-691. The
    # unit is narrowed to that claim and re-cited there instead of L2.
    u = by_id["BU-P6-082"]
    path = u["source"]["path"]
    set_quote(u["source"], path, 672, 691)
    u["statement"] = (
        "Independent review routing decides whether to publish a blocking "
        "gate only after every finding from the current review pass has "
        "already reached tracked work (routed to a task or explicitly "
        "refused with a stated reason) -- the gate decision is never made, "
        "and no finding is ever left unrouted, mid-loop."
    )
    u["confidence"] = "high"
    u["rationale"] = (
        "Recognizable trigger (a review pass completes) and a bounded "
        "outcome (findings are fully routed, then exactly one gate decision "
        "is made) still matches the workflow test at §6.2 -- narrowed from "
        "the original four-part claim to the one sub-claim this unit's own "
        "citation directly supports; parsing/sanitizing, retain-before-side-"
        "effect, and dedup routing are independently covered by BU-P6-087, "
        "BU-P6-084, and BU-P6-085/086 on their own stronger citations."
    )
    u["notes"] = (
        "N1 adjudication A10, applied by analogy per finding N1-BH-08 "
        "('update BU-P6-082 similarly if its citation is equally weak'): the "
        "original statement asserted parse+sanitize, retain-before-side-"
        "effect, dedup-to-one-task, AND gate-only-after-full-routing from a "
        "single 78-character header comment (L2). Re-anchored to L672-691, "
        "the actual sequencing evidence, and narrowed to only that claim; "
        "the other three sub-claims already have their own stronger-cited "
        "sibling units (BU-P6-084, BU-P6-085, BU-P6-086, BU-P6-087) and are "
        "not re-asserted here."
    )
    u["alternatives_considered"] = [
        "stage of no-mistakes (rejected: independently triggerable by any "
        "review pass -- standards, spec, readiness -- not exclusively a "
        "no-mistakes sub-step)",
        "fold into BU-P6-084 (rejected: BU-P6-084 is about retention before "
        "any side effect; this is a distinct ordering guarantee about the "
        "gate decision specifically, made after the routing loop, not "
        "before it)",
    ]

    # ---- (c) A11: mechanism-noun -> role-noun statement rewrites ----

    def rewrite(unit_id, new_statement, note_addition=None):
        uu = by_id[unit_id]
        uu["statement"] = new_statement
        if note_addition:
            uu["notes"] = note_addition if not uu.get("notes") else uu["notes"] + " " + note_addition

    rewrite(
        "BU-P6-017",
        "A project's DAG is declared as data in the project YAML — named "
        "stages, each naming its repos, its td task or literal brief, and the "
        "stages it depends on (after:) — and the DAG-orchestration entry "
        "point reads that declaration to create/verify the DAG and start a "
        "run, rather than the DAG being written as imperative dispatch calls.",
        "(A11: mechanism noun normalized — the entry point is the script bin/sgt-dag-run.)",
    )
    rewrite(
        "BU-P6-028",
        "The durably recorded event (event=<kind>, updated=<timestamp> "
        "written under fleet state) is the source of truth for a worker "
        "update; any live delivery transport to a coordinator's active "
        "session is an optional compatibility layer on top, and its own "
        "failure to reach a coordinator must not be treated as failure to "
        "record the update.",
    )
    rewrite(
        "BU-P6-055",
        "An action lease is only ever settled as completed from the agent's "
        "own durable proof, written as exactly '<notification_id>|<lease>' at "
        "a fixed location; nothing may fabricate completion from an "
        "inference about apparent process idleness — a mismatched "
        "identity, nonce, missing target, or absent proof always fails "
        "closed and is recorded as pending with its reason.",
        "(A11: mechanism noun normalized — the old implementation's rejected inference was literally 'the tmux pane looks idle'.)",
    )
    rewrite(
        "BU-P6-058",
        "A concurrent 'read drain state, then start new work' race is closed "
        "by an explicit admission lock that every dispatch/relaunch "
        "procedure and every drain-set/undrain procedure must acquire before "
        "reading or writing drain state, so a drain set mid-dispatch is "
        "never silently missed.",
        "(A11: mechanism nouns normalized — old mechanism: bin/sgt-dispatch and bin/sgt-respond on the dispatch/relaunch side, bin/sgt-drain on the drain-set/undrain side, each launching a new tmux pane.)",
    )
    rewrite(
        "BU-P6-066",
        "One registry not three drifting definitions is itself the durable "
        "design principle worth preserving independent of the specific "
        "harness-launch mechanism: any capability that gates admission, is "
        "probed for readiness, and is then invoked must derive all three "
        "decisions from the same single declaration, never three "
        "independently maintained ones.",
        "(A11: mechanism noun normalized — the concrete mechanism is a tmux-pane-based harness launch.)",
    )
    rewrite(
        "BU-P6-072",
        "During recovery, the replacement worker is only launched, and its "
        "identity validated, before the original stalled worker instance is "
        "ever terminated, so that any failure in the relaunch sequence "
        "leaves the original stalled process intact for investigation "
        "rather than losing the supervisor entirely.",
        "(A11: mechanism noun normalized — the concrete mechanism kills a tmux pane.)",
    )
    rewrite(
        "BU-P6-073",
        "A recovery attempt refuses to proceed while an unfinished "
        "notification action-lease exists, unless the lease's owner is "
        "provably dead (its execution target no longer resolves at all and "
        "its recorded process is not running) — anything else, including "
        "a target that merely looks idle, must fail closed to preserve "
        "exact-once delivery evidence.",
        "(A11: mechanism noun normalized — the concrete execution target is a tmux pane.)",
    )
    rewrite(
        "BU-P6-075",
        "A recovery attempt runs every pre-flight validation — stall "
        "proof, lease convergence, drain check, relaunch-metadata "
        "completeness, prior-execution-instance identity — to completion "
        "before stamping the attempt as made, so that any pre-flight "
        "failure leaves the stalled worker untouched and eligible for a "
        "real recovery attempt later.",
        "(A11: mechanism noun normalized — the concrete instance identity checked is a tmux pane identity.)",
    )
    rewrite(
        "BU-P6-080",
        "A response relaunch never allows a second, superseding worker "
        "instance to displace the first without preserving the first "
        "instance's superseded notification-target identity as evidence "
        "— and if that evidence would conflict with already-recorded "
        "evidence, the relaunch refuses outright rather than losing the "
        "older evidence.",
        "(A11: mechanism noun normalized — the concrete instance is a tmux pane.)",
    )
    rewrite(
        "BU-P6-101",
        "A bounded, side-effect-free activity snapshot answers exactly one "
        "narrow question — is Sergeant verifiably doing work right now "
        "— as constant-size versioned JSON, and reports busy:true only "
        "when ALL of a stable in_progress status, an exact live worker "
        "execution-instance identity match, and recent progress "
        "attributable to that exact instance hold together; every other "
        "outcome is busy:null, because absence of a verified witness is "
        "never treated as proof of idleness.",
    )
    rewrite(
        "BU-P6-102",
        "The bounded activity snapshot is strictly read-only by design: it "
        "runs before any reconciliation, never writes anything, and "
        "deliberately avoids the execution-identity-migration side effect "
        "that the ordinary status-printing path performs, because a "
        "side-effect-free observation surface must never silently mutate "
        "state as a byproduct of being queried.",
        "(A11: mechanism noun normalized — the concrete identity migrated is a tmux pane identity.)",
    )
    rewrite(
        "BU-P6-104",
        "Retiring a terminal (done, failed, or drained) worker's "
        "execution-instance recycling evidence is bound to the exact "
        "execution-instance identity being retired, not merely stamped as a "
        "permanent task-level marker — because binding to the wrong "
        "scope (any prior recycling ever) permanently suppressed recycling "
        "of every later relaunched instance once one instance had ever been "
        "recycled.",
        "(A11: mechanism noun normalized — the concrete instance is a tmux pane.)",
    )
    rewrite(
        "BU-P6-105",
        "Recycling a terminal worker's execution instance first settles its "
        "accepted notification action-lease before the instance is taken "
        "away, because recycling used to stop the only process that could "
        "ever publish completion, which is exactly how a completed turn "
        "became permanently unrecoverable.",
        "(A11: mechanism noun normalized — the concrete instance is a tmux pane.)",
    )
    rewrite(
        "BU-P6-123",
        "Dispatch is a bounded, independently invocable procedure: given a "
        "project, a brief (or a tracked-work task reference), and a set of "
        "target repos, it produces one durable task with an isolated "
        "worktree, a rendered mission brief, and a spawned interactive "
        "worker per repo — with every side effect (tracked-work creation, "
        "worktree acquisition, worker-process launch) validated and gated "
        "before the next repo's dispatch begins.",
        "(A11: mechanism noun normalized — the concrete launch is a tmux pane launch.)",
    )

    with open(CORPUS, "w", encoding="utf-8") as f:
        for uu in units:
            f.write(json.dumps(uu, ensure_ascii=False) + "\n")

    print(f"Wrote {len(units)} units.")


if __name__ == "__main__":
    main()
