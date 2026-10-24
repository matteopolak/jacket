# Postgres migrations

## What it is

Forward-only SQL migrations for `Extensions/Postgres`. The files are embedded in the program at compile time, so production needs nothing on disk, and applied at startup with `Pg.migrate`.

```jai
MIGRATIONS :: #run Pg.load_migrations("db/migrations");   // relative to this file

applied, error := Pg.migrate(db, MIGRATIONS);
```

## How it works

- **Files.** `db/migrations/NNNN_name.sql`. The number is the version; any width works, but versions must be unique. A file whose first line is `-- jacket: no-transaction` runs outside a transaction.
- **Loading** (`load_migrations`, `read_migrations`). Lists the folder, parses names, reads each file and hashes it with SHA-256, then sorts by version. A bad name or a repeated version is a compile error at the `#run`. A relative folder starts at the calling file's folder (`#caller_location`), so the result does not depend on the compiler's working directory.
- **Schema hash** (`schema_hash`). SHA-256 chained over each migration's name and digest, in order. Renaming, editing, adding or removing any file changes it. The describe service and the `db/schema.jai` check key on it.
- **Applying** (`migrate`):
  1. `pg_advisory_lock(hashtext('jacket_migrations'))`, so two instances never migrate at once. Released at the end.
  2. Creates `jacket_migrations(version, name, sha256, applied_at, duration_ms)` if it is missing.
  3. Compares the list with the table (`migration_status`): each migration is `PENDING`, `APPLIED`, `MODIFIED` (applied, file changed), `MISSING` (applied, not in the list) or `OUT_OF_ORDER` (pending, numbered below the newest applied one).
  4. Refuses, applying nothing, on `MODIFIED` or `MISSING` unless `allow_modified`, and on `OUT_OF_ORDER` unless `allow_out_of_order`.
  5. Applies each pending migration in its own `begin` ... `commit` with the simple protocol, so a file may hold many statements, and records it in the same transaction. A failure rolls back that migration only; the error message starts `migration <name> failed:` and keeps the server's SQLSTATE and position.
- **No-transaction files** are split into statements (`split_statements`, which skips quoted text, `E''` strings, dollar quotes and nested comments) and each is sent alone, because a multi-statement simple query is itself an implicit transaction. Such a file is recorded after its last statement; if one fails half way, the earlier statements stay applied, so write them to be safe to repeat (`if not exists`).

## How to change it

- New bookkeeping columns go in the `create table` inside `migration_status`. Existing databases already have the table, so add an `alter table ... add column if not exists` next to it.
- `split_statements` is a small SQL scanner. The psql shim (`tests/conformance/scan.jai`) has psql's full lexer; this one only needs statement ends.
- Tests: `stdlib/Extensions/Postgres/tests/migrate.jai`, with fixtures in `tests/migrations/` and `tests/migrations_bad/`. The split and loading checks run anywhere; the rest needs `JAI_PG_TEST_PORT` (see `tests/start_server.sh`).

## Configuration

`Migrate_Options`: `allow_modified`, `allow_out_of_order` (both false). The table name is fixed (`MIGRATIONS_TABLE`).

## Dependencies

`File_Utilities.file_list`, `internal/crypto/sha`, and the client's `execute_script` and `query_dynamic`. See the [design](design.md#the-migration-runner).
