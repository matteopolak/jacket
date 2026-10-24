# Postgres schema generation

## What it is

`generate` turns the project's migrations into `db/schema.jai`: a committed Jai file with a struct per table and view, an enum per Postgres enum, and the notes the ORM reads. SQL stays the source of truth ([design](design.md#layer-3-schema-migrations-and-table-structs), option C). The pg tool also runs the migrations (`migrate`, `status`, `new`).

```
jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- generate [--root DIR] [--out FILE]
jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- migrate|status [--root DIR]
jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- new "add posts" [--root DIR]
```

```jai
#load "db/schema.jai";    // User, Team, Member_Role, ...

Settings :: struct { theme: string; }      // what users.settings holds, see db/json_columns.jai
```

## How it works

1. **Schema database.** `Pg.generate_schema` connects to the server in `JACKET_DATABASE_URL` / `DATABASE_URL` (needs `CREATEDB`) and reads the migrations' schema database, `jacket_<hash12>`, the same one typed queries are described in (see [typed-queries.md](typed-queries.md#schema-databases)). When there is none it builds it from `template0` with `Pg.migrate`. Nothing is read from the developer's own database.
2. **Catalog** (`load_catalog`). Four queries: enums (`pg_enum`), columns of tables, partitioned tables, views, materialized views and foreign tables (`pg_attribute`; no system schemas, extension objects, partitions or `jacket_migrations`), and primary and foreign keys (`pg_constraint`). Column types go through `resolve_types` and `base_name` from the describe step, then `jai_type_of`, so a column has the same Jai type here and in a typed query. A view's columns are all `Nullable`: the catalog proves nothing about them.
3. **Text** (`schema_source`, a pure function of a `Catalog` and the schema hash, so it is tested without a server). Types are named `Pascal_Case` of the singular table name (`blog_posts` is `Blog_Post`); outside `public` the schema comes first. A name already taken falls back to the plural, then `_Table` / `_View`. A column that cannot be a field name (a keyword) is an error naming it.
4. **Check.** The file starts with `// schema hash: <hex>` (`Pg.schema_hash` of `db/migrations`) and runs `#run Pg.check_schema(#file)`. If the hash of the project's migrations differs, the compile stops with "db/schema.jai is stale: run the pg tool's generate". No server is involved. The project root is found as for typed queries (`find_project_root`).

### The generated file

```jai
Member_Role :: enum { owner; admin; __in_20progress; }      // 'in progress'

User :: struct @pg_table(public.users) @pg_belongs_to(team:team_id:Team:id) @pg_has_many(posts:Post:author_id:id) {
    id: s64; @pg_primary_key @pg_default
    team_id: s64;
    role: Member_Role; @pg_default
    settings: Pg.Nullable(Pg.Json(Pg.json_column_type("users.settings")));
}
```

- **Enums.** A Jai enum with the labels as member names; `Pg.enum_member_name` / `enum_label` spell labels that are not plain names (`__in_20progress`). The codecs send and read an enum by label ([postgres.md](../extensions/postgres.md)).
- **Notes** (read from `type_info(T).notes` and `members[i].notes`; a note has no spaces, and enums have none):
  - `@pg_table(schema.name)` on the struct;
  - `@pg_primary_key`, `@pg_default` (default, identity or generated: an insert may omit it) and `@pg_generated` (always generated: an insert may not set it) on columns;
  - `@pg_belongs_to(name:cols:Target:target cols)` on the table holding a foreign key, and `@pg_has_many(name:Source:source cols:own cols)` on the table it points to. Columns are comma separated. A belongs-to is named for its column without `_id`, else the target's singular name; a has-many for the source table, else `<table>_by_<column>`.
- **JSON.** Every `json` / `jsonb` column is `Pg.Json(Pg.json_column_type("table.column"))` (`schema.table.column` outside `public`). The macro looks the key up in `Json_Columns` and falls back to `Json_Raw`, so a mapping change needs no `generate`:

  ```jai
  // db/json_columns.jai, next to schema.jai; loaded by it when it exists
  Json_Columns :: Pg.Json_Column.[ .{ "users.settings", Settings } ];
  ```

  `#run Pg.check_json_columns` makes a mapping for a column that is not a json column of the schema a compile error. `Json(Json_Raw)` keeps the document as text (`.value.text`). Types used in `json_columns.jai` are looked up in the program that loads `schema.jai`.
- **Nulls.** `Pg.Nullable(T)` for every nullable column; `[] T` for arrays, as in typed queries.

## How to change it

- Generator and catalog queries: `generate.jai`. Add a note by printing it in `append_relation` and reading it from the ORM. A new column attribute goes in `Catalog_Column`, `load_relations` and the offline test's catalog.
- Staleness check, `Json_Column`, `json_column_type`, enum spelling: `schema.jai`.
- Tool commands: `Tool/main.jai`.
- Gotchas: golden files (`tests/schema/offline_expected.txt`, `expected_schema.txt`) change with any output change; regenerate and read the diff. The output must stay stable under `jaifmt`. `Json_Columns` is only visible to the schema file's scope.

## Configuration

`JACKET_DATABASE_URL`, then `DATABASE_URL` (the server holding the schema databases), `JACKET_PG_ROOT` (project root). `--root` and `--out` for the tool. `db/json_columns.jai` is optional.

## Dependencies

The wire client, `migrate` / `read_migrations` / `schema_hash`, `resolve_types`, `base_name` and `jai_type_of` (describe and shape), `find_project_root` and `migrations_of` (resolve), `schema_databases.jai`. Tests: `tests/schema.jai` (offline; with `JAI_PG_TEST_PORT` also the schema database, the golden file and enum and raw-JSON round trips) and `tests/schema/run.sh` (the tool's commands, a program built on the generated file, staleness and a bad mapping).
