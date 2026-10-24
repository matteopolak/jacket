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
2. **Resolve.** `resolve.jai` (`prepare_query`, memoised per call site) looks for `db/queries/<id>.json` under the project root. On a miss it asks the describe service.
3. **Describe.** `describe_service.jai` is a `jaic -interp` worker (`Service.serve`) that holds connections per URL, backs off on failure (1 s doubling to 30 s) and memoises connect errors. `describe.jai` does Parse + Describe, resolves type names (domains, arrays, enums), reads `attnotnull`, and runs `EXPLAIN (VERBOSE, FORMAT JSON)` on the generic plan to find columns made nullable by outer joins. Aliases can force nullability or a type: `name!` (non-null), `name?` (nullable), `name: Jai_Type`.
4. **Shape.** `shape.jai` builds `Row(shape)`, a struct with one doc-commented field per column, and checks arguments against the parameter types (`s64` does not narrow to `integer`). Errors point at the argument or, for server errors, at the offending byte of the string literal.
5. **Run.** `typed.jai` expands into a call that sends the rewritten statement (binary results when every column is decodable). `for rows` decodes one row at a time.

`query_one` and `query_scalar` return a `.NOT_FOUND` error when no row matches, and a `.USAGE` error for more than one. `query_as(T, ...)` checks a hand-written struct: columns match fields by name, nullable columns need `Nullable`, and `Json(T)` fields say what a json column holds.

### Project root and cache

The root is the nearest folder upward from the calling file containing `jacket.toml` or `db/`, else the file's own folder. Entries are JSON (`entry.jai`): columns, parameters, JSON types, dependencies and a schema fingerprint. A cached entry is good when its `schema_hash` (from `db/migrations/*.sql`) matches. Otherwise, with a URL and not offline, the fingerprints of its dependencies are re-checked through the service. Offline, a hash mismatch is stale, and anything else is used as is.

With `JACKET_PG_CHECK` set, every query used by a build is recorded in `used.txt` so the tool can prune.

### The pg tool

```
jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- prepare|check [--root DIR] [--build CMD] [--prune]
```

`prepare` builds the project online and fills the cache. `check` rebuilds offline and fails when any entry is missing or stale.

## How to change it

- Placeholder syntax: `sql_scan.jai` (keep `rewrite_placeholders` and `source_offset_of` in step).
- Nullability rules: `describe.jai` (`known_non_null`, `refine_from_body`, plan walk).
- Cache format: `entry.jai`; bump the entry version when it changes.
- Macros: `typed.jai`. They are `#expand` with `@Interpolates("sql")`, so inline `%name` holes work.

Gotchas: `Rows` buffers the whole result (no portal streaming); `fetch_all` borrows the rows' data, so free with `free_rows`. `query_one`, `query_opt` and `query_scalar` return values in temporary storage. Alias type overrides resolve in the module's scope, so user types in `Json(T)` overrides need `query_as`. Unreachable databases are silent in the editor (`Service.unknown`) and errors in builds.

## Configuration

- `JACKET_DATABASE_URL`, then `DATABASE_URL`: the compile-time database.
- `JACKET_PG_OFFLINE=1`: never connect; misses are errors.
- `JACKET_PG_SEARCH_PATH`: search path for describing.
- `JACKET_PG_ROOT`: override the project root.
- `JACKET_PG_CHECK`: record used queries.

## Dependencies

The wire client (`describe_statement`, `query_binary`, `query_dynamic`), `Json`, `Service` and `Io` (describe worker), SHA-256 for fingerprints. Tests: `tests/typed_scan.jai` (offline) and `tests/typed/run.sh`, which needs `JAI_PG_TEST_PORT` (optionally `JAI_PG_TEST_SOCKET_DIR`, `PSQL`, `JAIC`).
