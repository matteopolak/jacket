# Postgres typed queries

## What it is

`Pg.query`, `query_one`, `query_opt`, `execute`, `query_scalar` and `query_as` are macros that check a raw SQL string against a real database while the program compiles. Row types, nullability, argument types and server errors (with source spans) all come from the database.

```jai
rows, error := Pg.query(db, "select id, email from users where id = %", user_id);
for rows print("% %\n", it.id, it.email);
Pg.free_rows(*rows);
```

## How it works

1. **Rewrite.** `sql_scan.jai` turns each `%` into `$n`, skipping quoted strings, comments and dollar quotes (`%%` is a literal `%`; a Postgres `$n` is rejected). The statement id is a 64-bit hash of the rewritten text.
2. **Resolve.** `resolve.jai` (`prepare_query`, memoised per call site) looks for `<QUERY_CACHE>/<id>.json` (`db/queries` by default) under the project root. On a miss it asks the describe service, which uses the URL if there is one and otherwise, with managed mode on (`JACKET_PG_MANAGED=1` or `[postgres] managed = true` in `jacket.toml`), a managed server.
3. **Describe.** `describe_service.jai` is a `jaic -interp` worker (`Service.serve`) that holds connections per URL, backs off on failure (1 s doubling to 30 s) and memoises connect errors. `describe.jai` does Parse + Describe, resolves type names (domains, arrays, enums), reads `attnotnull`, and runs `EXPLAIN (VERBOSE, FORMAT JSON)` on the generic plan to find columns made nullable by outer joins. Aliases can force nullability or a type: `name!` (non-null), `name?` (nullable), `name: Jai_Type`.
4. **Shape.** `shape.jai` builds `Row(shape)`, a struct with one doc-commented field per column, and checks arguments against the parameter types (`s64` does not narrow to `integer`). Errors point at the argument or, for server errors, at the offending byte of the string literal.
5. **Run.** `typed.jai` expands into a call that sends the rewritten statement (binary results when every column is decodable) as a cached named statement: the first run on a connection parses it, later runs only bind. `plan = .CUSTOM` on any of the macros uses the unnamed statement instead. `for rows` decodes one row at a time. See "Statement cache" in [the client reference](../extensions/postgres.md).

`query_one` and `query_scalar` return a `.NOT_FOUND` error when no row matches, and a `.USAGE` error for more than one. `query_as(T, ...)` checks a hand-written struct: columns match fields by name, nullable columns need `Nullable`, and `Json(T)` fields say what a json column holds.

### Pipelines

`queue(p, sql, ..)` and `queue_execute(p, sql, ..)` are `query` and `execute` on a `*Pipeline` (`pipeline_enter`): the statement is checked at compile time as usual, queued unsent, and returns a `Pending(R)`. `take(p, pending)` returns the `Rows(...)` or the changed-row count.

```jai
p := Pg.pipeline_enter(c);
user := Pg.queue(p, "select id, email from users where id = %", id);
seen := Pg.queue_execute(p, "update users set seen_at = now() where id = %", id);
rows, error := Pg.take(p, user); // one round trip for both
count, error = Pg.take(p, seen);
Pg.pipeline_exit(p);
```

- The first `take` adds a sync point if the queued statements have none, so they run as one implicit transaction: an error fails its statement, and the rest of the batch fails with `.ABORTED`.
- Pendings can be taken in any order. Results read on the way are kept on the pipeline (`Pipeline.early`) until taken; `pipeline_exit` refuses while any are left.
- Each pending carries the pipeline's statement serial (`Pipeline.queued`, `Pipeline.read`), so mixing `take` with `pipeline_next_result` on the same batch is not supported.
- Queued statements use the unnamed statement, not the statement cache.

### Row type identity

When every column of a `query`, `query_one` or `query_opt` is read straight from one table under its own name (`select *`, `select u.* ... `, `returning *`), the macro returns that table's struct from `db/schema.jai` instead of an anonymous `Row(...)`, so the result flows into code that takes `User`.

- The table comes from the columns' origins (`schema.table.column`); the struct name from the `Name :: struct @pg_table(schema.table)` line in `<root>/db/schema.jai`.
- The name has to resolve where the query is written. The macro inserts it with `#insert,scope(call)` in two steps, `#exists(Name)` first, so a struct the caller cannot see (another module, `#scope_file`) falls back to `Row(...)` without an error.
- `row_type` (shape.jai) then checks the struct's fields against the columns, in order, by name and with `field_accepts`; any mismatch also falls back. Rows decode positionally either way.

### Schema databases

With a `db/migrations` folder, queries are described in a **schema database** built from the migrations, not in the database the URL names. `schema_databases.jai` picks it, and the service connects to it on the same server (the role needs `CREATEDB`):

- Each prefix of the migrations has a hash (`prefix_hashes` in `migrate.jai`; the last one is the schema hash). Its database is `jacket_<first 12 hex digits>`.
- The longest prefix with a database wins. Newer migrations run inside the describe's transaction, which is rolled back, so describing never writes.
- The database for the whole set is built from `template0` once the set has stayed the same for 2 s (`JACKET_PG_BUILD_DELAY_MS`), right away when there is no prefix database, or when a newer migration is `-- jacket: no-transaction`. It is migrated under a `jacket_build_*` name and renamed, so other workers never see it half built.
- After a build, all but the newest three are dropped. A database still in use is left for next time.

