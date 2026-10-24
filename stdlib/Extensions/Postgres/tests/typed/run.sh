#!/bin/sh
# Builds and runs the typed-query tests against a PostgreSQL server (see ../start_server.sh).
#
#     JAI_PG_TEST_PORT=54329 JAI_PG_TEST_SOCKET_DIR=/tmp/pgsock sh run.sh
#
# Needs jaic (JAIC, default `jaic`), psql (PSQL, default `psql`), and a server that lets the
# `postgres` user in without a password. It uses and rewrites the database jai_typed_test.
# Skipped when JAI_PG_TEST_PORT is not set.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
[ -n "${JAI_PG_TEST_PORT:-}" ] || { echo "skipped: set JAI_PG_TEST_PORT"; exit 0; }
jaic=${JAIC:-jaic}
psql=${PSQL:-psql}
socket=${JAI_PG_TEST_SOCKET_DIR:-/tmp}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

export DATABASE_URL="postgres://postgres@localhost/jai_typed_test?host=$socket&port=$JAI_PG_TEST_PORT"
export JACKET_PG_ROOT="$work/project"
mkdir -p "$JACKET_PG_ROOT"

admin() { "$psql" -h "$socket" -p "$JAI_PG_TEST_PORT" -U postgres -v ON_ERROR_STOP=1 -q "$@"; }
admin -d postgres -c "drop database if exists jai_typed_test" -c "create database jai_typed_test" > /dev/null
admin -d jai_typed_test -f "$here/schema.sql" > /dev/null

fail() { echo "FAILED: $*"; exit 1; }

# 1. Every macro, describing through the service and filling the cache.
"$jaic" "$here/queries.jai" -exe "$work/queries" || fail "build"
[ "$("$work/queries")" = ok ] || fail "queries"
count=$(find "$JACKET_PG_ROOT/db/queries" -type f | wc -l)
[ "$count" -gt 10 ] || fail "cache entries were not written ($count)"

# 2. Offline: the committed cache alone is enough.
rm -f "$work/queries"
JACKET_PG_OFFLINE=1 DATABASE_URL='' "$jaic" "$here/queries.jai" -exe "$work/queries" || fail "offline build"
[ "$("$work/queries")" = ok ] || fail "offline queries"

# 3. Offline with an empty cache is an error naming the query.
rm -rf "$JACKET_PG_ROOT/db"
if JACKET_PG_OFFLINE=1 DATABASE_URL='' "$jaic" "$here/queries.jai" -exe "$work/none" > "$work/out" 2>&1; then fail "offline build without a cache succeeded"; fi
grep -q "JACKET_PG_OFFLINE" "$work/out" || fail "offline error does not name the setting"

# 4. A changed table re-describes only the queries that use it.
"$jaic" "$here/queries.jai" -exe "$work/queries" > /dev/null || fail "rebuild"
before=$(cat "$JACKET_PG_ROOT"/db/queries/*.json | md5 2>/dev/null || cat "$JACKET_PG_ROOT"/db/queries/*.json | md5sum)
admin -d jai_typed_test -c "alter table posts add column pinned boolean not null default false" > /dev/null
"$jaic" "$here/queries.jai" -exe "$work/queries" > /dev/null || fail "build after alter"
[ "$("$work/queries")" = ok ] || fail "queries after alter"
after=$(cat "$JACKET_PG_ROOT"/db/queries/*.json | md5 2>/dev/null || cat "$JACKET_PG_ROOT"/db/queries/*.json | md5sum)
[ "$before" != "$after" ] || fail "a changed table did not change the cache"
unchanged=$(grep -l '"r:public.posts"' "$JACKET_PG_ROOT"/db/queries/*.json | wc -l)
total=$(find "$JACKET_PG_ROOT/db/queries" -name "*.json" | wc -l)
[ "$unchanged" -lt "$total" ] || fail "every entry depends on posts?"

# 5. The migrations' hash stands in for the database: record it, then work offline against it.
mkdir -p "$JACKET_PG_ROOT/db/migrations"
echo "create table t (a int);" > "$JACKET_PG_ROOT/db/migrations/0001_init.sql"
"$jaic" "$here/queries.jai" -exe "$work/queries" > /dev/null || fail "build with migrations"
grep -q '"schema_hash": "[0-9a-f]\{64\}"' "$JACKET_PG_ROOT"/db/queries/*.json || fail "the schema hash was not recorded"
rm -f "$work/queries"
JACKET_PG_OFFLINE=1 DATABASE_URL='' "$jaic" "$here/queries.jai" -exe "$work/queries" || fail "offline build against the schema hash"
[ "$("$work/queries")" = ok ] || fail "offline queries against the schema hash"
echo "create table t2 (a int);" > "$JACKET_PG_ROOT/db/migrations/0002_more.sql"
if JACKET_PG_OFFLINE=1 DATABASE_URL='' "$jaic" "$here/queries.jai" -exe "$work/stale" > "$work/out" 2>&1; then fail "a changed migration did not stale the cache"; fi
rm -rf "$JACKET_PG_ROOT/db/migrations"

# 6. The pg tool: prepare, then check is clean; a stray entry and a missing one are found.
tool="$here/../../Tool/main.jai"
"$jaic" "$tool" -interp -- prepare --root "$JACKET_PG_ROOT" > /dev/null || fail "tool prepare"
"$jaic" "$tool" -interp -- check --root "$JACKET_PG_ROOT" --build "$jaic $here/queries.jai -exe $work/queries" || fail "tool check"
echo '{}' > "$JACKET_PG_ROOT/db/queries/1111111111111111.json"
if "$jaic" "$tool" -interp -- check --root "$JACKET_PG_ROOT" > "$work/out" 2>&1; then fail "tool check missed an unused entry"; fi
grep -q "unused entry 1111111111111111" "$work/out" || fail "tool check did not name the unused entry"
rm -f "$JACKET_PG_ROOT/db/queries/1111111111111111.json"

# 7. Bad programs: the error is reported at the right place.
python3 "$here/bad.py" "$jaic" "$work" || fail "bad programs"

echo "all typed query tests passed"
