#!/usr/bin/env bash
# Replaces the main branch of matteopolak/jacket.nvim with editors/nvim and tags it. Needs TAG, VERSION,
# EDITOR_VERSION and DEPLOY_KEY (a write-enabled deploy key); run from the checkout.
set -e
if [ -z "$TAG" ]; then echo "No tag (dry run); not pushing the plugin"; exit 0; fi
if [ -z "$DEPLOY_KEY" ]; then echo "NVIM_MIRROR_DEPLOY_KEY is not set; skipping the Neovim plugin mirror"; exit 0; fi
key="$RUNNER_TEMP/mirror_key"
known="$RUNNER_TEMP/known_hosts"
printf '%s\n' "$DEPLOY_KEY" > "$key"
chmod 600 "$key"
# GitHub's published ed25519 host key fingerprint, so the push cannot go to an impostor.
ssh-keyscan -t ed25519 github.com > "$known" 2> /dev/null
ssh-keygen -lf "$known" | grep -q 'SHA256:+DiY3wvvV6TuJJhbpZisF/zLDA0zPMSvHdkr4UvCOqU' || { echo "unexpected github.com host key"; exit 1; }
export GIT_SSH_COMMAND="ssh -i $key -o IdentitiesOnly=yes -o UserKnownHostsFile=$known -o StrictHostKeyChecking=yes"
git clone git@github.com:matteopolak/jacket.nvim.git "$RUNNER_TEMP/mirror"
cd "$RUNNER_TEMP/mirror"
git checkout -B main
git rm -rq --ignore-unmatch .
cp -R "$GITHUB_WORKSPACE/editors/nvim/." .
cp "$GITHUB_WORKSPACE/LICENSE" "$GITHUB_WORKSPACE/LICENSE-EXCEPTION" .
git add -A
if git diff --cached --quiet; then
  echo "The mirror already has this content"
else
  git -c user.name='github-actions[bot]' -c user.email='41898282+github-actions[bot]@users.noreply.github.com' \
    commit -q -m "jacket.nvim $EDITOR_VERSION (toolchain $VERSION)"
fi
if git ls-remote --exit-code --tags origin "refs/tags/v$EDITOR_VERSION" > /dev/null; then
  git fetch -q origin "refs/tags/v$EDITOR_VERSION:refs/tags/v$EDITOR_VERSION"
  tagged_commit="$(git rev-parse "refs/tags/v$EDITOR_VERSION^{commit}")"
  current_commit="$(git rev-parse HEAD)"
  test "$tagged_commit" = "$current_commit" || {
    echo "Mirror tag v$EDITOR_VERSION already points to different plugin content; publish a new editor_version instead of retagging." >&2
    exit 1
  }
  echo "The mirror tag v$EDITOR_VERSION already points to this content"
  git push origin main
else
  git -c user.name='github-actions[bot]' -c user.email='41898282+github-actions[bot]@users.noreply.github.com' \
    tag -a "v$EDITOR_VERSION" -m "jacket.nvim $EDITOR_VERSION (toolchain $VERSION)"
  git push origin main "v$EDITOR_VERSION"
fi
echo "Pushed jacket.nvim $EDITOR_VERSION for toolchain $VERSION to matteopolak/jacket.nvim"
