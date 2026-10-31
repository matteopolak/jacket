# Postgres bulk loading (COPY)

`Pg.copy_in` sends an array of structs through `COPY ... FROM STDIN (FORMAT BINARY)`, the fastest way to load many rows. `Pg.copy_many` is the [ORM](orm.md) form for table structs.

```jai
stored, error := Pg.copy_in(db, "copy users (email, score) from stdin (format binary)", rows, .["email", "score"]);
stored, error := Pg.copy_many(db, users);
```

Both return the number of rows stored. Neither reads rows back; use `insert`/`insert_many` when you need ids or defaults.

`Pg.copy_out` is the other direction:

```jai
count, error := Pg.copy_out(db, "copy (select email from users) to stdout (format csv)", write_piece, user_data);
copied, error := Pg.copy_out(db, "copy users (email, score) to stdout (format binary)", User_Row, .["email", "score"]);
for copied.rows print("% %\n", it.email, it.score);
Pg.free_copy_rows(*copied);
```

The callback form hands each piece to you as it arrives, in any format. The row form decodes a binary COPY into `Copy_Rows(Row)`; `free_copy_rows` frees everything. Both also take a `*Pool`.

## How it works

- `copy_in` feeds `run_script` from a `Script_Hooks.copy_in` hook (`copy.jai`): header, rows in 64 KiB pieces, then the trailer. Each row is a field count plus a length and binary form per member (`encode_binary`, `encode_array`, `numeric_binary` in `codecs.jai`).
- `columns` picks members by name in the statement's column order. Empty means every member.
- `copy_many` builds its statement at compile time from `@pg_table` and the fields without `@pg_default`/`@pg_generated`.
- `copy_out` decodes rows with the same decoders as `get`. A NULL needs a `Nullable` member; a wrong width, text format or count mismatch is an error.
- A row that cannot be encoded aborts the COPY; the error names the row and member.

## How to change it

A new type goes in `encode_binary` (parameters get it too), or in `put_copy_value` if it only makes sense in COPY.

Gotchas:

- Binary COPY casts nothing. Member types must match columns exactly: `s32` for `int4`, `s64` for `int8`, `[] string` for `text[]` (not `varchar[]`). Generated structs always match.
- A `Json(T)` member goes as `jsonb` unless it has `@pg_json`, which `generate` writes on `json` columns.
- Arrays of enums are refused.
- A `Nullable(T)` zero value is not null; set `.null = true`.

## Dependencies

`query.jai` (`run_script`, `Script_Hooks`), `codecs.jai`, `orm.jai` (`copy_many`). Tests: `test_copy_out` in `tests/server.jai` and `test_copy` in `tests/orm/main.jai`.
