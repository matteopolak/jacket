# Postgres typed queries

`Pg.query`, `query_one`, `query_opt`, `execute`, `query_scalar` and `query_as` are macros that check a raw SQL string against a real database while the program compiles. Row types, nullability, argument types and server errors (with source spans) all come from the database.

```jai
rows, error := Pg.query(db, "select id, email from users where id = %", user_id);
for rows print("% %\n", it.id, it.email);
Pg.free_rows(*rows);
```

`query_one` and `query_scalar` return `.NOT_FOUND` for no row and `.USAGE` for more than one. `query_as(T, ...)` checks a hand-written struct: columns match fields by name, nullable columns need `Nullable`, and `Json(T)` fields say what a json column holds.

## How it works

1. **Rewrite.** `sql_scan.jai` turns each `%` into `$n`, skipping quotes, comments and dollar quotes (`%%` is a literal `%`; a Postgres `$n` is rejected). The statement id is a hash of the rewritten text.
2. **Resolve.** `resolve.jai` looks for `<QUERY_CACHE>/<id>.json` (`db/queries`) under the project root. On a miss it asks the describe service, which uses the URL, or with managed mode on a [managed server](managed.md).
3. **Describe.** `describe_service.jai` is a `jaic -interp` worker that keeps connections per URL and backs off on failure. `describe.jai` does Parse + Describe, resolves domains, arrays and enums, reads `attnotnull`, and runs `EXPLAIN` on the generic plan to find outer-join nullability. Aliases override: `name!`, `name?`, `name: Jai_Type`.
4. **Shape.** `shape.jai` builds `Row(shape)` and checks arguments against parameter types (`s64` does not narrow to `integer`). Errors point at the argument, or at the byte in the string for server errors.
5. **Arguments.** The macros take `..Any`, so the call boxes each argument, but they do not look at the box to find the type. `arguments_code` (`shape.jai`) reads each argument's static type from the call and writes the encoding: plain types, `Nullable`s and arrays of them go to `argument_param` (`encoders.jai`), which calls `param_of` or `array_param_of` for that type with no test of the type at run time. Other types (enums, `Json(T)`, `Pg.NULL`) and an argument list spread from an array (`..values`) are encoded by `encode_param` as `query_dynamic` does. A spread of a fixed `[N] Any` counts as N arguments in the placeholder check (`argument_count`, `resolve.jai`); one of a `[] Any` or `[..] Any` is only checked at run time. An argument whose boxed type differs from the one at the call also goes to `encode_param`.
6. **Run.** `typed.jai` expands to a call that uses a cached named statement: parsed once per connection, then only bound. `plan = .CUSTOM` uses the unnamed statement.

Entries in the cache are JSON with columns, parameters, dependencies and a schema fingerprint. A cached entry is good when its `schema_hash` (from `db/migrations/*.sql`) matches. With a URL, dependency fingerprints are re-checked. Offline, a hash mismatch is stale.

When every column comes straight from one table (`select *`, `returning *`), the macro returns that table's struct from `db/schema.jai` instead of an anonymous `Row`, so the result flows into code that takes `User`. If the name is not visible at the call or the fields differ, it quietly falls back to `Row`.

## Streaming and pipelines

`Pg.stream(c, *error, sql, ..; batch = 1000)` is `query` for results too big to hold. At most one batch is in memory; strings in `it` point into it, so copy what outlives the iteration. `break` cancels the statement.

```jai
error: Pg.Pg_Error;
for Pg.stream(c, *error, "select id, body from events where day = %", day) total += it.body.count;
```

`queue` and `queue_execute` add checked statements to a `*Pipeline` and return a `Pending`; `take` collects one. The batch runs as one implicit transaction, so after an error the rest fail with `.ABORTED`.

```jai
p := Pg.pipeline_enter(c);
user := Pg.queue(p, "select id, email from users where id = %", id);
seen := Pg.queue_execute(p, "update users set seen_at = now() where id = %", id);
rows, error := Pg.take(p, user);   // one round trip for both
count, error = Pg.take(p, seen);
Pg.pipeline_exit(p);
```

## Schema databases

With a `db/migrations` folder, queries are described in a database built from the migrations (`jacket_<hash>`), not the one the URL names. The role needs `CREATEDB`.

