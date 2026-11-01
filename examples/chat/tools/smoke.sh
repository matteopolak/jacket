#!/bin/sh
# All network endpoints are loopback; SMTP never leaves this machine.
set -eu
app=$(CDPATH="" cd -- "$(dirname -- "$0")/.." && pwd)
if ! command -v python3 >/dev/null 2>&1; then
    echo 'chat smoke: skipped (python3 is absent)'
    exit 0
fi
build=${T:-${TMPDIR:-/tmp}}
work=$(mktemp -d "$build/chat-smoke.XXXXXX")
if [ "${CHAT_KEEP_WORK:-0}" = 1 ]; then
    echo "chat smoke work: $work"
else
    trap 'rm -rf "$work"' EXIT INT TERM
fi
"${JAIC:-jaic}" "$app/main.jai" -exe "$work/chat"
python3 "$app/tools/smoke.py" "$work"
