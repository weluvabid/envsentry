#!/usr/bin/env sh
set -eu

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

if [ ! -x ./.githooks/pre-commit ]; then
  echo ".githooks/pre-commit is missing or not executable; cannot install Git hooks." >&2
  exit 1
fi

git config --local core.hooksPath .githooks
printf 'Git hooks installed. Pre-commit checks will run from .githooks/pre-commit.\n'
