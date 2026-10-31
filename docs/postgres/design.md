# Postgres client and compile-time ORM

`Extensions/Postgres` is a PostgreSQL client written in Jai, with SQL that is checked while your program compiles. Four layers, each usable without the ones above it:

1. **Client.** The wire protocol on [Io](../extensions/io-engine.md) fibers and [Tls](../extensions/tls.md): auth, extended protocol, pipelines, COPY, LISTEN/NOTIFY, cancellation, a pool. Reference: [Postgres client](../extensions/postgres.md).
2. **Typed queries.** `Pg.query(db, "select ... where id = %id")` is described by a real server at compile time, so row types, nullability and argument types come from the database. [typed-queries.md](typed-queries.md).
3. **Schema and migrations.** SQL migrations are the source of truth; Jai structs are generated from them and committed. [migrations.md](migrations.md), [schema.md](schema.md), [diff.md](diff.md).
4. **ORM.** `Pg.select`, `insert`, `update` and `delete` take filters as Jai expressions and lower to layer 2. [orm.md](orm.md).

```jai
rows, error := Pg.query(db, "select id, email from users where id = %user_id");
for rows print("% <%>\n", it.id, it.email);
```

## How it works

- Compile-time code has full host access, so the client that runs in your program is the same one macros use to describe statements. There is no second implementation.
- Describe requests go to the server you name (`DATABASE_URL`) or to a [managed](managed.md) private one. Answers are cached in `db/queries`, so CI and offline builds need no server.
- The editor gets the same answers from a long-lived `jaic -interp` worker that a trusted workspace starts ([workspace trust](../tools/workspace-trust.md)). Ordinary `#run` code in the editor stays sandboxed.

## Inline parameters

`%name`, `%name.field`, `%array[i]` and `%(any expression)` inside the SQL string are arguments. `%%` is a literal `%`. Quoted strings, quoted identifiers and comments are skipped, so `like '%@example.com'` needs no escape. A `%` followed by a space is plain, so `a % b` is modulo.

This is the compiler's `@Interpolates("sql")` note, not Postgres code. The compiler parses each hole as an expression in the caller's scope, with spans pointing into the string, and rewrites the call to positional form. Type errors, hover, go to definition and rename then work on names inside the string. See [inline holes](../language/interpolates.md).

## Nullability

A describe says nothing about nulls, so the macro works them out:

- A table column is non-null if `attnotnull` is set and it is not on the nullable side of an outer join (found from the generic `EXPLAIN` plan).
- Computed columns are nullable unless they are `count(...)`, a literal, an `exists(...)`, or a `coalesce` ending in a non-null.
- Aliases override: `as "posts!"` forces non-null, `as "name?"` forces nullable, `as "at: Pg.Local_Timestamp"` sets the Jai type.
- Nullable values are `Nullable(T)` with `value` and `null`, and `or_else(x, default)`.

## How to change it

- A column has one Jai type everywhere (typed queries, `generate`, `copy`). Change `jai_type_of` once.
- Tests that need a server read `JAI_PG_TEST_PORT`. The PostgreSQL suites are in [conformance](conformance.md).

## Dependencies

[Io](../extensions/io-engine.md), [Tls](../extensions/tls.md), [Json](../extensions/json.md), SHA-256, HMAC and PBKDF2 from `internal/crypto`, and the compile-time `Service` worker for the editor.
