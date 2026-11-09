# Json

`to_json`, `from_json` and `free_json`, modelled on serde_json. Encoders and decoders for your types are generated at compile time from `Type_Info`. Builds for wasm. Shared rules: [README](README.md#conventions-of-io-tls-http-and-json).

```jai
User :: struct @json(rename_all=camelCase) {
    id: int;
    display_name: string;                       // "displayName"
    email: string;  @json(skip_serializing_if=empty,default)
    tags: [..] string;
}

text := to_json(user);                          // pretty = true indents
user, error := from_json(User, text);           // error.kind == .NONE on success
value, error2 := from_json(Json_Value, text);   // untyped tree
free_json(*user);                               // values, Json_Value, Json_Error
```

## How it works

- `from_json($T, text, options)` returns `T, Json_Error` (`kind`, `message`, `offset`, `line`, `column`, `path` like `tags[2]`). `json_path_text(error)` renders jsonpath (`$.tags[2]`). `Json_Options` has `max_depth` (256) and `max_size` (1 GiB) plus the options below. `write_json(*builder, value)` encodes into a builder.
- `Json_Value` is the untyped tree: build with `json_null/bool/int/...`, `json_push`, `json_set`, take members out with `json_remove` (every member of that key, order kept); read with `json_get`, `json_at`, `json_count`, `json_as_*`.
- `from_json($T, *tree, options)` decodes a parsed `Json_Value` (the whole document or one member from `json_get`) with the same notes, options, errors and paths as text, without writing it out first; errors report offset 0, line 1, column 1. `to_json_value(value)` is the other way: it returns the tree that parsing `to_json` would give (invalid UTF-8 becomes U+FFFD as in text), to edit or store with `json_set`. Both use the generated code: `Json_Writer` has a tree mode (`tree`, `root`, `frames`) in which the `json_write_*`, `json_begin_*`, `json_key*` procedures build nodes, and `Json_Reader` has one (`tree`, `cur`, `frames`; `frames` is a `Tree_Stack` that holds the first 16 levels inline and spills to the heap only for deeper documents, so a typical decode allocates nothing for it) in which `json_peek`, `json_begin_*`, `json_*_next`, `scan_string` and `scan_number` read the nodes, so `with=` handlers and variants work unchanged. Variants rewind through `Json_Mark`, which saves the tree position too. The tree must outlive a `borrow` result and must not change while decoding.
- `Json_Source { data, fill }` gives a document in pieces; Json gathers it in one buffer before decoding. `Io.read_json` and the HTTP client's `read_json` build one (Json doesn't import Io).
- Integers are exact over their range (`.OUT_OF_RANGE` otherwise). Floats print as the shortest round-trip string and decode correctly rounded; NaN and infinity encode as `null`. Supported: strings, bools, enums (by name), arrays, pointers, nested and recursive structs, `Hash_Table` with string keys, `Json_Value`, `Variant`. No 128-bit integers.
- Parsing is strict RFC 8259 (no BOM, trailing commas, lone surrogates, bad UTF-8). A failed decode frees what it allocated and returns a default `T`.
- A decoder written by hand drives a `Json_Reader` directly: `json_reader_init(*r, text)`, the `json_read_*` and `json_begin_*` procedures, `json_read_number` for a number's checked text, then `json_reader_error` on failure and `json_reader_free`. [Protobuf](protobuf.md)'s JSON mapping reads this way.
- `free_json` is generated per type (`generate_freer`). It skips fields still holding their declared default and resets what it frees, so a second call is harmless. Never call it on values that weren't decoded.

| `@json(...)` | Where | Effect |
| --- | --- | --- |
| `rename=x`, `alias=y`, `rename_all=RULE` | member / struct | key name; extra accepted key; case rule (camelCase, snake_case, kebab-case, ...) |
| `skip`, `skip_serializing`, `skip_deserializing` | member | omit in one or both directions |
| `skip_serializing_if=empty\|zero\|null` | member | omit when empty, zero, null |
| `default` | member / struct | a missing field keeps its initializer |
| `flatten` | member | inline a struct member's fields |
| `with=name` | member | `T.name_encode(w, *field)` / `T.name_decode(r, *field)` declared inside the struct |
| `as_number` | enum member | encode as the number |
| `tag=k`, `content=c`, `untagged`, `tags=a,b` | Variant member | how a [Variant](variant.md) is tagged |
| `deny_unknown_fields`, `transparent` | struct | unknown keys are errors; one-field struct encodes as its field |

Notes go after the member's `;` (struct-level before `{`). Mistakes are compile errors. A value may be a quoted JSON string when it holds commas or quotes: `@json(rename="a,b")`. `\\`, `\"`, `\n`, `\r`, `\t` and `\uXXXX` are the escapes it reads.

Runtime `Json_Options`:

| Option | Default | Effect |
| --- | --- | --- |
| `deny_unknown_fields` | `false` | every struct denies unknown keys |
| `missing_fields_default` | `false` | a missing field keeps its initializer instead of `.MISSING_FIELD` |
| `duplicate_keys` | `.ERROR` | `.LAST` keeps the last value of a repeated key |
| `borrow` | `false` | strings without escapes (and `Json_Value` keys and numbers) are slices of the input |

`borrow` results point into the input, which must outlive them, and must not go through `free_json`: decode into an arena or `temp` (`from_json(T, text, .{ borrow = true },, temp)`) and reset that.

