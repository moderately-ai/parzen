#!/usr/bin/env bash
# Copyright 2026 Thomas Santerre and Moderately AI Inc.
#
# SPDX-License-Identifier: MIT OR Apache-2.0

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if ! command -v addlicense >/dev/null 2>&1; then
  echo "error: addlicense not found; install github.com/google/addlicense@v1.2.0" >&2
  exit 1
fi

args=(
  -f license-header.txt
  -ignore 'target/**'
  -ignore '**/target/**'
  -ignore '.git/**'
  -ignore '.github/**'
  -ignore 'scripts/**'
  -ignore 'comparison-benchmarks/results/analysis/*.html'
  -ignore '**/*.json'
  -ignore '**/*.md'
  -ignore '**/*.toml'
  -ignore '**/*.lock'
  -ignore '**/*.snap'
  -ignore '**/*.sh'
  -ignore '**/*.txt'
)

case "${1:-apply}" in
  apply) addlicense "${args[@]}" . ;;
  check) addlicense -check "${args[@]}" . ;;
  *) echo "usage: $0 [apply|check]" >&2; exit 2 ;;
esac
