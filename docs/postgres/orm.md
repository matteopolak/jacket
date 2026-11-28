# Postgres ORM

`Pg.select`, `insert`, `update` and `delete` work on the table structs of `db/schema.jai` ([schema.md](schema.md)). Filters are Jai expressions passed as `Code`. They become SQL while the program builds and go through the [typed-query](typed-queries.md) path, so they are described, checked and cached like a hand-written `Pg.query`. Nothing is built at run time.

```jai
#load "db/schema.jai";

min_score := 3;
users, error := Pg.select(db, User, where = #code it.score >= min_score && Pg.like(it.email, "%@example.com"), order_by = #code it.email, limit = 20);

row, error := Pg.insert(db, User.{ email = "a@example.com", score = 1 });   // the stored row
changed, error := Pg.update(db, User, set = #code it.score = it.score + 1, where = #code it.id == row.id);
removed, error := Pg.delete(db, User, where = #code it.id == row.id);
```

| Call | Returns |
| --- | --- |
| `select(db, T, where?, order_by?, desc?, limit?, offset?, from_sql?, fields?)` | `Rows(T), Pg_Error` (with `fields = "id, email"`, `Rows(Partial(T, ...))`) |
| `for each(db, T, *error, where?, ...)` | a loop over `T` that selects only the columns the body reads |
| `insert(db, value)` | the stored row (`returning`); fields marked `@pg_default` or `@pg_generated` are left out |
| `insert_many(db, values)` | stored rows, one `insert` each |
| `copy_many(db, values)` | rows stored, via binary COPY ([below](#bulk-loading-copy)) |
| `update(db, T, set, where)`, `delete(db, T, where)` | rows changed. `where` is required (`#code true` for all rows) |
| `load_many(db, parents, "posts")`, `load_one(db, posts, "author")` | related rows for many parents in one query |

## How it works

Each call is an `#expand` macro that inserts the output of `orm_statement_code` at compile time.

1. **Table.** `T`'s `@pg_table(schema.table)` note names the table; its members are the columns. No note is a compile error.
2. **Filter walk.** `compiler_get_nodes` gives the expression tree and `orm_expression` prints SQL with a `%` hole per parameter:
   - `it.field` is a column; `it.json_field.a.b` is a `->`/`->>` path using the `@json` names.
   - Anything that does not mention `it` (`min_score`, `row.id`) is a parameter, evaluated in the caller's scope.
   - `&& || ! == != < <= > >= + - * / %` map to SQL; `== null` becomes `is null`.
   - Stub calls that only name SQL: `Pg.like`, `ilike`, `in`, `lower`, `upper`, `if_some(x, #code cond)`, and the jsonb ones `contains`, `has_key`, `json_path`.
   - `Pg.sql("lower(%(it.email)) = lower(%email)")` is raw SQL: `%(it.field)` is a column, `%name` a parameter.
   - Anything else is a compile error at that node.
3. **Parameter types.** Each parameter is typed in the caller's scope and cast in SQL (`$1 ::int8`), so Postgres never guesses.
4. **Typed query.** The macro calls `query_as(T, ...)` or `execute` with `project_file` set to the caller's file, so the cache and `.env` come from the calling project.

`where = #code it.score >= min && it.display_name != null` becomes:

```sql
select "id", "email", ... from "public"."users"
where (("score" >= $1 ::int8 ) and ("display_name" is not null))
```

`from_sql` replaces the table with your own query returning its columns: `Pg.select(db, User, from_sql = "select * from users_visible_to(%viewer)", where = ...)`.

### Column pruning

`Pg.each` is meant for `for`. It reads the loop body for `it.<field>` and selects those fields plus the primary key; other fields of `it` stay zero. If the body uses `it` whole, or reads every field, all columns are selected.

```jai
error: Pg.Pg_Error;
for Pg.each(db, User, *error, where = #code it.score >= min) print("%\n", it.email);
// select "id", "email" from "public"."users" where ("score" >= $1 ::int8 )
```

- **Inside json.** Reading only top-level keys of a `Json(S)` column fetches a smaller document with just those keys.
- **To-one relation.** `it.author.email` adds a scalar subquery to the select. `it` becomes an `Orm_Item` that `using`s `T` as `it.orm_row`, with one member per relation read. One level only.
- **Has-many.** `for post: it.posts` loads the related rows for all parents with one more query. `strategy = .LATERAL` does it in the same statement, which pays off when the server is far away and each parent has few children.

The query runs when the loop starts; a failure goes to `error` and the body never runs. The result is freed when the loop ends, so copy strings that must outlive it.

### Relations

`generate` writes relation notes (`@pg_belongs_to(author:author_id:User:id)`, `@pg_has_many(posts:Post:author_id:id)`; see [schema.md](schema.md)). They give:

- Filters through a to-one relation: `where = #code it.author.email == owner` is a scalar subquery. Paths nest. Has-many cannot be filtered on.
- Enum literals: `it.mood == .happy` takes the enum from the column.
- Loading in one query:

  ```jai
  posts, error := Pg.load_many(db, users, "posts");         // posts.items[i] belongs to users[i]
  defer Pg.free_loaded(*posts);
  authors, error := Pg.load_one(db, posts_list, "author");  // authors.items[i]: Nullable(User)
  ```

  Keys go in as one array parameter, rows come back once, and a hash table hands them out. Everything lives until `free_loaded`. Multi-column keys match pairs through `unnest`. A null key gives an empty list or null item.

## Bulk loading (COPY)

`Pg.copy_in` sends an array of structs through `COPY ... FROM STDIN (FORMAT BINARY)`, the fastest way to load many rows. `Pg.copy_many` is the form for table structs. Both return the number of rows stored and read nothing back; use `insert`/`insert_many` when you need ids or defaults.

```jai
stored, error := Pg.copy_in(db, "copy users (email, score) from stdin (format binary)", rows, .["email", "score"]);
stored, error := Pg.copy_many(db, users);

count, error := Pg.copy_out(db, "copy (select email from users) to stdout (format csv)", write_piece, user_data);
copied, error := Pg.copy_out(db, "copy users (email, score) to stdout (format binary)", User_Row, .["email", "score"]);
for copied.rows print("% %\n", it.email, it.score);
Pg.free_copy_rows(*copied);
```

The callback form of `copy_out` hands each piece to you as it arrives, in any format. The row form decodes a binary COPY into `Copy_Rows(Row)`. All of them also take a `*Pool`.

- `copy_in` feeds `run_script` from a `Script_Hooks.copy_in` hook (`copy.jai`): header, rows in 64 KiB pieces, then the trailer. `copy_put_member` is written for the row struct at compile time, one case per member, each calling `put_copy_value` for its type. Arrays use `array_param_of` and elements use `put_scalar` (`encoders.jai`), so no row walks type information.
- `columns` picks members by name in the statement's column order. Empty means every member.
- `copy_many` builds its statement at compile time from `@pg_table` and the fields without `@pg_default`/`@pg_generated`.
- `copy_out` decodes rows with the same decoders as `get`. A NULL needs a `Nullable` member; a wrong width, text format or count mismatch is an error.
- A row that cannot be encoded aborts the COPY; the error names the row and member.

Gotchas:

- Binary COPY casts nothing. Member types must match columns exactly: `s32` for `int4`, `s64` for `int8`, `[] string` for `text[]` (not `varchar[]`). Generated structs always match.
- A `Json(T)` member goes as `jsonb` unless it has `@pg_json`, which `generate` writes on `json` columns.
- Arrays of enums are refused.

## How to change it

- Pruning: `orm_read_columns` decides the columns, `orm_each_code` writes the loop.
- Relations: `orm_relation` parses the notes, `orm_related_column` writes the subquery, `orm_load_code` writes `load_many`/`load_one`. The note format is shared with `generate.jai` and `diff.jai`.
- New operator or call: a case in `orm_binary` or `orm_call`, a stub next to `like`, a use in `tests/orm/main.jai`, an error case in `tests/typed/bad.py`.
- New parameter type: `parameter_cast`. An unknown type gets no cast.
- New COPY type: add it to `scalar_oid` and `put_scalar` (`encoders.jai`; query arguments and array elements get it too, once it is in `PLAIN_TYPES`) and to `put_copy_value`, which has its own typed branch per type. A type with no branch there is an error naming it.

Gotchas:

- A parameter expression is evaluated once per call, in the caller's scope.
- Inside an `#expand` macro, `return f()` of a two-value call crashes the compiler today; the generated code binds results to locals first.
- A `Nullable(T)` zero value is not null.

## Configuration

Same as [typed queries](typed-queries.md#configuration): `DATABASE_URL`, `JACKET_PG_ROOT`, `JACKET_PG_OFFLINE`.

## Dependencies

`typed.jai` and the describe service, `query.jai` (`run_script`, `Script_Hooks`), `codecs.jai`, `Compiler` (`compiler_get_nodes`, `compiler_report`) and `Program_Print`. Tests: `test_copy_out` in `tests/server.jai`, `test_copy` in `tests/orm/main.jai`, `tests/orm/`, run by step 8 of `tests/typed/run.sh`, and the `orm` cases of `tests/typed/bad.py`.
