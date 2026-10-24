# Postgres bulk loading (COPY)

## What it is

`Pg.copy_in` sends an array of structs through `COPY ... FROM STDIN (FORMAT BINARY)`, the fastest way to load many rows. `Pg.copy_many` is the ORM form for table structs ([orm.md](orm.md)).

```jai
stored, error := Pg.copy_in(db, "copy users (email, score) from stdin (format binary)", rows, .["email", "score"]);
stored, error := Pg.copy_many(db, users);   // copy "public"."users" (<fields an insert sets>) ...
```

Both return the number of rows the server stored (from the `COPY n` tag). Neither reads rows back; use `insert` / `insert_many` when you need ids or defaults.

`Pg.copy_out` is the other direction, `COPY ... TO STDOUT`:

```jai
count, error := Pg.copy_out(db, "copy (select email from users) to stdout (format csv)", write_piece, user_data);
copied, error := Pg.copy_out(db, "copy users (email, score) to stdout (format binary)", User_Row, .["email", "score"]);
for copied.rows print("% %\n", it.email, it.score);
Pg.free_copy_rows(*copied);
```

The first form hands each `CopyData` piece to a callback as it arrives (any format; one row per piece in text and CSV) and returns the row count. The second reads a binary COPY into a `Copy_Rows(Row)`; `free_copy_rows` frees the rows and everything they point into. Both also take a `*Pool`.

## How it works

- `copy_in` runs the statement with `run_script` and feeds it from a `Script_Hooks.copy_in` hook (`copy.jai`). The hook writes the binary header, then rows, about 64 KiB at a time, then the `-1` trailer.
- Each row is a field count and, per member, a length (`-1` for null) and the member's binary form. The forms are the ones binary parameters use (`encode_binary`, `encode_array`, `numeric_binary` in `codecs.jai`); an enum goes as its label and `[] u8` as `bytea`.
- `columns` picks the members by name, in the statement's column order. Empty means every member, in declaration order.
- `copy_many` builds the statement at compile time from `@pg_table` and the fields without `@pg_default` / `@pg_generated`, like `insert`.
- `copy_out` runs the statement with `run_script` and a `Script_Hooks.copy_out` hook. The callback form passes the pieces straight through. The row form appends them to a result's data, then decodes after the `COPY n` tag: header, then per row a field count and length-prefixed values (`-1` is NULL), each decoded by the member's type with `decode` (the same decoders `get` uses). `copy_member` is generated per struct by `copy_member_code`, so the members' types are known at compile time. Members are matched to columns by position (`columns` picks them by name, as for `copy_in`). A NULL needs a `Nullable` member; a wrong width, a text-format statement, a count mismatch or a row count that disagrees with the tag is an error.
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

- `query.jai` (`run_script`, `Script_Hooks`, `result_allocator`), `codecs.jai` (binary encoders), `orm.jai` (`copy_many`).
- Tests: `test_copy_out` in `tests/server.jai` (streams in text, CSV and binary, rows of every kind of column, the `copy_in` round trip, error cases, a pool), and `test_copy` in `tests/orm/main.jai` (every kind of column, nulls, several chunks, a width mismatch), run by step 8 of `tests/typed/run.sh`.
