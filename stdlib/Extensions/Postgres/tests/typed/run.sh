#!/bin/sh
# Builds and runs the typed-query tests against a PostgreSQL server (see ../start_server.sh).
#
#     JAI_PG_TEST_PORT=54329 JAI_PG_TEST_SOCKET_DIR=/tmp/pgsock sh run.sh
#
# Needs jaic (JAIC, default `jaic`), psql (PSQL, default `psql`), and a server that lets the
# `postgres` user in without a password. It uses and rewrites the database
# jai_typed_test (JAI_PG_TEST_DATABASE picks another).
# Skipped when JAI_PG_TEST_PORT is not set.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
[ -n "${JAI_PG_TEST_PORT:-}" ] || { echo "skipped: set JAI_PG_TEST_PORT"; exit 0; }
jaic=${JAIC:-jaic}
psql=${PSQL:-psql}
socket=${JAI_PG_TEST_SOCKET_DIR:-/tmp}
database=${JAI_PG_TEST_DATABASE:-jai_typed_test}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

export DATABASE_URL="postgres://postgres@localhost/$database?host=$socket&port=$JAI_PG_TEST_PORT"
export JACKET_PG_ROOT="$work/project"
mkdir -p "$JACKET_PG_ROOT"

admin() { "$psql" -h "$socket" -p "$JAI_PG_TEST_PORT" -U postgres -v ON_ERROR_STOP=1 -q "$@"; }
admin -d postgres -c "drop database if exists $database" -c "create database $database" > /dev/null
admin -d "$database" -f "$here/schema.sql" > /dev/null

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

