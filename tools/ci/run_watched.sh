#!/usr/bin/env bash
# Runs a command under a time limit, and on Linux stops the largest process when memory runs low,
# naming what it stopped so the job log says what went wrong instead of the runner freezing.
# Elsewhere it just runs the command.
#
# Usage: tools/ci/run_watched.sh <minutes> -- <command> [args...]
set -u

minutes=$1
shift 2

if [ "$(uname -s)" != Linux ]; then
    exec "$@"
fi

# Stop the largest process when less than this much memory is left (KiB).
low_kib=$((1536 * 1024))

setsid "$@" &
pid=$!
deadline=$((SECONDS + minutes * 60))

while kill -0 "$pid" 2>/dev/null; do
    sleep 5
    available=$(awk '/^MemAvailable:/ { print $2 }' /proc/meminfo)
    if [ "$available" -lt "$low_kib" ]; then
        echo "::error::Only $((available / 1024)) MiB of memory left; the largest processes:"
        ps -eo pid=,rss=,etimes=,args= --sort=-rss | head -5
        read -r biggest _ < <(ps -eo pid= --sort=-rss | head -1)
        echo "Stopping process $biggest."
        kill -9 "$biggest" 2>/dev/null
    fi
    if [ "$SECONDS" -ge "$deadline" ]; then
        echo "::error::Still running after $minutes minutes:"
        ps -eo pid,ppid,etimes,rss,args --forest
        kill -9 -- "-$pid" 2>/dev/null
        wait "$pid"
        exit 124
    fi
done

wait "$pid"
