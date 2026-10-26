#!/usr/bin/env bash
# Sets PLATFORM (GITHUB_ENV) to this runner's release archive platform and `has` (GITHUB_OUTPUT) to whether
# the latest release carries jacket-<platform>.tar.gz. Needs GH_TOKEN.
set -e
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) plat=macos-arm64 ;;
  Darwin-x86_64) plat=macos-x64 ;;
  Linux-x86_64) plat=linux-x64 ;;
  Linux-aarch64) plat=linux-arm64 ;;
esac
echo "PLATFORM=$plat" >> "$GITHUB_ENV"
assets="$(gh release view --repo "$GITHUB_REPOSITORY" --json assets -q '.assets[].name')"
if echo "$assets" | grep -Eqx "jacket-$plat\.tar\.gz"; then has=true; else has=false; fi
echo "$plat in the latest release: $has"
echo "has=$has" >> "$GITHUB_OUTPUT"
