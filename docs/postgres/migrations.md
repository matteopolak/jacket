# Postgres migrations

Forward-only SQL migrations for `Extensions/Postgres`. The files are embedded in the program at compile time, so production needs nothing on disk, and applied at startup with `Pg.migrate`.

```jai
MIGRATIONS :: #run Pg.load_migrations("db/migrations");   // relative to this file

applied, error := Pg.migrate(db, MIGRATIONS);
```

## How it works

- **Files.** `db/migrations/NNNN_name.sql`. The number is the version and must be unique. A first line of `-- jacket: no-transaction` runs the file outside a transaction.
- **Loading.** `load_migrations` lists the folder, parses names, hashes each file with SHA-256 and sorts by version. A bad name or repeated version is a compile error at the `#run`.
- **Schema hash.** `schema_hash` chains SHA-256 over each migration's name and digest. Any rename, edit, add or remove changes it. The describe service and the `db/schema.jai` check key on it.
- **Applying.** `migrate` takes an advisory lock, creates `jacket_migrations` if missing, and compares the list with it (`migration_status`: `PENDING`, `APPLIED`, `MODIFIED`, `MISSING`, `OUT_OF_ORDER`). It refuses, applying nothing, on `MODIFIED` or `MISSING` unless `allow_modified`, and on `OUT_OF_ORDER` unless `allow_out_of_order`. Each pending file runs in its own transaction and is recorded in it. A failure rolls back that migration only and the error starts `migration <name> failed:`.
- **No-transaction files** are split into statements (`split_statements`) and sent one by one. If one fails half way, the earlier ones stay applied, so write them to be repeatable (`if not exists`).

## How to change it

- New bookkeeping columns go in the `create table` inside `migration_status`. Existing databases already have the table, so add an `alter table ... add column if not exists` next to it.
- `split_statements` only needs statement ends; the psql shim in `tests/conformance/scan.jai` has the full lexer.

## Configuration

`Migrate_Options`: `allow_modified`, `allow_out_of_order` (both false). The table name is fixed (`MIGRATIONS_TABLE`).

## Dependencies

`File_Utilities.file_list`, `internal/crypto/sha`, the client's `execute_script` and `query_dynamic`. Tests: `tests/migrate.jai`, fixtures in `tests/migrations*/`; most need `JAI_PG_TEST_PORT`.