- Each prefix of the migrations has a hash. The longest prefix with a database wins, and newer migrations run in the describe's transaction, which is rolled back.
- The full set is built from `template0` once it has been unchanged for 2 s (`JACKET_PG_BUILD_DELAY_MS`), then renamed into place. The newest three databases are kept.
- If a migration fails, the build reports it at its line in the `.sql` file and checks the queries against the schema before it. Those answers are used for that compile only, never cached.

## The pg tool

```
jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- prepare|check [--root DIR] [--build CMD] [--prune]
```

`prepare` builds online and fills the cache. `check` rebuilds offline and fails if an entry is missing or stale. The tool also has `generate`, `migrate`, `status`, `new` and `diff` ([schema](schema.md)).

## Time arithmetic

`Pg.days(n)`, `hours`, `minutes` and `seconds` build a `Pg.Interval`; `+` and `-` work with `Pg.Timestamp`. A day is a calendar day, the others are exact. The operators are UTC; SQL's `now() - interval '7 days'` uses the session zone. With a namespaced import, write `Pg.operator -(a, b)` or `using Pg;`.

## JSON columns

A `json`/`jsonb` column is a `Json_Raw` (its text) unless an alias override, a `Json(T)` field in `query_as`, or a `Json_Columns` entry names its type. `Pg.json_value(raw)` reads any document; free it with `free_json` from the Json module.

Decoding strictness is `Pg.Json_Options` (`unknown_fields`, `missing_fields`, `duplicate_keys`). Layers, each falling through `.INHERIT` to the next: the connection (`Config.json_options`), the column (`Json(T, OPTIONS)`), then the query (`json = .{ unknown_fields = .DENY }`). A struct's own `@json(...)` notes win over all of them.

## How to change it

- Placeholder syntax: `sql_scan.jai`. Keep `rewrite_placeholders` and `source_offset_of` in step.
- Nullability rules: `describe.jai`.
- Cache format: `entry.jai`. Entries record `ENTRY_FORMAT`; bump it when the description, the `%` rewrite or the alias syntax changes, and old files are described again.
- Schema database naming and pruning: `schema_databases.jai`. Statement cache: `statements.jai`.
- Macros are `#expand` with `@Interpolates("sql")` in `typed.jai`.
- Row decoding (`read_row_code`) and the copy in `fetch_all` (`copy_size_code`, `copy_into_code`) are written per row type at compile time. A new kind of owned member (a string, array or pointer is one) goes in `copy_needed` and both generators.

Gotchas:

- `Rows` buffers the whole result. `fetch_all` copies rows and everything they point to into one block that outlives `free_rows`.
- `query_one`, `query_opt` and `query_scalar` return values in temporary storage.
- Alias type overrides resolve in the module's scope, so user types in `Json(T)` overrides need `query_as`.
- Unreachable databases are silent in the editor and errors in builds.

## Configuration

```jai
Pg :: #import "Extensions/Postgres"(MIGRATIONS = "db/migrations", SCHEMA = "db/schema.jai", QUERY_CACHE = "db/queries", MIN_SERVER_VERSION = 15);
```

`MIN_SERVER_VERSION` makes describe fail against an older server. A cache hit never talks to a server, so it is not checked. Use `migrations_path`, `schema_path` and `query_cache_path`, never literal `db/...`.

- `JACKET_DATABASE_URL`, then `DATABASE_URL`: the compile-time database. Else the same two keys are read from `<root>/.env` (the environment wins; no other keys are loaded).
- `JACKET_PG_MANAGED=1` or `[postgres] managed = true` in `jacket.toml`: describe in a [managed server](managed.md) when no URL is set. `JACKET_PG_MANAGED_VERSION`, `JACKET_PG_MANAGED_CACHE`.
- `JACKET_PG_OFFLINE=1`: never connect.
- `JACKET_PG_SEARCH_PATH`, `JACKET_PG_ROOT`, `JACKET_PG_CHECK` (record used queries for `--prune`), `JACKET_PG_BUILD_DELAY_MS`.

The project root is the nearest folder upward with `jacket.toml` or `db/`.

## Dependencies

The wire client, `Json`, `Service` and `Io`, SHA-256. Tests: `tests/typed_scan.jai` (offline) and `tests/typed/run.sh` (needs `JAI_PG_TEST_PORT`).
