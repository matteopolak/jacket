# Typed TOML

`from_toml(T, text)` reads a TOML document straight into a struct and `to_toml(value)` writes one back. No document is built: values land in the target as they are parsed. It is the fast path for configuration files; use [`parse_toml`](toml.md) when the shape is not known up front or you need spans and statement history.

```jai
Server :: struct {
    host: string;
    port: int; @toml(default)
    tags: [..] string; @toml(default)
}

server, error := from_toml(Server, "host = 'a'\nport = 80\n");
assert(error.kind == .NONE && server.port == 80);
text, _ := to_toml(server);   // host = "a"\nport = 80\ntags = []\n
free(text);
free_toml_value(*server);
```

## How it works

Decoding reuses the document parser's reader: the same string, number and date grammar, UTF-8 check, error spans and `Toml_Options` limits. A scalar is read into one scratch node and stored at once. What replaces the document is `Toml_Frame` (`decode.jai`): one record per table the input mentions, holding how each key was introduced (implicit parent, `[header]`, dotted key, assigned value, `[[array]]`). Those records are all TOML's rules about duplicate keys and table redefinition need, so `a = 1` then `a = 2`, `[a]` twice, `a.b = 1` then `[a]`, or extending an inline table are rejected as `.DUPLICATE` with the earlier declaration in `related`.

`generate.jai` writes, per struct type at compile time, three procedures that `module.jai` inserts: `toml_descend` (key dispatch and storage), `toml_finish` (required keys, run when a table can no longer change) and `toml_free_fields`. A `Table(string, V)` gets the same three, keyed by a frame record per key (the table gets an entry when its key is first declared), and a `Toml_Value` is descended by `toml_descend_value` in `decode.jai`, which reads assigned values with the document parser into a scratch document and copies the result out. Keys the type has no field for are validated and ignored, tracked by name in the frame, so unknown data still obeys the rules (frames with more than eight unknown keys add a hash index, so wide tables stay linear); a field marked `@toml(unknown)` collects them.

Encoding writes key/value lines first, then `[table]` and `[[array of tables]]` headers for struct fields (TOML requires that order), recursing with the dotted path. Structs inside arrays of non-struct parents are written as inline tables. An empty array of tables is written as `key = []` so the key survives a round trip. Null pointers are omitted. Floats use the shortest round-trip spelling.

## Supported types

Integers (range-checked), floats (integers are accepted), `bool`, `string`, enums (by name; `as_number` for numbers), `Toml_Datetime`, structs, pointers (absent keys stay null), `[..] T`, `[] T` and `[N] T`. Arrays of tables (`[[x]]`) go into a `[..]`, `[]` or `[N]` of structs; a `[]` view is grown in place, so it must start empty, and a `[N]` needs exactly `N` headers (or an inline array of `N` elements). Unions are not supported.

`Table(string, V)` (the `Hash_Table`) takes a table: its keys come from inline tables, dotted keys and `[a.key]` headers, and `V` is anything else on this list, including structs, arrays of tables (`[[a.key]]`) and other tables. Keys are copied and freed with the table. `@toml(...)` options on the field apply to the table, not to its values; the values' own struct notes still apply.

`Toml_Value` takes any value as the document parser would build it, whatever shape the data has: `extra = { a = 1, b = [1, 'x'] }`, or a table made of `[extra.sub]` and `[[extra.list]]` headers. It is an untyped tree that the target owns (nothing points into the input), so `toml_get`, `toml_get_path` and `write_toml` work on it, and `free_toml_value` releases it. `from_toml(Toml_Value, text)` and `from_toml(Table(string, V), text)` also work as a whole document. `[..] Toml_Value` and `*Toml_Value` read inline arrays and values only, not `[[x]]` headers.

## Notes

`@toml(...)` follows the member (`port: int; @toml(default)`) or the struct (`struct @toml(default) {`).

| Option | Where | Effect |
| --- | --- | --- |
| `rename=key`, `alias=key` | field | Key in the file; extra accepted keys |
| `rename_all=rule` | struct, enum field | `lowercase`, `UPPERCASE`, `camelCase`, `PascalCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, `kebab-case`, `SCREAMING-KEBAB-CASE`. Enum names are `snake_case` unless a rule says otherwise |
| `default` | field, struct | Not required; the declared initializer stays |
| `skip`, `skip_serializing`, `skip_deserializing` | field | Ignore in both or one direction |
| `skip_serializing_if=empty\|zero\|null` | field | Omit when true |
| `flatten` | struct field | The field's keys appear in the parent table |
| `range=lo..hi` | integer field | Narrower bounds than the type |
| `message="..."` | field | Static text for the field's type and range errors |
| `as_number` | enum field | Read and write the integer value |
| `unknown` | `[..] Toml_Unknown` field | Collects ignored keys: dotted `path`, the statement `start`/`end`, the assignment's `value` offset and `header` |
| `deny_unknown_fields` | struct | Unknown keys are `.TYPE` errors; `Toml_Options.deny_unknown_fields` does it for every struct |

A field is required unless it has `default` (or its struct does). A missing key is a `.TYPE` error whose path names it. Error paths include the element of an array of tables: `$["items"][1]["name"]`. Type and range errors point at the value; duplicates at the second declaration.

## Ownership

`from_toml` allocates strings, arrays and pointers from `context.allocator`. `free_toml_value(*value)` releases them and leaves fields that still hold their declared default alone. A failed decode frees everything it allocated and returns a default value. `Toml_Unknown.path` is owned too and is freed by the same call.

## Changing it

- A new option: parse it in `read_leaf` (`generate.jai`), then use it in `emit_read`, `generate_descend`, or the encoders.
- A new field type: add a `toml_read_*` helper in `decode.jai`, a case in `emit_read`/`emit_write`/`emit_free`. A type that is a table of its own (like `Table`) also needs a branch in `generate_descend`, `generate_finish` and the encoders; `emit_step` is the shared part that decides what a key does once it reached a place.
- The declaration rules live in `toml_enter`; the parser's `toml_parent`/`toml_header` in `reader.jai` states the same rules for the document, so change both together.
- Limits count what is read: `max_nodes` counts values and tables, `max_depth` nested arrays and inline tables, `max_retained_bytes` the decoded strings. Writing counts the root, every value and every array against `max_nodes`, and the depth of every value against `max_depth` (the root table is depth zero, each value is one deeper than its table or array, and an array of tables adds a level for the array and one for its elements), exactly as `write_toml` does; `tests/writer-depth.jai` compares the two at every limit. A `float32` is written with its own shortest digits (`0.1`, not `0.10000000149011612`).

## Configuration and dependencies

`Toml_Options` (see [TOML](toml.md)) plus `deny_unknown_fields`. Needs only `Basic`, `String` and `Compiler`. Tests: `stdlib/Extensions/Toml/tests/typed.jai` (also `typed-shapes.jai`, `typed-maps.jai`, `error-paths.jai`, `unknown-wide.jai`, `writer-limits.jai`), which also checks that decoding, success or failure, leaves no allocation behind.
