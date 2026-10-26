#!/usr/bin/env bash
# Pushes the rendered formula to matteopolak/homebrew-tap. Needs TAG, VERSION and TAP_TOKEN;
# the formula is at RUNNER_TEMP/packages/homebrew/jacket.rb.
set -e
if [ -z "$TAG" ]; then echo "No tag (dry run); not pushing the formula"; exit 0; fi
if [ -z "$TAP_TOKEN" ]; then echo "HOMEBREW_TAP_TOKEN is not set; skipping the Homebrew tap"; exit 0; fi
git clone --depth 1 "https://x-access-token:${TAP_TOKEN}@github.com/matteopolak/homebrew-tap.git" "$RUNNER_TEMP/tap"
cd "$RUNNER_TEMP/tap"
mkdir -p Formula
cp "$RUNNER_TEMP/packages/homebrew/jacket.rb" Formula/jacket.rb
git add Formula/jacket.rb
if git diff --cached --quiet; then echo "Formula/jacket.rb is already at $VERSION"; exit 0; fi
git -c user.name='github-actions[bot]' -c user.email='41898282+github-actions[bot]@users.noreply.github.com' \
  commit -q -m "jacket $VERSION"
git push origin HEAD
echo "Pushed jacket $VERSION to matteopolak/homebrew-tap"
