#!/bin/sh
# A throwaway PostgreSQL for the dogfood smoke tests: a cluster in a temporary folder, listening on
# 127.0.0.1 only, trusting every local connection. Not for anything but a developer's machine.
#
#     eval "$(examples/game2d/tools/local_postgres.sh start)"   # sets DATABASE_URL and PG_DIR
#     examples/game2d/tools/local_postgres.sh stop "$PG_DIR"
#
# It needs PostgreSQL's server binaries (initdb, pg_ctl): from PATH, $PG_BIN, or Homebrew's
# postgresql@17 / postgresql@16 / postgresql@15.

set -eu

# macOS starts a multithreaded postmaster (and refuses to run) unless the locale is valid.
export LC_ALL=en_US.UTF-8

find_bin() {
    if [ -n "${PG_BIN:-}" ] && [ -x "$PG_BIN/pg_ctl" ]; then echo "$PG_BIN"; return; fi
    if command -v pg_ctl >/dev/null 2>&1; then dirname "$(command -v pg_ctl)"; return; fi
    for v in 17 16 15 18; do
        for prefix in /opt/homebrew/opt /usr/local/opt; do
            if [ -x "$prefix/postgresql@$v/bin/pg_ctl" ]; then echo "$prefix/postgresql@$v/bin"; return; fi
        done
    done
    for d in /usr/lib/postgresql/*/bin; do
        if [ -x "$d/pg_ctl" ]; then echo "$d"; fi
    done | tail -1
}

free_port() {
    python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])'
}

case "${1:-}" in
start)
    bin=$(find_bin)
    if [ -z "$bin" ]; then echo "echo 'no PostgreSQL server binaries (initdb, pg_ctl) found; set PG_BIN' >&2; false"; exit 1; fi
    dir=$(mktemp -d "${TMPDIR:-/tmp}/game2d-pg.XXXXXX")
    port=$(free_port)
    "$bin/initdb" -D "$dir/data" -U postgres -A trust --no-sync -E UTF8 --locale=C >/dev/null
    "$bin/pg_ctl" -D "$dir/data" -l "$dir/server.log" -w \
        -o "-p $port -k $dir -c listen_addresses=127.0.0.1 -c fsync=off -c max_connections=100" start >/dev/null
    echo "PG_DIR='$dir'"
    echo "PG_BIN='$bin'"
    echo "DATABASE_URL='postgres://postgres@127.0.0.1:$port/postgres?sslmode=disable'"
    echo "export PG_DIR PG_BIN DATABASE_URL"
    ;;
stop)
    dir=${2:?usage: local_postgres.sh stop DIR}
    bin=$(find_bin)
    "$bin/pg_ctl" -D "$dir/data" -m immediate -w stop >/dev/null 2>&1 || true
    rm -rf "$dir"
    ;;
*)
    echo "usage: local_postgres.sh start | stop DIR" >&2
    exit 2
    ;;
esac
