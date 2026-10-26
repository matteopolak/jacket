#!/usr/bin/env bash
# Computes the release metadata and writes it to GITHUB_OUTPUT. Reads TAG, EDITOR_INPUT,
# PUBLISH_INTEGRATIONS and EVENT; run from the checkout (Cargo.toml holds the version).
set -e
version="$(python3 -c 'import tomllib; print(tomllib.load(open("Cargo.toml", "rb"))["workspace"]["package"]["version"])')"
is_current=false
dry_run=false
publish_stores=false
publish_editor_stores=false
editor_pinned=false
editor_valid=false
if [ -z "$TAG" ]; then dry_run=true; elif [ "$TAG" = "v$version" ]; then is_current=true; fi
if [ "$EVENT" = push ] || [ "$PUBLISH_INTEGRATIONS" = true ]; then publish_stores=true; fi
editor_version="$EDITOR_INPUT"
if [ -n "$editor_version" ]; then editor_pinned=true; else editor_version="${TAG#v}"; fi
if [ -z "$editor_version" ]; then editor_version="$version"; fi
if [[ "$editor_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then editor_valid=true; fi
if [ "$is_current" = true ] && { [ "$EVENT" = push ] || { [ "$PUBLISH_INTEGRATIONS" = true ] && [ "$editor_pinned" = true ]; }; }; then
  publish_editor_stores=true
fi
{
  echo "version=$version"
  echo "tag=$TAG"
  echo "is_current_release=$is_current"
  echo "dry_run=$dry_run"
  echo "publish_stores=$publish_stores"
  echo "publish_editor_stores=$publish_editor_stores"
  echo "editor_version=$editor_version"
  echo "editor_valid=$editor_valid"
  echo "editor_pinned=$editor_pinned"
} >> "$GITHUB_OUTPUT"