# 2b. An entry written by another format is a miss: an error offline, described again online.
grep -q '"format": 1' "$JACKET_PG_ROOT"/db/queries/*.json || fail "entries do not record their format"
for entry in "$JACKET_PG_ROOT"/db/queries/*.json; do
    sed 's/"format": 1/"format": 0/' "$entry" > "$entry.tmp" && mv "$entry.tmp" "$entry"
done
if JACKET_PG_OFFLINE=1 DATABASE_URL='' "$jaic" "$here/queries.jai" -exe "$work/old_format" > "$work/out" 2>&1; then fail "entries of another format were used"; fi
grep -q "JACKET_PG_OFFLINE" "$work/out" || fail "the old format error does not name the setting"
"$jaic" "$here/queries.jai" -exe "$work/queries" > /dev/null || fail "build over entries of another format"
[ "$("$work/queries")" = ok ] || fail "queries over entries of another format"
[ "$(grep -L '"format": 1' "$JACKET_PG_ROOT"/db/queries/*.json | wc -l)" -eq 0 ] || fail "entries of another format were not rewritten"

# 3. Offline with an empty cache is an error naming the query.
rm -rf "$JACKET_PG_ROOT/db"
if JACKET_PG_OFFLINE=1 DATABASE_URL='' "$jaic" "$here/queries.jai" -exe "$work/none" > "$work/out" 2>&1; then fail "offline build without a cache succeeded"; fi
grep -q "JACKET_PG_OFFLINE" "$work/out" || fail "offline error does not name the setting"

# 4. A changed table re-describes only the queries that use it.
"$jaic" "$here/queries.jai" -exe "$work/queries" > /dev/null || fail "rebuild"
before=$(cat "$JACKET_PG_ROOT"/db/queries/*.json | md5 2>/dev/null || cat "$JACKET_PG_ROOT"/db/queries/*.json | md5sum)
admin -d "$database" -c "alter table posts add column pinned boolean not null default false" > /dev/null
"$jaic" "$here/queries.jai" -exe "$work/queries" > /dev/null || fail "build after alter"
[ "$("$work/queries")" = ok ] || fail "queries after alter"
after=$(cat "$JACKET_PG_ROOT"/db/queries/*.json | md5 2>/dev/null || cat "$JACKET_PG_ROOT"/db/queries/*.json | md5sum)
[ "$before" != "$after" ] || fail "a changed table did not change the cache"
unchanged=$(grep -l '"r:public.posts"' "$JACKET_PG_ROOT"/db/queries/*.json | wc -l)
total=$(find "$JACKET_PG_ROOT/db/queries" -name "*.json" | wc -l)
[ "$unchanged" -lt "$total" ] || fail "every entry depends on posts?"

# 5. With migrations, queries are described in a schema database built from them, and the
# migrations' hash stands in for the database offline.
schema_databases() { admin -d postgres -At -c "select datname from pg_database where datname ~ '^jacket_([0-9a-f]{12}|build_.*)$' order by oid"; }
drop_schema_databases() { for db in $(schema_databases); do admin -d postgres -c "drop database \"$db\"" > /dev/null; done; }
drop_schema_databases
mkdir -p "$JACKET_PG_ROOT/db/migrations"
grep -v "schema public" "$here/schema.sql" > "$JACKET_PG_ROOT/db/migrations/0001_init.sql"
"$jaic" "$here/queries.jai" -exe "$work/queries" > /dev/null || fail "build with migrations"
[ "$(schema_databases | wc -l)" -eq 1 ] || fail "no schema database was built: $(schema_databases)"
grep -q '"schema_hash": "[0-9a-f]\{64\}"' "$JACKET_PG_ROOT"/db/queries/*.json || fail "the schema hash was not recorded"
rm -f "$work/queries"
JACKET_PG_OFFLINE=1 DATABASE_URL='' "$jaic" "$here/queries.jai" -exe "$work/queries" || fail "offline build against the schema hash"
[ "$("$work/queries")" = ok ] || fail "offline queries against the schema hash"
# A new migration is first applied on top of the existing database, in a rolled-back transaction.
echo "alter table posts add column pinned boolean not null default false;" > "$JACKET_PG_ROOT/db/migrations/0002_pinned.sql"
if JACKET_PG_OFFLINE=1 DATABASE_URL='' "$jaic" "$here/queries.jai" -exe "$work/stale" > "$work/out" 2>&1; then fail "a changed migration did not stale the cache"; fi
cat > "$work/pinned.jai" <<'EOF'
#import "Basic";
Pg :: #import "Extensions/Postgres";
pinned :: (db: *Pg.Connection) -> bool {
    row, error := Pg.query_one(db, "select pinned from posts where id = 1");
    return row.pinned;
}
main :: () { print("%", cast(*void) pinned); }
EOF
mkdir -p "$work/pinned_root/db" && cp -R "$JACKET_PG_ROOT/db/migrations" "$work/pinned_root/db/"
JACKET_PG_ROOT="$work/pinned_root" JACKET_PG_BUILD_DELAY_MS=600000 "$jaic" "$work/pinned.jai" -exe "$work/pinned" > /dev/null || fail "a query using the new migration's column"
[ "$(schema_databases | wc -l)" -eq 1 ] || fail "the new migration built a database early: $(schema_databases)"
JACKET_PG_BUILD_DELAY_MS=600000 "$jaic" "$here/queries.jai" -exe "$work/queries" > /dev/null || fail "build with a new migration"
[ "$("$work/queries")" = ok ] || fail "queries with a new migration"
# One that cannot run in a transaction makes a database for the whole set right away; only the
# newest three are kept.
printf -- '-- jacket: no-transaction\ncreate index concurrently posts_title on posts (title);\n' > "$JACKET_PG_ROOT/db/migrations/0003_index.sql"
"$jaic" "$here/queries.jai" -exe "$work/queries" > /dev/null || fail "build with a no-transaction migration"
[ "$(schema_databases | wc -l)" -eq 2 ] || fail "the no-transaction migration did not build a database: $(schema_databases)"
for n in 4 5 6; do
    printf -- '-- jacket: no-transaction\ncreate table extra_%s (a int);\n' "$n" > "$JACKET_PG_ROOT/db/migrations/000${n}_extra.sql"
    "$jaic" "$here/queries.jai" -exe "$work/queries" > /dev/null || fail "build with migration $n"
done
[ "$(schema_databases | wc -l)" -eq 3 ] || fail "old schema databases were not dropped: $(schema_databases)"
drop_schema_databases
rm -rf "$JACKET_PG_ROOT/db/migrations"

# 5b. A migration that fails: the error is reported at its line in the migration file, and the
# queries are still checked against the schema before it (never cached as the project's).
drop_schema_databases
mkdir -p "$JACKET_PG_ROOT/db/migrations"
grep -v "schema public" "$here/schema.sql" > "$JACKET_PG_ROOT/db/migrations/0001_init.sql"
printf -- '-- adds a flag\nalter table posts add column flagged boolean;\nalter table nope_table add column x int;\n' > "$JACKET_PG_ROOT/db/migrations/0002_broken.sql"
mv "$JACKET_PG_ROOT/db/queries" "$work/saved_queries"
broken_build() {
    if "$jaic" "$here/queries.jai" -exe "$work/broken" > "$work/out" 2>&1; then fail "a failing migration did not stop the build"; fi
    grep -q '0002_broken.sql:3:1: error: migration 0002_broken.sql failed at line 3: relation "nope_table" does not exist' "$work/out" || fail "the failed migration is not reported at its line"
    grep -q 'checked against the schema before the failed migration 0002_broken.sql' "$work/out" || fail "no note about the schema before the failed migration"
    [ "$(grep -c ': error:' "$work/out")" -eq 1 ] || fail "a failed migration also failed the queries"
    [ "$(find "$JACKET_PG_ROOT/db/queries" -name '*.json' 2>/dev/null | wc -l)" -eq 0 ] || fail "descriptions of the schema before a failed migration were cached"
}
# Nothing is built yet: the database for the migrations before it is kept.
broken_build
[ "$(schema_databases | wc -l)" -eq 1 ] || fail "the schema before the failed migration was not kept: $(schema_databases)"
# That database exists now: the failed migration runs in the describe's transaction instead.
JACKET_PG_BUILD_DELAY_MS=600000 broken_build
[ "$(schema_databases | wc -l)" -eq 1 ] || fail "a failed migration built a database"
# One that cannot run in a transaction leaves no schema to trust: the queries cannot be checked.
rm "$JACKET_PG_ROOT/db/migrations/0002_broken.sql"
printf -- '-- jacket: no-transaction\nselect 1;\nselect * from missing_t;\n' > "$JACKET_PG_ROOT/db/migrations/0002_unsafe.sql"
if "$jaic" "$here/queries.jai" -exe "$work/broken" > "$work/out" 2>&1; then fail "a failing no-transaction migration did not stop the build"; fi
grep -q '0002_unsafe.sql:3:15: error: migration 0002_unsafe.sql failed at line 3: relation "missing_t" does not exist' "$work/out" || fail "the failed no-transaction migration is not reported at its line"
drop_schema_databases
rm -rf "$JACKET_PG_ROOT/db/migrations" "$JACKET_PG_ROOT/db/queries"
mv "$work/saved_queries" "$JACKET_PG_ROOT/db/queries"

# 6. The pg tool: prepare, then check is clean; a stray entry and a missing one are found.
tool="$here/../../Tool/main.jai"
"$jaic" "$tool" -interp -- prepare --root "$JACKET_PG_ROOT" > /dev/null || fail "tool prepare"
"$jaic" "$tool" -interp -- check --root "$JACKET_PG_ROOT" --build "$jaic $here/queries.jai -exe $work/queries" || fail "tool check"
echo '{}' > "$JACKET_PG_ROOT/db/queries/1111111111111111.json"
if "$jaic" "$tool" -interp -- check --root "$JACKET_PG_ROOT" > "$work/out" 2>&1; then fail "tool check missed an unused entry"; fi
grep -q "unused entry 1111111111111111" "$work/out" || fail "tool check did not name the unused entry"
rm -f "$JACKET_PG_ROOT/db/queries/1111111111111111.json"

# 7. Row type identity: `select *` returns the table's struct from db/schema.jai.
cp -R "$here/identity" "$work/identity"
JACKET_PG_ROOT="$work/identity" "$jaic" "$work/identity/main.jai" -exe "$work/identity_test" || fail "build row type identity"
[ "$("$work/identity_test")" = ok ] || fail "row type identity"

# 8. The ORM: select, insert, update and delete through table structs.
cp -R "$here/../orm" "$work/orm"
JACKET_PG_ROOT="$work/orm" "$jaic" "$work/orm/main.jai" -exe "$work/orm_test" || fail "build orm"
[ "$("$work/orm_test")" = ok ] || fail "orm"
# Pg.each fetched only what its loop body reads.
grep -rqF 'select \"id\", \"email\" from \"public\".\"users\"' "$work/orm/db/queries" || fail "orm each did not prune its columns"
grep -rqF 'jsonb_each(\"settings\"::jsonb) as orm_j' "$work/orm/db/queries" || fail "orm each did not prune the settings document"
grep -rqF 'as \"orm__author__email\"' "$work/orm/db/queries" || fail "orm each did not read the author in the same statement"
grep -rqF '"select \"id\" from \"public\".\"users\" order by \"id\""' "$work/orm/db/queries" || fail "orm each read more than the key for a has-many loop"
grep -rqF 'left join lateral (select coalesce(array_agg(orm_m0_rows.' "$work/orm/db/queries" || fail "orm each with strategy = .LATERAL did not read the posts in the same statement"

# 9. Bad programs: the error is reported at the right place.
python3 "$here/bad.py" "$jaic" "$work" || fail "bad programs"

# 9b. Module parameters: other paths for the migrations and the cache, and a minimum server version.
params_root="$work/params_project"
mkdir -p "$params_root/sql/migrations"
grep -v "schema public" "$here/schema.sql" > "$params_root/sql/migrations/0001_init.sql"
cat > "$work/params.jai" <<'EOF'
#import "Basic";
Pg :: #import "Extensions/Postgres"(MIGRATIONS = "sql/migrations", SCHEMA = "sql/schema.jai", QUERY_CACHE = "sql/cache", MIN_SERVER_VERSION = 15);
check :: (db: *Pg.Connection) {
    row, error := Pg.query_one(db, "select id from users where id = 1");
}
main :: () { print("%", cast(*void) check); }
EOF
sed 's/MIN_SERVER_VERSION = 15/MIN_SERVER_VERSION = 99/' "$work/params.jai" > "$work/params_new.jai"
drop_schema_databases
JACKET_PG_ROOT="$params_root" "$jaic" "$work/params.jai" -exe "$work/params" > /dev/null || fail "build with module parameters"
[ "$(find "$params_root/sql/cache" -name '*.json' | wc -l)" -eq 1 ] || fail "the cache did not go to QUERY_CACHE"
[ ! -e "$params_root/db" ] || fail "module parameters left a db folder behind"
[ "$(schema_databases | wc -l)" -eq 1 ] || fail "the migrations folder from MIGRATIONS was not used"
rm -rf "$params_root/sql/cache"
if JACKET_PG_ROOT="$params_root" "$jaic" "$work/params_new.jai" -exe "$work/params_new" > "$work/out" 2>&1; then fail "a server older than MIN_SERVER_VERSION was accepted"; fi
grep -q "needs at least 99 (MIN_SERVER_VERSION)" "$work/out" || fail "the old server error does not name MIN_SERVER_VERSION"
drop_schema_databases

# 9c. The URL from the project's .env file when the environment has none; the environment wins.
env_root="$work/dotenv_project"
mkdir -p "$env_root"
printf '# local database\nexport DATABASE_URL="%s" # the dev server\n' "$DATABASE_URL" > "$env_root/.env"
env -u DATABASE_URL -u JACKET_DATABASE_URL JACKET_PG_ROOT="$env_root" "$jaic" "$here/queries.jai" -exe "$work/dotenv_queries" > /dev/null || fail "build with the URL in .env"
[ "$("$work/dotenv_queries")" = ok ] || fail "queries with the URL in .env"
[ "$(find "$env_root/db/queries" -type f | wc -l)" -gt 10 ] || fail "the .env URL did not fill the cache"
rm -rf "$env_root/db"
if env DATABASE_URL="postgres://postgres@localhost/none?host=/nonexistent&port=1" JACKET_PG_ROOT="$env_root" "$jaic" "$here/queries.jai" -exe "$work/dotenv_none" > "$work/out" 2>&1; then fail ".env won over the environment"; fi

# 10. Managed mode: no database URL at all; the describe service starts its own server and builds
# the schema database from the migrations. Downloads the pinned PostgreSQL release unless
# JAI_PG_TEST_MANAGED_CACHE names a folder that has it. Gated on JAI_PG_TEST_MANAGED=1.
if [ "${JAI_PG_TEST_MANAGED:-}" = 1 ]; then
    managed_root="$work/managed_project"
    managed_cache=${JAI_PG_TEST_MANAGED_CACHE:-$work/managed_cache}
    mkdir -p "$managed_root/db/migrations"
    managed() { env -u DATABASE_URL -u JACKET_DATABASE_URL JACKET_PG_ROOT="$managed_root" JACKET_PG_MANAGED=1 JACKET_PG_MANAGED_CACHE="$managed_cache" "$@"; }
    stop_managed_server() {
        for data in "$managed_cache"/cluster-*; do
            [ -f "$data/postmaster.pid" ] || continue
            for pg_ctl in "$managed_cache"/*/bin/pg_ctl; do "$pg_ctl" stop -D "$data" -m fast > /dev/null 2>&1 || true; done
        done
    }
    trap 'stop_managed_server; rm -rf "$work"' EXIT

    # Without migrations there is no schema to describe against: the error says so.
    if managed "$jaic" "$here/queries.jai" -exe "$work/managed_none" > "$work/out" 2>&1; then fail "a managed build without migrations succeeded"; fi
    grep -q "no migrations" "$work/out" || fail "the managed error does not mention migrations"

    grep -v "schema public" "$here/schema.sql" > "$managed_root/db/migrations/0001_init.sql"
    managed "$jaic" "$here/queries.jai" -exe "$work/managed_queries" || fail "managed build"
    [ "$("$work/managed_queries")" = ok ] || fail "managed queries"
    managed_count=$(find "$managed_root/db/queries" -type f | wc -l)
    [ "$managed_count" -gt 10 ] || fail "managed cache entries were not written ($managed_count)"
    running=0
    for pidfile in "$managed_cache"/cluster-*/postmaster.pid; do [ -f "$pidfile" ] && running=1; done
    [ "$running" = 1 ] || fail "the managed server is not running"

    # A second build with an empty cache finds the server running and reuses it.
    rm -rf "$managed_root/db/queries" "$work/managed_queries"
    managed "$jaic" "$here/queries.jai" -exe "$work/managed_queries" || fail "managed rebuild"
    [ "$("$work/managed_queries")" = ok ] || fail "managed queries after a rebuild"
    [ "$(find "$managed_root/db/queries" -type f | wc -l)" -eq "$managed_count" ] || fail "managed rebuild wrote a different cache"

    # `[postgres] managed = true` in jacket.toml turns it on without the variable.
    printf '[postgres]\nmanaged = true\n' > "$managed_root/jacket.toml"
    from_toml() { env -u DATABASE_URL -u JACKET_DATABASE_URL -u JACKET_PG_MANAGED JACKET_PG_ROOT="$managed_root" JACKET_PG_MANAGED_CACHE="$managed_cache" "$@"; }
    rm -rf "$managed_root/db/queries"
    from_toml "$jaic" "$here/queries.jai" -exe "$work/managed_queries" || fail "managed build from jacket.toml"
    [ "$(find "$managed_root/db/queries" -type f | wc -l)" -eq "$managed_count" ] || fail "the jacket.toml build wrote a different cache"

    # The pg tool's prepare describes against the managed server too.
    from_toml "$jaic" "$here/../../Tool/main.jai" -interp -- prepare --root "$managed_root" > "$work/out" 2>&1 || { cat "$work/out"; fail "pg prepare with managed mode"; }

    # Two builds at once with the server stopped: the lock lets one start it, the other reuses it.
    stop_managed_server
    rm -rf "$managed_root/db/queries"
    mkdir -p "$work/race_a" "$work/race_b"
    from_toml "$jaic" "$here/queries.jai" -exe "$work/race_a/q" > "$work/race_a/out" 2>&1 &
    race_a=$!
    from_toml "$jaic" "$here/queries.jai" -exe "$work/race_b/q" > "$work/race_b/out" 2>&1 &
    race_b=$!
    wait "$race_a" || { cat "$work/race_a/out"; fail "the first of two racing managed builds"; }
    wait "$race_b" || { cat "$work/race_b/out"; fail "the second of two racing managed builds"; }
    [ "$("$work/race_a/q")" = ok ] && [ "$("$work/race_b/q")" = ok ] || fail "racing managed builds"
    [ "$(find "$managed_root/db/queries" -type f | wc -l)" -eq "$managed_count" ] || fail "racing managed builds wrote a different cache"
    rm -f "$managed_root/jacket.toml"
    stop_managed_server
fi

echo "all typed query tests passed"
