# Postgres schema generation

`generate` turns the project's migrations into `db/schema.jai`: a committed Jai file with a struct per table and view, an enum per Postgres enum, and the notes the [ORM](orm.md) reads. SQL stays the source of truth. The pg tool also runs migrations (`migrate`, `status`, `new`), and [`diff`](diff.md) proposes a migration from hand-edited structs.

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

## How to change it

- Generator and catalog queries: `generate.jai`. A new note is printed in `append_relation` and read by the ORM. A new column attribute goes in `Catalog_Column`, `load_relations` and the offline test's catalog.
- Staleness check, `Json_Column`, enum spelling: `schema.jai`. A new JSON option goes in `json_options.jai`.
- Tool commands: `Tool/main.jai`.
- Golden files (`tests/schema/offline_expected.txt`, `expected_schema.txt`) change with any output change, so read the diff. Output must stay stable under `jaifmt`.

## Configuration

`JACKET_DATABASE_URL`, then `DATABASE_URL` (a server where the role can create databases), `JACKET_PG_ROOT`. Tool flags `--root`, `--out`.

## Dependencies

The wire client, `migrate`/`read_migrations`/`schema_hash`, `jai_type_of`, `find_project_root`, `schema_databases.jai`. Tests: `tests/schema.jai` (offline; with `JAI_PG_TEST_PORT` also a real schema database) and `tests/schema/run.sh`.
