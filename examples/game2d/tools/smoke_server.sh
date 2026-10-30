#!/bin/sh
# Local smoke test of the backend: builds the server and the load test, starts a throwaway
# PostgreSQL, serves the API over HTTP and then over HTTPS, checks a few refusals with curl and
# runs a short concurrent load test against each. Not part of CI.
#
#     examples/game2d/tools/smoke_server.sh            # needs jaic (or $JAIC), curl, openssl, python3, PostgreSQL
#     ROUNDS=200 WORKERS=32 examples/game2d/tools/smoke_server.sh
#
# WITH_GAME=1 also runs the game's smoke test (smoke_game.sh) against the server over HTTP.
#
# Environment: JAIC (compiler, default jaic), PG_BIN (PostgreSQL's bin folder), WORKERS and ROUNDS
# (load per scheduler thread), THREADS (scheduler threads of the load test).

set -eu

here=$(cd "$(dirname "$0")" && pwd)
app=$(dirname "$here")
jaic=${JAIC:-jaic}
workers=${WORKERS:-8}
rounds=${ROUNDS:-25}
threads=${THREADS:-2}

work=$(mktemp -d "${TMPDIR:-/tmp}/game2d-smoke.XXXXXX")
server_pid=""
pg_dir=""

cleanup() {
    if [ -n "$server_pid" ]; then kill "$server_pid" 2>/dev/null || true; wait "$server_pid" 2>/dev/null || true; fi
    if [ -n "$pg_dir" ]; then "$here/local_postgres.sh" stop "$pg_dir" || true; fi
    rm -rf "$work"
}
trap cleanup EXIT INT TERM

fail() {
    echo "smoke_server: FAILED: $*" >&2
    if [ -f "$work/server.log" ]; then echo "--- server log ---" >&2; tail -20 "$work/server.log" >&2; fi
    exit 1
}

free_port() {
    python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])'
}

echo "== build"
"$jaic" "$app/server/main.jai" -exe "$work/server" -quiet || fail "the server does not build"
"$jaic" "$app/loadtest/main.jai" -exe "$work/loadtest" -quiet || fail "the load test does not build"

echo "== postgres"
eval "$("$here/local_postgres.sh" start)"
pg_dir=${PG_DIR:?}

openssl req -x509 -newkey rsa:2048 -nodes -keyout "$work/key.pem" -out "$work/cert.pem" -days 2 \
    -subj /CN=localhost -addext "subjectAltName=DNS:localhost,IP:127.0.0.1" >/dev/null 2>&1 || fail "openssl could not make a certificate"

# start_server [server arguments]: starts it on a free port and waits until /api/health answers.
start_server() {
    port=$(free_port)
    "$work/server" --listen "127.0.0.1:$port" "$@" >"$work/server.log" 2>&1 &
    server_pid=$!
    scheme=http
    case " $* " in *" --tls-cert "*) scheme=https ;; esac
    base="$scheme://localhost:$port"
    tries=0
    until curl -sf --cacert "$work/cert.pem" "$base/api/health" >/dev/null 2>&1; do
        tries=$((tries + 1))
        if [ $tries -gt 100 ]; then fail "the server did not come up ($base)"; fi
        kill -0 "$server_pid" 2>/dev/null || fail "the server stopped at start"
        sleep 0.1
    done
}

stop_server() {
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
    server_pid=""
}

expect_status() {
    want=$1
    shift
    got=$(curl -s -o "$work/body" -w '%{http_code}' --cacert "$work/cert.pem" "$@")
    if [ "$got" != "$want" ]; then fail "expected HTTP $want, got $got for: $* ($(cat "$work/body"))"; fi
}

echo "== http"
start_server --database "$DATABASE_URL" --threads 2
expect_status 200 "$base/api/health"
expect_status 201 -X POST "$base/api/accounts" -H 'content-type: application/json' -d '{"name":"smoke"}'
grep -q '"token"' "$work/body" || fail "the new account has no token"
expect_status 409 -X POST "$base/api/accounts" -H 'content-type: application/json' -d '{"name":"smoke"}'
expect_status 422 -X POST "$base/api/accounts" -H 'content-type: application/json' -d '{"name":"no spaces"}'
expect_status 400 -X POST "$base/api/accounts" -H 'content-type: application/json' -d '{not json'
expect_status 401 -X POST "$base/api/matches" -H 'content-type: application/json' -d '{"seed":1,"score":10,"kills":1,"duration_ms":5000,"checksum":"x"}'
expect_status 401 -X POST "$base/api/matches" -H 'content-type: application/json' -H 'Authorization: Bearer nope' -d '{"seed":1,"score":10,"kills":1,"duration_ms":5000,"checksum":"x"}'
expect_status 404 "$base/api/accounts/999999"
expect_status 404 "$base/api/nothing"
expect_status 200 "$base/api/leaderboard?limit=5"
expect_status 422 "$base/api/leaderboard?limit=0"
"$work/loadtest" --url "$base" --threads "$threads" --workers "$workers" --rounds "$rounds" || fail "the load test reported failures over http"
expect_status 200 "$base/api/leaderboard?limit=3"
grep -q '"rank":1' "$work/body" || fail "the leaderboard has no first place after the load test"
if [ "${WITH_GAME:-0}" = 1 ]; then
    echo "== game against the server"
    SERVER_URL=$base "$here/smoke_game.sh" || fail "the game smoke test failed"
fi
stop_server

echo "== https"
start_server --database "$DATABASE_URL" --threads 2 --tls-cert "$work/cert.pem" --tls-key "$work/key.pem"
expect_status 200 "$base/api/health"
"$work/loadtest" --url "$base" --roots "$work/cert.pem" --threads "$threads" --workers "$workers" --rounds "$rounds" || fail "the load test reported failures over https"
stop_server

echo "smoke_server: ok"
