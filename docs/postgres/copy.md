# Postgres bulk loading (COPY)

## What it is

`Pg.copy_in` sends an array of structs through `COPY ... FROM STDIN (FORMAT BINARY)`, the fastest way to load many rows. `Pg.copy_many` is the ORM form for table structs ([orm.md](orm.md)).

```jai
stored, error := Pg.copy_in(db, "copy users (email, score) from stdin (format binary)", rows, .["email", "score"]);
stored, error := Pg.copy_many(db, users);   // copy "public"."users" (<fields an insert sets>) ...
```

Both return the number of rows the server stored (from the `COPY n` tag). Neither reads rows back; use `insert` / `insert_many` when you need ids or defaults.

## How it works

- `copy_in` runs the statement with `run_script` and feeds it from a `Script_Hooks.copy_in` hook (`copy.jai`). The hook writes the binary header, then rows, about 64 KiB at a time, then the `-1` trailer.
- Each row is a field count and, per member, a length (`-1` for null) and the member's binary form. The forms are the ones binary parameters use (`encode_binary`, `encode_array`, `numeric_binary` in `codecs.jai`); an enum goes as its label and `[] u8` as `bytea`.
- `columns` picks the members by name, in the statement's column order. Empty means every member, in declaration order.
- `copy_many` builds the statement at compile time from `@pg_table` and the fields without `@pg_default` / `@pg_generated`, like `insert`.
- A row that cannot be encoded aborts the COPY with `CopyFail`; the call returns the server's error, whose message names the row and member.

## How to change it

- **A new type.** Add it to `encode_binary` (parameters get it too) or to `put_copy_value` if it only makes sense in COPY.
- **Gotchas.**
  - Binary COPY casts nothing. Member types must match the columns exactly: `s32` for `int4`, `s64` for `int8`, `[] string` for `text[]` (not `varchar[]`). A mismatch is the server's error, never a bad row. Generated `db/schema.jai` structs always match.
  - `json` and `jsonb` differ in binary. A `Json(T)` member goes as `jsonb` unless it has the `@pg_json` note, which `generate` writes on `json` columns (and `diff` reads).
  - Arrays of enums are refused: their element type id is per database.
  - A `Nullable(T)` zero value is not null; set `.null = true`.

## Configuration

None.

## Dependencies

- `query.jai` (`run_script`, `Script_Hooks`), `codecs.jai` (binary encoders), `orm.jai` (`copy_many`).
- Tests: `test_copy` in `tests/orm/main.jai` (every kind of column, nulls, several chunks, a width mismatch), run by step 8 of `tests/typed/run.sh`.
