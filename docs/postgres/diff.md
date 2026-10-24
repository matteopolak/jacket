# Postgres schema diff

## What it is

`diff` is the pg tool command for struct-first authoring as scaffolding. You hand-edit the structs in `db/schema.jai`, and `diff` writes the migration that would make the migrated database match them. It only proposes: the file is yours to review and edit, nothing is applied, and SQL migrations stay the source of truth ([schema.md](schema.md), [design](design.md#layer-3-schema-migrations-and-table-structs), option C).

```
jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- diff [--root DIR] [--name NAME]
```

With `--name "add posts"` the file is `db/migrations/0003_add_posts.sql`; the default name is `schema_changes`. The number follows the newest migration, as for `new` ([migrations.md](migrations.md)). When nothing differs it prints so and writes no file.

## How it works

1. **Read the structs.** `Pg.parse_schema_text` reads `db/schema.jai` as text. It accepts the shape `generate` writes: `Name :: struct @pg_table(schema.table) @pg_belongs_to(...) {`, one `name: Type; @note` per line, and a `}` in the first column; enums with one plain member per line. Structs without `@pg_table` are skipped. Anything else in a table struct or an enum (a default value, fields on one line, an enum member with a value) is an error naming the line. A parser was chosen over compiling a program that walks `type_info`, because it needs no second compile and works while the file is half-edited. The `check_schema` hash covers the migrations only, so editing structs never makes the file stale, and the tool never compiles the file anyway.
2. **Read the catalog.** `Pg.migrated_catalog` (shared with `generate`) returns the catalog of the migrations' schema database on the server in `DATABASE_URL`. Your database is not read.
3. **Compare** (`Pg.propose_migration`, a pure function of a `Catalog` and a `Schema_Description`, so it is tested without a server). Structs match tables by the `@pg_table` note, and enums by Jai name or by the `/// Enum schema.name.` comment `generate` writes; a new enum is `public.<lower-case type name>` unless it has that comment.
4. **Write.** The file has sections in an order that runs: schemas, enums, new tables, altered tables, then comments for what it could not do.

| In the structs | Proposed |
| --- | --- |
| a struct whose table is missing | `create table`, with `not null` unless `Pg.Nullable`, `primary key (...)` from `@pg_primary_key`, and a `foreign key` per `@pg_belongs_to` whose target is a struct of the file |
| a new field | `alter table ... add column` |
| a field that became `Pg.Nullable` | `alter column ... drop not null` |
| a new enum | `create type ... as enum (...)` |
| new enum members | `alter type ... add value`, placed with `before` / `after` |
| a new schema in `@pg_table` | `create schema if not exists` |

Jai types go back to Postgres types by the inverse of `jai_type_of`: `s64` is `bigint`, `string` is `text`, `Pg.Timestamp` is `timestamptz`, any `Pg.Json(...)` is `jsonb`, `[] T` is `T[]`, a file enum is its type.

What it cannot express becomes a `-- TODO:` comment, counted in the summary it prints:

- a removed field or struct: the `drop column` / `drop table` is **commented out** under a warning, because it destroys data;
- a column that was added and another removed in the same table: a hint that it may be a rename;
- a changed type: the old and new Jai types and a sketch of `alter column ... type`;
- a field that is no longer nullable: the commented `set not null`, which fails on existing nulls;
- a new `not null` column with no `@pg_default`: it fails on a table with rows;
- `@pg_default` and `@pg_generated` on new columns: the default value is not known;
- a type with no Postgres mapping: the column is left out;
- an enum label that was removed: Postgres cannot drop one.

`@pg_belongs_to` / `@pg_has_many` changes on tables that already exist, and changes to `@pg_primary_key`, are not compared.

## How to change it

- Parser and SQL writer: `diff.jai`. A new Jai type goes in `sql_type_of_text`; a new kind of change goes in `propose_create`, `propose_alter` or `compare_column`, with the output in `tests/schema/diff_expected.txt`.
- The names (`assign_names`) and the field text (`field_type`) are exported from `generate.jai` so that a struct is compared with the type `generate` would write for the same column. Change them there and both commands follow.
- The tool command is `propose_diff` in `Tool/main.jai`; `next_migration_path` is shared with `new`.
- Gotchas: the golden file changes with any wording change, so read the diff. Statement order matters, since a `create table` may use a type created above it. `alter type ... add value` cannot be used in the same transaction that adds it, so do not follow it with a default that uses the value.

## Configuration

`JACKET_DATABASE_URL`, then `DATABASE_URL`: a server where the user may `create database`, as for `generate`. `--root` is the project root, `--name` the migration's name.

## Dependencies

`migrated_catalog` and `load_catalog` (`generate.jai`), `schema_databases.jai`, `read_migrations` (`migrate.jai`), `enum_label` (`schema.jai`). Tests: `tests/schema.jai` (the golden SQL for a catalog and edited structs, the round trip of the generated file, parse errors; with `JAI_PG_TEST_PORT` also a real schema database, and running the proposal on a scratch database) and `tests/schema/run.sh` (the tool command).
