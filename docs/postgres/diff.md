# Postgres schema diff

`diff` is the pg tool command for struct-first authoring. You hand-edit the structs in `db/schema.jai`, and `diff` writes the migration that would make the migrated database match. It only proposes: the file is yours to review, nothing is applied, and SQL migrations stay the source of truth ([schema.md](schema.md)).

```
jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- diff [--root DIR] [--name NAME]
```

`--name "add posts"` writes `db/migrations/0003_add_posts.sql` (default name `schema_changes`; the number follows the newest migration, as for `new`). If nothing differs it writes no file.

## How it works

1. **Read the structs.** `Pg.parse_schema_text` reads `db/schema.jai` as text, in the shape `generate` writes. Anything else in a table struct or enum (a default value, fields on one line) is an error naming the line. A text parser needs no second compile and works on a half-edited file.
2. **Read the catalog.** `Pg.migrated_catalog` returns the catalog of the migrations' schema database. Your database is not read.
3. **Compare.** `Pg.propose_migration` is a pure function of the catalog and the parsed structs. Structs match tables by `@pg_table`, enums by Jai name or the `/// Enum schema.name.` comment.
4. **Write** in an order that runs: schemas, enums, new tables, altered tables, then comments for what it could not do.

| In the structs | Proposed |
| --- | --- |
| struct with no table | `create table` (`not null` unless `Nullable`, primary key, foreign keys) |
| new field | `alter table ... add column` |
| field became `Nullable` | `drop not null` |
| new enum, new members | `create type`, `alter type ... add value` |
| new schema in `@pg_table` | `create schema if not exists` |

What it cannot express becomes a `-- TODO:` comment: removed fields and structs (the `drop` is commented out because it destroys data), possible renames, type changes, `set not null`, new `not null` columns without a default, removed enum labels. Relation notes on existing tables and primary key changes are not compared.

## How to change it

- Parser and SQL writer: `diff.jai`. A new Jai type goes in `sql_type_of_text`; a new kind of change in `propose_create`, `propose_alter` or `compare_column`, with output in `tests/schema/diff_expected.txt`.
- Names (`assign_names`) and field text (`field_type`) live in `generate.jai`, so both commands agree.
- Statement order matters: a `create table` may use a type created above it, and `alter type ... add value` cannot be used in the transaction that adds it.

## Configuration

`JACKET_DATABASE_URL`, then `DATABASE_URL`, with the right to create databases. `--root`, `--name`.

## Dependencies

`migrated_catalog`/`load_catalog` (`generate.jai`), `schema_databases.jai`, `read_migrations`, `enum_label`. Tests: `tests/schema.jai` and `tests/schema/run.sh`.
