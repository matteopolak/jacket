#!/bin/sh
# The pg tool's migrate, status, new and generate commands on a project built from this folder, and
# a program that uses the generated db/schema.jai: it compiles, goes stale when a migration
# changes, and rejects a bad db/json_columns.jai.
#
#     JAI_PG_TEST_PORT=54329 sh run.sh
#
# Needs jaic (JAIC, default `jaic`), psql (PSQL, default `psql`) and a server on 127.0.0.1 that
# lets the `postgres` user in without a password (see ../start_server.sh). It uses and drops the
# database jai_schema_test. Skipped when JAI_PG_TEST_PORT is not set.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
[ -n "${JAI_PG_TEST_PORT:-}" ] || { echo "skipped: set JAI_PG_TEST_PORT"; exit 0; }
jaic=${JAIC:-jaic}
psql=${PSQL:-psql}
work=$(mktemp -d)
project="$work/project"
tool="$here/../../Tool/main.jai"
admin() { "$psql" -h 127.0.0.1 -p "$JAI_PG_TEST_PORT" -U postgres -v ON_ERROR_STOP=1 -q "$@"; }
cleanup() { admin -d postgres -c "drop database if exists jai_schema_test" > /dev/null 2>&1 || true; rm -rf "$work"; }
trap cleanup EXIT
fail() { echo "FAILED: $*"; exit 1; }
pg() { "$jaic" "$tool" -interp -- "$@" --root "$project"; }

mkdir -p "$project/db"
cp -r "$here/migrations" "$project/db/migrations"
cp "$here/json_columns.jai" "$project/db/json_columns.jai"
cp "$here/program.jai" "$project/main.jai"
export DATABASE_URL="postgres://postgres@127.0.0.1:$JAI_PG_TEST_PORT/jai_schema_test"
admin -d postgres -c "drop database if exists jai_schema_test" -c "create database jai_schema_test"

# status and migrate
pg status | grep -q '^0001_teams_and_users  pending$' || fail "status before migrating"
pg migrate | grep -q 'applied 2 migrations' || fail "migrate"
pg status | grep -q '^0002_posts_views_audit  applied$' || fail "status after migrating"
pg migrate | grep -q 'applied 0 migrations' || fail "second migrate"

# new numbers after the newest migration
pg new "Add Things" | grep -q '0003_add_things.sql' || fail "new"
[ -f "$project/db/migrations/0003_add_things.sql" ] || fail "new did not write the file"
rm "$project/db/migrations/0003_add_things.sql"

# generate, then build a program on the result
pg generate | grep -q 'wrote' || fail "generate"
cmp -s "$project/db/schema.jai" "$here/expected_schema.txt" || fail "the generated schema differs from expected_schema.txt"
"$jaic" "$project/main.jai" -exe "$work/program" || fail "build"
[ "$("$work/program")" = ok ] || fail "program"
[ -z "$(admin -d postgres -At -c "select datname from pg_database where datname like 'jacket\\_build\\_%'")" ] || fail "a half-built schema database was left behind"

# a changed migration makes the file stale, with no server needed
echo "create table extra (id int8);" > "$project/db/migrations/0003_extra.sql"
if DATABASE_URL='' "$jaic" "$project/main.jai" -exe "$work/stale" > "$work/out" 2>&1; then fail "a stale schema compiled"; fi
grep -q "db/schema.jai is stale: run the pg tool's generate" "$work/out" || fail "the stale error does not say what to do"
rm "$project/db/migrations/0003_extra.sql"

# a mapping for a column that is not json is an error
printf 'Json_Columns :: Pg.Json_Column.[ .{ "users.email", Settings } ];\n' > "$project/db/json_columns.jai"
if "$jaic" "$project/main.jai" -exe "$work/bad" > "$work/out" 2>&1; then fail "a bad json mapping compiled"; fi
grep -q 'maps "users.email"' "$work/out" || fail "the json mapping error does not name the column"

echo "all schema tests passed"
