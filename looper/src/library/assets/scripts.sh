#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

move_to_next_version() {
  while true; do
    git fetch --prune

    child_commit="$(git rev-list --ancestry-path --first-parent \
      HEAD.."${KEREK_GIT_BRANCH}" | tail -1)"

    if [[ -n "${child_commit}" ]]; then
      echo "Checking out Git commit ${child_commit}."
      git checkout "${child_commit}"
      break
    fi

    sleep "$(("${RANDOM}" % 20))s"
  done
}

main() {
  "$1"
}

main "$@"
