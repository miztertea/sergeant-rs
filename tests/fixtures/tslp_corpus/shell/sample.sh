#!/usr/bin/env bash
# Hand-counted shell fixture for the F5 corpus gate. Never executed — read as
# bytes by the corpus suite. Lives outside scripts/, so ci.yml's ShellCheck
# job (which globs scripts/ only) does not lint it.
set -euo pipefail

source ./lib/common.sh
. ./lib/extra.sh

LIMIT=8

greet() {
  echo "hello $1"
}

function farewell {
  echo "bye $1"
}

main() {
  greet world
  farewell world
  echo "$LIMIT"
}

main "$@"
