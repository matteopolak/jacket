# Postgres ORM

## What it is

`Pg.select`, `Pg.insert`, `Pg.update` and `Pg.delete` work on the table structs of `db/schema.jai` ([schema.md](schema.md)). Filters are Jai expressions passed as `Code`. They are compiled to SQL while the program builds, and the statement goes through the typed-query path ([typed-queries.md](typed-queries.md)), so it is described, checked and cached like a hand-written `Pg.query`. Nothing is built at run time. The overall design is [Layer 4](design.md#layer-4-the-orm).

```jai
#load "db/schema.jai";

min_score := 3;
users, error := Pg.select(db, User, where = #code it.score >= min_score && Pg.like(it.email, "%@example.com"), order_by = #code it.email, limit = 20);

row, error := Pg.insert(db, User.{ email = "a@example.com", score = 1 });   // returns the stored row
changed, error := Pg.update(db, User, set = #code it.score = it.score + 1, where = #code it.id == row.id);
removed, error := Pg.delete(db, User, where = #code it.id == row.id);
```

| Call | Returns | Statement |
| --- | --- | --- |
| `select(db, T, where?, order_by?, desc?, limit?, offset?)` | `Rows(T), Pg_Error` | `select <every column> from <table> where ... order by ... limit ... offset ...` |
| `insert(db, value)` | `T, Pg_Error` | `insert ... returning <every column>` |
| `insert_many(db, values)` | `[] T, Pg_Error` | one `insert` per value, stopping at the first error |
| `update(db, T, set, where)` | `s64, Pg_Error` (rows changed) | `update <table> set ... where ...` |
| `delete(db, T, where)` | `s64, Pg_Error` (rows deleted) | `delete from <table> where ...`; `where` is required (`#code true` for every row) |

## How it works

Each call is an `#expand` macro whose body is `#insert #run orm_statement_code(...)` (or `orm_insert_code`). At compile time:

1. **Table.** `T`'s `@pg_table(schema.table)` note names the table; its members (minus constants and `using`) are the columns, in order. A struct without the note is a compile error.
2. **Filter walk.** `compiler_get_nodes` on the `where` / `order_by` / `set` code gives the expression tree. `orm_expression` turns it into SQL text with a `%` hole per parameter:
   - `it.field` is `"field"`; `it.json_field.a.b` is `("field"->'a'->>'b')::<type>` with keys from the `@json` names of the column's `Json(T)` type.
   - Number literals stay inline, string literals become quoted SQL literals.
   - Any other expression that does not mention `it` (`min_score`, `row.id`, `ids`) is a parameter. Its Jai text is printed back with `Program_Print` and evaluated where the filter was written.
   - `&& || ! == != < <= > >= + - * / %` map to SQL; `== null` / `!= null` become `is null` / `is not null`.
   - `Pg.like`, `Pg.ilike`, `Pg.in` (`= any($n)`), `Pg.lower`, `Pg.upper` and `Pg.if_some(x, #code cond)` (`(x is null or cond)`, with `x` standing for its value inside `cond`) are the only calls. These procedures do nothing at run time; they only name SQL.
   - Anything else (a call to another procedure, `it` passed whole) is a compile error at that node.
3. **Parameter types.** The macro first emits `Orm_P<i> :: type_of((#insert,scope(where) "<text>"))` for each parameter, so the compiler types it in the caller's scope. Each hole gets a cast from that type (`$1 ::int8`, `$2 ::text`), so Postgres never has to guess.
4. **Typed query.** The emitted code calls `query_as(T, db, <sql>, <parameters>)` (or `execute` for update/delete). From there it is a typed query: the statement is described against the database or the schema database, the result columns are checked against `T`, and the description is cached in `db/queries`.

Example: `where = #code it.score >= min && Pg.like(it.email, pattern) && it.display_name != null` becomes

```sql
select "id", "email", ... from "public"."users"
where ((("score" >= $1 ::int8 ) and ("email" like $2 ::text )) and ("display_name" is not null))
```

`insert` leaves out fields marked `@pg_default` or `@pg_generated`, so the database fills them, and reads the stored row back with `returning`.

## How to change it

- **A new operator or call.** Add a case to `orm_binary` or `orm_call` in `orm.jai`, a stub procedure next to `like` if it is a call, a use in `tests/orm/main.jai`, and an error case in `tests/typed/bad.py` if it can be misused.
- **A new parameter type.** `parameter_cast` maps a Jai type to the cast written after its hole. A type it does not know gets no cast, and Postgres infers one.
- **Gotchas.**
  - The filter is walked as written, so a parameter expression is evaluated once per call, in the caller's scope.
  - Inside an `#expand` macro, `return f()` of a two-value call crashes the compiler today. The generated code binds the results to locals first.
  - A `Nullable(T)` zero value is not null (`null` defaults to `false`).
- **Not yet built** (see the design): column pruning from the loop body, to-one joins and to-many batched loading through `@pg_belongs_to` / `@pg_has_many`, `Pg.sql` fragments and `from_sql`, `Pg.contains` / `has_key` / `json_path`, `Json_Maybe` comparisons, and COPY for large `insert_many`.

## Configuration

The same as typed queries: `DATABASE_URL` / `JACKET_DATABASE_URL`, `JACKET_PG_ROOT`, `JACKET_PG_OFFLINE` ([typed-queries.md](typed-queries.md#configuration)).

## Dependencies

- `typed.jai` (`query_as`, `execute`) and the describe service behind it.
- `Compiler` (`compiler_get_nodes`, `compiler_report`) and `Program_Print` (printing parameter expressions).
- Tests: `tests/orm/` (a hand-written `db/schema.jai` and `main.jai`, run by step 8 of `tests/typed/run.sh` against a local server) and the `orm ...` cases of `tests/typed/bad.py`.
