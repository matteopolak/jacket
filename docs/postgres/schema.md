# Postgres schema generation

`generate` turns the project's migrations into `db/schema.jai`: a committed Jai file with a struct per table and view, an enum per Postgres enum, and the notes the [ORM](orm.md) reads. SQL stays the source of truth. The pg tool also runs migrations (`migrate`, `status`, `new`), and [`diff`](#proposing-a-migration-from-structs) proposes a migration from hand-edited structs.

```
jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- generate [--root DIR] [--out FILE]
jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- migrate|status [--root DIR]
jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- new "add posts" [--root DIR]
```

```jai
Member_Role :: enum { owner; admin; __in_20progress; }      // 'in progress'

User :: struct @pg_table(public.users) @pg_belongs_to(team:team_id:Team:id) @pg_has_many(posts:Post:author_id:id) {
    id: s64; @pg_primary_key @pg_default
    team_id: s64;
    role: Member_Role; @pg_default
    settings: Pg.Nullable(Pg.Json(Pg.json_column_type("users.settings")));
}
```

## How it works

1. **Schema database.** `Pg.generate_schema` reads the migrations' schema database (`jacket_<hash12>`, the one [typed queries](typed-queries.md#schema-databases) use), building it from `template0` if needed. Your own database is never read.
2. **Catalog.** `load_catalog` reads enums, columns (tables, partitioned tables, views, materialized views, foreign tables) and primary and foreign keys. Column types go through the same `jai_type_of` as typed queries. A view's columns are all `Nullable`.
3. **Text.** `schema_source` is a pure function of the catalog and the schema hash. Types are `Pascal_Case` of the singular table name (`blog_posts` is `Blog_Post`); outside `public` the schema comes first. A name clash falls back to the plural, then `_Table`/`_View`.
4. **Staleness check.** The file starts with `// schema hash: <hex>` and runs `#run Pg.check_schema(#file)`. If the migrations' hash differs, compilation stops with "db/schema.jai is stale: run the pg tool's generate". No server is needed.

### Notes

- On the struct: `@pg_table(schema.name)`, `@pg_belongs_to(name:cols:Target:target cols)` on the table with the foreign key, `@pg_has_many(name:Source:source cols:own cols)` on the table it points to.
- On columns: `@pg_primary_key`, `@pg_default` (insert may omit it), `@pg_generated` (insert may not set it), `@pg_json` (a `json`, not `jsonb`, column; binary COPY needs it).
- Enum labels that are not plain names are spelled like `__in_20progress` (`Pg.enum_member_name`, `enum_label`).

### JSON columns

Every `json`/`jsonb` column is `Pg.Json(Pg.json_column_type("table.column"), ...)`. Which Jai type it holds comes from `Json_Columns`, so changing a mapping needs no `generate`:

```jai
// db/json_columns.jai, next to schema.jai; loaded by it when it exists
Json_Columns :: Pg.Json_Column.[ .{ "users.settings", Settings, .{ unknown_fields = .DENY } } ];
```

Entries are `column`, `type` and `options` (`Pg.Json_Options`), in that order, or named: `.{ column = "users.settings", type = Settings }`. Unmapped columns are `Json_Raw`. `#run Pg.check_json_columns` makes a mapping for a non-json column a compile error.

## Proposing a migration from structs

`diff` is for struct-first authoring: you hand-edit the structs in `db/schema.jai` and it writes the migration that would make the migrated database match. It only proposes. The file is yours to review, nothing is applied, and SQL migrations stay the source of truth.

```
jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- diff [--root DIR] [--name NAME]
```

`--name "add posts"` writes `db/migrations/0003_add_posts.sql` (default name `schema_changes`; the number follows the newest migration, as for `new`). If nothing differs it writes no file.

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

- Generator and catalog queries: `generate.jai`. A new note is printed in `append_relation` and read by the ORM. A new column attribute goes in `Catalog_Column`, `load_relations` and the offline test's catalog.
- Staleness check, `Json_Column`, enum spelling: `schema.jai`. A new JSON option goes in `json_options.jai`.
- Tool commands: `Tool/main.jai`.
- `diff`: parser and SQL writer are in `diff.jai`. A new Jai type goes in `sql_type_of_text`; a new kind of change in `propose_create`, `propose_alter` or `compare_column`, with output in `tests/schema/diff_expected.txt`. Names (`assign_names`) and field text (`field_type`) live in `generate.jai`, so both commands agree. Statement order matters: a `create table` may use a type created above it, and `alter type ... add value` cannot be used in the transaction that adds it.
- Golden files (`tests/schema/offline_expected.txt`, `expected_schema.txt`) change with any output change, so read the diff. Output must stay stable under `jaifmt`.

## Configuration

`JACKET_DATABASE_URL`, then `DATABASE_URL` (a server where the role can create databases), `JACKET_PG_ROOT`. Tool flags `--root`, `--out`, `--name`.

## Dependencies

The wire client, `migrate`/`read_migrations`/`schema_hash`, `jai_type_of`, `find_project_root`, `schema_databases.jai`, `enum_label`. Tests: `tests/schema.jai` (offline; with `JAI_PG_TEST_PORT` also a real schema database) and `tests/schema/run.sh`.
