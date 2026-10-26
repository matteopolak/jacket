#!/usr/bin/env bash
# Publishes the packaged VSIX to Open VSX, creating the namespace first. Needs OVSX_PAT; run from
# editors/vscode with the VSIX in ../../vsix.
set -e
if [ -z "$OVSX_PAT" ]; then echo "OVSX_PAT is not set; skipping Open VSX"; exit 0; fi
vsix="$(ls ../../vsix/jacket-vscode-*.vsix)"
# The namespace (package.json's publisher) must exist before the first publish. Creating
# it again fails with "Namespace already exists", which is the state we want.
if ! out="$(pnpm exec ovsx create-namespace matteopolak -p "$OVSX_PAT" 2>&1)"; then
  echo "$out"
  echo "$out" | grep -qi "already exists" || exit 1
else
  echo "$out"
fi
pnpm exec ovsx publish "$vsix" -p "$OVSX_PAT" --skip-duplicate