`Json_Maybe(T)` (`state` of `.MISSING`, `.NULL`, `.SET`, and `value`) tells a missing key, a `null` and a value apart. Encoding omits a `.MISSING` member.

### Variants

A [`Variant`](variant.md) is encoded by its live case, tagged with the case's field name (`circle`, or `s64` for the `s64_` case) run through `rename_all`. A `void` case (no payload) is `null` when the variant is untagged and the tag alone otherwise: `"none"`, `{"kind":"none"}`, `{"t":"none"}`. `null` decodes to the `void` case in every form; a variant without one rejects `null`. A variant that is not set holds its first case, so a missing field decodes as that.

| Form | Note | Text for `Circle{ radius = 1 }` |
| --- | --- | --- |
| externally tagged (default) | none | `{"circle":{"radius":1}}` |
| internally tagged | `@json(tag=kind)`, every case a struct | `{"kind":"circle","radius":1}` |
| adjacently tagged | `@json(tag=t,content=c)` | `{"t":"circle","c":{"radius":1}}` |
| untagged | `@json(untagged)`; default when every case is a scalar or enum of a different type (or `void`) | `{"radius":1}` |

`@json(tags=circle,rect)` names the tags in case order. Decoding finds an internal tag anywhere in the object. An unknown tag is `.UNKNOWN_VARIANT` listing the choices; a missing tag is `.MISSING_FIELD`. An untagged variant tries each case in order and the first that decodes wins, so two cases of one type must be tagged. Variants may hold pointers to themselves. Bad notes are compile errors.

### Query and form text

`from_query($T, pairs)` and `from_param($T, text)` (`query.jai`) decode URL query/form pairs and single strings by the field types, with the same notes and errors. A repeated key fills an array field. The HTTP server's `Query`, `Form`, `Path` and `Params` use them.

```jai
Search :: struct @json(default) { q: string; limit := 20; tags: [..] string; }
search, error := from_query(Search, pairs);        // "?q=jai&tags=a&tags=b"
page, error2 := from_param(int, "42");
```

### Speed

The generated code is straight-line per struct and field, with no runtime type switch; keys match by length then word compares against constants. Float printing uses Schubfach, parsing uses Eisel-Lemire. Vector scans (`Extensions/SIMD`) speed escaping and string scanning; for texts of at least `json_index_min_size` (256 bytes) a simdjson-style stage 1 index finds string extents, with any anomaly falling back to the scalar reader so errors are identical on every path. Benchmarks: `jacket bench --path stdlib/Extensions/Json`. Fetch real-world inputs first with `PYTHONPATH=tools python3 tools/dev/json_bench_data.py` (also part of `tools/portable/build_portable.py data`). Each generated/real document, decode/encode operation and scalar/vector/indexed mode has its own case.

## How to change it

- New runtime option: add the field to `Json_Options`, copy it onto `Json_Reader` in `reader_init`, and test `r.<field>` in the code `generate_decoder` emits.
- New wrapper type (like `Json_Maybe` or `Variant`): an `is_*` helper plus cases in `emit_encode`, `emit_decode`, `emit_free`, and the top-level checks in the generators. Variant code is in `variant.jai`.
- New attribute: parse in `collect_leaves` (`generate.jai`), carry on `Leaf`, emit in the encoder/decoder generators. A new owning type needs `emit_free` or failed decodes leak (`tests/leaks.jai`).
- Query field kinds need a `Pair_Kind` and a case in `generate_pair_kinds`.
- Writer: a reserved window is valid only until the next append; write at most what you reserved, then `builder_commit`. `tests/encode.jai` pins output text exactly.
- Vector paths stay behind `simd_enabled()` and must match scalar results (`tests/differential.jai`). `pow5.jai`/`pow10.jai` are generated; their headers say how.
- JSONTestSuite fixtures must stay byte-exact; `tools/corpus/gen_json_corpus.py` regenerates `tests/fixtures/suite_cases.jai`.

Other knobs: `json_use_simd` (true), `json_index_min_size`, `json_index_stats`.

## Compared with the community libraries

`benchmarks/compare/json/benches/json.jai` (`tools/bench/library_compare.py json`, see [library comparisons](../tools/library-comparisons.md)) decodes and encodes the same text and types with Jacket, jaison and Contiguous on three generated documents: a 69 KB API page, 20,000 telemetry records, and a configuration nested 100 deep (Apple M5). Time relative to Jacket (above 1 is slower):

| Workload | jaison | Contiguous |
| --- | --- | --- |
| typed decode | 2.5x-3.7x | 1.1x-1.2x |
| untyped tree decode | 1.7x-2.2x | 1.2x-1.3x (75x on the deep config) |
| encode | 2.9x-3.4x | 1.7x-2.3x |

jaison walks `Type_Info` on every call. Contiguous is close on typed decode because it doesn't unescape strings and checks input only with `assert`. `borrow` makes Jacket another 3-5% faster on typed decodes and 13-29% on untyped trees.

Beyond speed: Jacket's parser follows RFC 8259 with positioned errors, handles surrogate pairs, tells absent from null, has tagged unions, maps, `flatten`, depth limits and query-string decoding. Neither community library has these.

`json_benches` emits one selectable procedure for each input, type, scan mode and operation. Extend that matrix when adding a workload; keep input generation, file reads and pre-decoding before `reset_timer`.
