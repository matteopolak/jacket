#!/usr/bin/env bash
# Runs a command under a time limit and, on Linux, a memory limit, so a job that runs away is
# stopped with a message in the log instead of freezing the runner (a frozen runner uploads no
# log at all). Elsewhere it just runs the command.
#
# The memory limit is a cgroup: when the command's processes together use more than all but
# HEADROOM_MIB of the machine's memory, the kernel kills the largest of them. The test that owned
# it then fails by name, and the rest go on.
#
# Usage: tools/ci/run_watched.sh <minutes> -- <command> [args...]
set -u

minutes=$1
shift 2

if [ "$(uname -s)" != Linux ]; then
    exec "$@"
fi

headroom_mib=${HEADROOM_MIB:-2048}
group=/sys/fs/cgroup/run-watched-$$

limited=false
total_kib=$(awk '/^MemTotal:/ { print $2 }' /proc/meminfo)
limit_bytes=$(((total_kib / 1024 - headroom_mib) * 1024 * 1024))
if grep -qw memory /sys/fs/cgroup/cgroup.subtree_control 2> /dev/null \
    && sudo -n mkdir "$group" 2> /dev/null; then
    echo "$limit_bytes" | sudo -n tee "$group/memory.max" > /dev/null
    echo 0 | sudo -n tee "$group/memory.swap.max" > /dev/null 2>&1
    echo $$ | sudo -n tee "$group/cgroup.procs" > /dev/null && limited=true
fi
if ! $limited; then
    echo "::warning::No memory limit: cannot create a cgroup here."
fi

report_memory() {
    $limited || return 0
    local kills peak
    kills=$(awk '$1 == "oom_kill" { print $2 }' "$group/memory.events")
    peak=$(cat "$group/memory.peak" 2> /dev/null || echo 0)
    echo "Peak memory: $((peak / 1024 / 1024)) MiB of $((limit_bytes / 1024 / 1024)) MiB allowed."
    if [ "${kills:-0}" -gt 0 ]; then
        echo "::error::The memory limit stopped $kills process(es); the tests they belonged to failed above."
    fi
}

setsid "$@" &
pid=$!
deadline=$((SECONDS + minutes * 60))

while kill -0 "$pid" 2> /dev/null; do
    sleep 5
    if [ "$SECONDS" -ge "$deadline" ]; then
        echo "::error::Still running after $minutes minutes:"
        ps -eo pid,ppid,etimes,rss,args --forest
        kill -9 -- "-$pid" 2> /dev/null
        wait "$pid"
        report_memory
        exit 124
    fi
done

wait "$pid"
status=$?
report_memory
exit "$status"
