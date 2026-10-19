#!/usr/bin/env bash
# Validate a rendered Homebrew formula in a throwaway local tap, for ci.yml and release.yml.
#
#   check_formula.sh FORMULA.rb TAP audit     # brew style and a strict audit (no release URLs needed)
#   check_formula.sh FORMULA.rb TAP install   # also an online audit, a source install, `brew test` and the uninstall
#
# TAP is a `user/name` such as jaitest/local; it is created for the check and removed after.
set -euo pipefail
if [[ $# != 3 || ( $3 != audit && $3 != install ) ]]; then
  echo "usage: $0 FORMULA.rb TAP audit|install" >&2
  exit 2
fi
formula=$1 tap=$2 mode=$3
if [[ "${RUNNER_OS:-}" == Linux ]]; then
  eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"
fi
brew --version
brew tap-new --no-git "$tap"
cp "$formula" "$(brew --repository "$tap")/Formula/jacket.rb"
brew trust "$tap" 2> /dev/null || true
brew style "$tap/jacket"
if [[ $mode == audit ]]; then
  brew audit --formula --strict "$tap/jacket"
else
  brew audit --formula --strict --online "$tap/jacket"
  brew install --build-from-source "$tap/jacket"
  brew test "$tap/jacket"
  brew linkage "$tap/jacket" || true
  brew uninstall "$tap/jacket"
fi
brew untap "$tap"