The dependency fingerprints are read from the same schema database.

### Project root and cache

The root is the nearest folder upward from the calling file containing `jacket.toml` or the first folder of `MIGRATIONS` (`db/`), else the file's own folder. Entries are JSON (`entry.jai`): columns, parameters, JSON types, dependencies and a schema fingerprint. A cached entry is good when its `schema_hash` (from `db/migrations/*.sql`, see `schema_hash` in `migrate.jai`) matches. Otherwise, with a URL and not offline, the fingerprints of its dependencies are re-checked through the service. Offline, a hash mismatch is stale, and anything else is used as is.

With `JACKET_PG_CHECK` set, every query used by a build is recorded in `used.txt` so the tool can prune.

### The pg tool

```
jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- prepare|check [--root DIR] [--build CMD] [--prune]
```

`prepare` builds the project online and fills the cache. `check` rebuilds offline and fails when any entry is missing or stale. The tool also has `generate`, `migrate`, `status` and `new` ([schema](schema.md)).

## How to change it

- Placeholder syntax: `sql_scan.jai` (keep `rewrite_placeholders` and `source_offset_of` in step).
- Nullability rules: `describe.jai` (`known_non_null`, `refine_from_body`, plan walk).
- Cache format: `entry.jai`; bump the entry version when it changes.
- Schema database naming, build timing and pruning: `schema_databases.jai`. The service (`describe_service.jai`) keeps one connection per database.
- Statement cache and `plan`/`verify_statements`: `statements.jai`; `expected_columns` (shape.jai) supplies the compiled column names for the check. A named `plan = ` argument is not a placeholder value: `value_arguments` (shape.jai) filters it out of the argument count.
- Macros: `typed.jai`. They are `#expand` with `@Interpolates("sql")`, so inline `%name` holes work.

Gotchas: `Rows` buffers the whole result (no portal streaming); `fetch_all(rows, allocator = context.allocator)` copies every row, and the strings, arrays and documents it points to, into one block (two passes over the type info: size, then copy), so the copies outlive `free_rows` and `array_free(items)` frees them. `query_one`, `query_opt` and `query_scalar` return values in temporary storage. Alias type overrides resolve in the module's scope, so user types in `Json(T)` overrides need `query_as`. Unreachable databases are silent in the editor (`Service.unknown`) and errors in builds.

## Configuration

### Module parameters

```jai
Pg :: #import "Extensions/Postgres"(MIGRATIONS = "db/migrations", SCHEMA = "db/schema.jai", QUERY_CACHE = "db/queries", MIN_SERVER_VERSION = 15);
```

- `MIGRATIONS`, `SCHEMA`, `QUERY_CACHE`: the migrations folder, the generated schema file and the query cache folder, relative to the project root (the defaults are the paths shown). `resolve.jai` turns them into paths with `migrations_path`, `schema_path` and `query_cache_path`; use those, never a literal `db/...`. The root is also found by the first folder of `MIGRATIONS` (see below).
- `MIN_SERVER_VERSION`: the oldest server major version the project supports (default `0`, any). It travels in the describe request, and the service answers every describe and fingerprint request from an older server with "the server is PostgreSQL 14, but this project needs at least 15". A cache hit does not talk to a server, so it is not checked.

The describe service and the pg tool import the module on their own, so they see the defaults: the service gets paths and the minimum in each request, and the tool resolves its paths through `Pg.migrations_path` and friends of its own import (a project with other paths runs the tool from a wrapper that imports the module with the same parameters).

### Environment

- `JACKET_DATABASE_URL`, then `DATABASE_URL`: the compile-time database.
- `JACKET_PG_MANAGED=1` (or `managed = true` under `[postgres]` in `jacket.toml`): with no URL, describe in a private PostgreSQL the service downloads and starts itself ([managed.md](managed.md)). It needs `db/migrations`; without them a miss is an error. Ignored when a URL is set or `JACKET_PG_OFFLINE` is. `JACKET_PG_MANAGED_VERSION` and `JACKET_PG_MANAGED_CACHE` pick the release and the cache folder.
- `JACKET_PG_OFFLINE=1`: never connect; misses are errors.
- `JACKET_PG_SEARCH_PATH`: search path for describing.
- `JACKET_PG_ROOT`: override the project root.
- `JACKET_PG_CHECK`: record used queries.
- `JACKET_PG_BUILD_DELAY_MS`: how long the migrations must stay unchanged before their schema database is built (default 2000).

## Dependencies

The wire client (`describe_statement`, `query_binary`, `query_dynamic`), `Json`, `Service` and `Io` (describe worker), SHA-256 for fingerprints. Tests: `tests/typed_scan.jai` (offline) and `tests/typed/run.sh`, which needs `JAI_PG_TEST_PORT` (optionally `JAI_PG_TEST_SOCKET_DIR`, `PSQL`, `JAIC`).
