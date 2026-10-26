#!/usr/bin/env bash
# Creates the GitHub release for TAG from dist/ and notes.md, or updates it when it exists and is mutable.
# Needs GH_TOKEN, TAG and CURRENT_VERSION.
set -e
if gh release view "$TAG" --repo "$GITHUB_REPOSITORY" --json isImmutable > release.json 2> release-error.txt; then
  if [ "$(node -p 'require("./release.json").isImmutable')" = "true" ]; then
    echo "GitHub release $TAG is immutable; its archives and checksums cannot be replaced safely." >&2
    exit 1
  fi
  gh release upload "$TAG" dist/* --repo "$GITHUB_REPOSITORY" --clobber
  if [ "$TAG" = "v$CURRENT_VERSION" ]; then
    gh release edit "$TAG" --repo "$GITHUB_REPOSITORY" --title "Jacket ${TAG#v}" --notes-file notes.md --latest
  else
    gh release edit "$TAG" --repo "$GITHUB_REPOSITORY" --title "Jacket ${TAG#v}" --notes-file notes.md
  fi
elif grep -qi 'release not found' release-error.txt; then
  if [ "$TAG" = "v$CURRENT_VERSION" ]; then
    gh release create "$TAG" dist/* --repo "$GITHUB_REPOSITORY" --title "Jacket ${TAG#v}" --notes-file notes.md --latest --verify-tag
  else
    gh release create "$TAG" dist/* --repo "$GITHUB_REPOSITORY" --title "Jacket ${TAG#v}" --notes-file notes.md --latest=false --verify-tag
  fi
else
  cat release-error.txt >&2
  exit 1
fi
