#!/usr/bin/env bash
# Workflow-local helper for code-review's 00-pin-fixed-point stage.
# Single consumer — stays local per docs/icm/convention.md §5 rule 3.
#
# Captures the review's diff and commit list once, in the three-dot form
# (comparison against the merge-base, not the fixed point's own tip).
#
# Usage: capture-diff.sh <fixed-point>
set -euo pipefail

fixed_point="$1"

git diff "${fixed_point}...HEAD"
git log "${fixed_point}..HEAD" --oneline
