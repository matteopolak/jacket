#!/usr/bin/env bash
# Update the package index and install the given Ubuntu packages without recommendations, with the
# retries and timeouts of apt_lib.sh. Usage: apt_install.sh PACKAGE...
set -euo pipefail
if [[ $# == 0 ]]; then
  echo "usage: $0 PACKAGE..." >&2
  exit 2
fi
# shellcheck source=tools/ci/apt_lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/apt_lib.sh"
apt_get update
apt_get install --yes --no-install-recommends "$@"
