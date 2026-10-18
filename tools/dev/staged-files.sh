#!/bin/sh
# Print the staged files that still exist after the commit, one per line, for one language.
#
#   tools/staged-files.sh [jai|rust|all]      (default: all)
#
# Shared by .githooks/pre-commit and `just fmt|lint --staged`. Plain POSIX sh with git only, so the
# hook needs neither just nor Python. Rust means the workspace (crates/**/*.rs), as in CI.

root=$(git rev-parse --show-toplevel) || exit 1
cd "$root" || exit 1

# Added, copied, modified or renamed: deletions have nothing left to check.
staged=$(git -c core.quotepath=off diff --cached --name-only --diff-filter=ACMR)
case "${1:-all}" in
    jai) printf '%s\n' "$staged" | grep '\.jai$' ;;
    rust) printf '%s\n' "$staged" | grep '^crates/.*\.rs$' ;;
    all) printf '%s\n' "$staged" | grep -e '\.jai$' -e '^crates/.*\.rs$' ;;
    *) echo "usage: staged-files.sh [jai|rust|all]" >&2; exit 2 ;;
esac
# grep exits 1 when nothing matched; an empty list is not an error.
exit 0
