#!/usr/bin/env bash
# Installs the previous release with install.sh, then the latest through a pipe, and checks the layout.
# Needs GH_TOKEN; run from the checkout.
set -e
previous="$(gh release list --repo "$GITHUB_REPOSITORY" --exclude-drafts --exclude-pre-releases --json tagName -q '.[1].tagName')"
latest="$(gh release view --repo "$GITHUB_REPOSITORY" --json tagName -q .tagName)"
echo "previous=$previous latest=$latest"
# With a single release there is nothing to upgrade from; install the latest only.
if [ -n "$previous" ]; then
  JAI_VERSION="$previous" ./install.sh
  test -d "$HOME/.local/share/jai/${previous#v}"
fi
# A PATH without the bin folder (the runners already have ~/.local/bin on theirs).
PATH=/usr/bin:/bin sh < install.sh | tee out.txt
grep -q "is not on your PATH" out.txt
test -d "$HOME/.local/share/jai/${latest#v}"
[ -z "$previous" ] || test ! -e "$HOME/.local/share/jai/${previous#v}"
test "$(readlink "$HOME/.local/bin/jaic")" = "$HOME/.local/share/jai/${latest#v}/jaic"
echo "$HOME/.local/bin" >> "$GITHUB_PATH"
