#!/bin/sh
# Print the staged files that still exist after the commit, one per line, for one language.
#
#   tools/dev/staged-files.sh [jai|rust|shell|shell-all|all]      (default: all)
#
# Shared by .githooks/pre-commit, `just fmt|lint --staged` and CI's shellcheck job. Plain POSIX sh
# with git only, so the hook needs neither just nor Python. Rust means the workspace
# (crates/**/*.rs), as in CI. Shell means *.sh files and files whose first line is a sh, bash, dash
# or ksh shebang; `shell` lists the staged ones, `shell-all` every file in the index. Neither is
# part of `all`, which is what the formatters take.

root=$(git rev-parse --show-toplevel) || exit 1
cd "$root" || exit 1

# Added, copied, modified or renamed: deletions have nothing left to check.
staged=$(git -c core.quotepath=off diff --cached --name-only --diff-filter=ACMR)

# Every file in the index that is a shell script, sorted.
shell_files() {
    {
        git -c core.quotepath=off ls-files '*.sh'
        git -c core.quotepath=off grep --cached -I -n -E '^#!.*[/ ](ba|da|k)?sh( |$)' |
            sed -n 's/^\(.*\):1:#!.*/\1/p'
    } | sort -u
}

staged_shell_files() {
    scripts=$(shell_files)
    printf '%s\n' "$staged" | while IFS= read -r f; do
        [ -n "$f" ] && printf '%s\n' "$scripts" | grep -q -x -F -e "$f" && printf '%s\n' "$f"
    done
}

case "${1:-all}" in
    jai) printf '%s\n' "$staged" | grep '\.jai$' ;;
    rust) printf '%s\n' "$staged" | grep '^crates/.*\.rs$' ;;
    shell) staged_shell_files ;;
    shell-all) shell_files ;;
    all) printf '%s\n' "$staged" | grep -e '\.jai$' -e '^crates/.*\.rs$' ;;
    *) echo "usage: staged-files.sh [jai|rust|shell|shell-all|all]" >&2; exit 2 ;;
esac
# grep exits 1 when nothing matched; an empty list is not an error.
exit 0
