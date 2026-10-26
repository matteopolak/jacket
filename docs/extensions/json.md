# Json (`Extensions/Json`)

Shared rules: [README](README.md#conventions-of-io-tls-http-and-json).

## What it is

`to_json`, `from_json` and `free_json`, modelled on serde_json. Encoders and decoders for your types are generated at compile time from `Type_Info` (`#insert` in polymorphic procedures, as in Regex and Args). Builds for wasm.

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
free_json(*user);                               // one overload set: values, Json_Value, Json_Error
```

## How it works

- `from_json($T, text, options)` returns `T, Json_Error` (`kind`, `message`, `offset`, `line`, `column`, `path` such as `tags[2]`). `json_path_text(error)` renders the path as jsonpath (`$.tags[2]`; keys that are not plain identifiers quoted, `$."we ird".a`; `$` for the document). `Json_Options`: `max_depth` 256, `max_size` 1 GiB (`.LIMIT`), plus the strictness and ownership options below. `write_json(*builder, value)` encodes into a builder. `Json_Value` is built with `json_null/bool/int/...`, `json_push`, `json_set`, read with `json_get`, `json_at`, `json_count`, `json_as_*`.
- **From a stream.** `Json_Source { data, fill }` gives the document in pieces; `from_json(T, source)` gathers it in one buffer then decodes, so any cut gives the same value or error. Json does not import Io; `Io.read_json(reader, T)` and the HTTP client's `read_json` build a source.
- **Types.** Integers are exact over their range (`.OUT_OF_RANGE` otherwise). Floats print as the shortest round-trip string; NaN/infinity encode as `null`; decoding is correctly rounded (direct conversion for up to 19 digits, Eisel-Lemire, else Basic's conversion; `1e999999` is `.OUT_OF_RANGE`). Supported: strings, bools, enums (by name), arrays, pointers, nested/recursive structs, `Hash_Table` with string keys, `Json_Value`, `Variant` (below). Not 128-bit integers.
- **Parsing** is strict RFC 8259 (no BOM, trailing commas, lone surrogates, bad UTF-8). A failed decode frees everything it allocated and returns a default `T`. Unknown fields are ignored unless denied; a missing field without `default` is an error; duplicate keys are an error (see the options below).
- **Freeing.** `free_json(*value)` calls `free_json_fields(value, defaults)`, generated per type by `generate_freer`. It skips fields still holding their declared default (a string literal would crash `free`) and resets what it frees, so a second call is harmless. Fields with a `with=` handler belong to the handler. Never call it on values that were not decoded.

| `@json(...)` option | Where | Effect |
| --- | --- | --- |
| `rename=x`, `alias=y`, `rename_all=RULE` | member / struct | key name; extra accepted key; case rule (camelCase, snake_case, kebab-case, ...) |
| `skip`, `skip_serializing`, `skip_deserializing` | member | omit in one or both directions |
| `skip_serializing_if=empty\|zero\|null` | member | omit when empty, zero, null |
| `default` | member / struct | a missing field keeps its initializer |
| `flatten` | member | inline a struct member's fields |
| `with=name` | member | `T.name_encode(w, *field)` / `T.name_decode(r, *field)` declared inside the struct (generated code cannot see file-scope names) |
| `as_number` | enum member | encode as the number |
| `tag=k`, `content=c`, `untagged`, `tags=a,b` | Variant member | how a [Variant](variant.md) is tagged; see Variants |
| `deny_unknown_fields`, `transparent` | struct | unknown keys are errors; one-field struct encodes as its field |

Runtime options on `Json_Options`, tested by the generated decoders through the reader (`r.deny_unknown`, `r.missing_default`, `r.duplicate_last`, `r.borrow`):

| Option | Default | Effect |
| --- | --- | --- |
| `deny_unknown_fields` | `false` | every struct denies unknown keys; a struct's own note still denies when `false` |
| `missing_fields_default` | `false` | a missing non-optional field keeps its declared initializer instead of `.MISSING_FIELD` |
| `duplicate_keys` | `.ERROR` | `.LAST`: a repeated key in a struct releases the earlier value and decodes the new one (as Postgres `jsonb` keeps); a `Hash_Table` always replaces |
| `borrow` | `false` | strings without escapes (and a `Json_Value`'s keys, text and numbers, and table keys) are slices of the input; strings with escapes are allocated |

`borrow` results point into the input, which must outlive them, and must not go through `free_json` (nor may a failed decode free through a heap allocator): decode with `context.allocator` set to an arena or `temp` (`from_json(T, text, .{ borrow = true },, temp)`) and reset that instead. The streaming `from_json` and `from_query`/`from_param` ignore it, since their text does not outlive the call.

`Json_Maybe(T)` (`state: Json_Presence` of `.MISSING`, `.NULL`, `.SET`, and `value: T`) tells a missing key, a `null` and a value apart. As a member a missing key leaves `.MISSING` (never `.MISSING_FIELD`); in the document or an array `null` gives `.NULL`. Encoding omits a `.MISSING` member (a `.MISSING` array element or document writes `null`). `free_json` frees `value` when `.SET`.

Written as a note after the member's `;` (struct-level before `{`). Mistakes are compile errors.

### Variants

A [`Variant`](variant.md) (a tagged union whose cases are types) is encoded by its live case. A case is named by its type's name (`Circle`, `s64`), run through the struct's `rename_all` (or the member's own `rename_all=`).

| Form | Note on the member | Text for `Circle{ radius = 1 }` |
| --- | --- | --- |
| externally tagged (default) | none | `{"Circle":{"radius":1}}` |
| internally tagged | `@json(tag=kind)`, every case a struct | `{"kind":"Circle","radius":1}` |
| adjacently tagged | `@json(tag=t,content=c)` | `{"t":"Circle","c":{"radius":1}}` |
| untagged | `@json(untagged)`; the default when every case is an integer, float, bool, string or enum | `{"radius":1}` or the bare scalar |

`@json(tags=circle,rect)` names the tags in case order (the count must match; no spaces, as in every note). A note on an array, `Json_Maybe`, pointer or `Hash_Table` member reaches the variants inside. A top-level `to_json(variant)`/`from_json(Shape, text)` uses the default form. A variant may hold pointers to itself (`Tree :: Variant(Leaf, Branch)` with `Branch :: struct { children: [..] *Tree; }`), and nests as JSON does. A payload-less case is an empty struct (`Empty :: struct {}`), written `{}`.

Decoding reads the tag from anywhere in an internally or adjacently tagged object (it looks ahead, then decodes), ignores the tag key in the case's fields, and takes the content key once (a repeat is `.DUPLICATE_FIELD`, or the last wins with `duplicate_keys = .LAST`). An unknown tag is `.UNKNOWN_VARIANT` with the tag and the choices in the message (`unknown variant `Oval`, expected one of: Circle, Rect`) and the path of the variant; a missing tag key is `.MISSING_FIELD`. An untagged variant tries each case in order on the same text: the first that decodes wins, and when none does the `.TYPE` error lists each case's failure briefly. A failed attempt frees what it allocated, so nothing leaks. `free_json` frees the live case and zeroes the variant, except one that still holds its declared default. Bad notes (`tag=` with a case that is not a struct, `tags=` of the wrong count or with repeats, `content=` without `tag=`, a case with a field named like the tag, a note on a field without a variant) are compile errors.

### Query and form text

`from_query($T, pairs)` and `from_param($T, text)` (`query.jai`) decode URL query/form pairs and single strings (path segment, header) by the type of the field each key names, with the same `@json` notes and errors as `from_json` (the pairs become the equivalent JSON object). A repeated key fills an array field and is a "duplicate field" error otherwise. The HTTP server's `Query`, `Form`, `Path` and `Params` use them.

```jai
Search :: struct @json(default) { q: string; limit := 20; tags: [..] string; }
search, error := from_query(Search, pairs);        // "?q=jai&tags=a&tags=b"
page, error2 := from_param(int, "42");
```

### Specialization and SIMD

Straight-line code per struct and field, no runtime type switch. Keys match by length test then 8/4/2/1-byte word compares against compile-time constants; compact output appends prebuilt `"key":` constants. Constant documents can be decoded with `#run` and baked in. Float printing uses Schubfach (`number.jai`, table in `pow10.jai`); float parsing uses `pow5.jai`. Vector scans (`Extensions/SIMD`, `u8x16`) accelerate escaping and string scanning; for texts of at least `json_index_min_size` (256) bytes a simdjson-style stage 1 index (`index.jai`) finds string extents, abandoned when entries are too dense or sparse, and any anomaly re-parses with the scalar reader so errors are identical on every path. A full structural index for typed decoding was tried and dropped: the schema-guided single pass is as fast. Benchmarks: `stdlib/Extensions/Json/bench/bench.jai` (`-O2`); `tools/dev/json_bench_data.py` fetches the large documents. The vector scans are worth about 1.0 to 1.1x; the parser is bound by value construction, allocation and number conversion.

## How to change it

- **New runtime option:** add the field to `Json_Options`, copy it onto `Json_Reader` in `reader_init`, and test `r.<field>` in the code `generate_decoder` emits (keep it to one branch). **New wrapper type** (like `Json_Maybe`): recognise it with an `is_*` helper and add cases in `emit_encode`, `emit_decode` and `emit_free`, and in the top-level checks of `generate_encoder`/`generate_decoder`/`generate_freer`. `Variant` is one: `variant.jai` has its `emit_*_variant` generators, the tag notes (`Enum_Style` carries them down to the variant), and the reader helpers `json_mark`/`json_rewind`/`json_find_tag`/`json_note_attempt`. An internally tagged case is coded by `json_encode_tagged`/`json_decode_tagged`, the struct coders with the tag key written first or skipped; an untagged attempt is a local procedure per case so that the decoders' `return false` can leave it.
- **New attribute:** parse in `collect_leaves` (`generate.jai`), carry on `Leaf`, emit in `generate_encoder`/`generate_decoder`. **New type:** cases in `emit_encode` and `emit_decode`; a new owning type also needs `emit_free` or failed decodes leak (`tests/leaks.jai`).
- `query.jai` field kinds need a `Pair_Kind` and a case in `generate_pair_kinds`. Runtime helpers are in `reader.jai`/`writer.jai`; stage 1 in `index.jai`.
- Writer pieces (`put`, `builder_reserve`, `copy_bytes`): a reserved window is valid only until the next append; write at most what you reserved, then `builder_commit`. `tests/encode.jai` pins output text exactly.
- Vector paths stay behind `simd_enabled()` and must match scalar results (`tests/differential.jai`). Tables in `pow5.jai`/`pow10.jai` are generated; headers say how. `tests/numbers.jai` sweeps are small by default: set `FULL_SWEEP` for the large run.
- JSONTestSuite (`tests/fixtures/JSONTestSuite/`, MIT) must stay byte-exact; `tools/corpus/gen_json_corpus.py` regenerates `tests/fixtures/suite_cases.jai`; per-case choices for `i_` cases are in `tests/suite.jai`.

## Configuration

`json_use_simd` (true), `json_index_min_size` (256), `Json_Options` (`max_depth`, `max_size`, `deny_unknown_fields`, `missing_fields_default`, `duplicate_keys`, `borrow`), `json_index_stats` (counters).

## Dependencies

`Basic`, `String`, `Compiler`, `Hash_Table`, `Extensions/SIMD`, `Extensions/Variant`. Nothing from Io (`Io.read_json` imports Json). The structural-index design follows simdjson (Langdale and Lemire); the code is written for this repo.

## Compared with the community libraries

Jacket's Json next to [`rluba--jaison`](https://github.com/rluba/jaison) and [`kujukuju--ContiguousJsonJai`](https://github.com/kujukuju/ContiguousJsonJai) (called Contiguous below), the two JSON libraries in the upstream corpus. The cells come from reading each library's source; ⚠ has a note, and a library not named in a row has nothing for it.

| Feature | Jacket Json | jaison | Contiguous |
| --- | --- | --- | --- |
| How the code is made | generated per type at compile time (`#insert`), straight-line code per struct | one procedure that walks `Type_Info` on every call (`json_write_native`, `parse_value`) | generated per type at compile time (`#insert`), values stored as offsets into one block |
| Typed decode into structs and arrays | ✅ | ✅ | ✅ |
| Untyped tree | ✅ `Json_Value` | ✅ `JSON_Value` | ✅ `JsonGeneric` |
| Typed field that holds an untyped subtree | ✅ | ✅ | ✅ |
| Encode | ✅ | ✅ | ⚠ [1] |
| Pretty printing | ✅ | ✅ | ✅ |
| Parser follows RFC 8259 | ✅ | ⚠ [2] | ❌ [3] |
| Error with position and path | ✅ line, column, byte offset, jsonpath | ❌ [4] | ❌ [3] |
| Escapes decoded (`\n`, `é`, surrogate pairs) | ✅ | ⚠ [5] | ❌ [6] |
| Output is valid UTF-8 JSON for any string | ✅ | ⚠ [7] | ❌ [1] |
| Integers exact over their range, out of range is an error | ✅ | ✅ | ⚠ [8] |
| Floats print as the shortest string that reads back, parse correctly rounded | ✅ | ⚠ [9] | ❌ [10] |
| NaN and infinity | ✅ `null` | ✅ `null` or strings, by option | ❌ |
| Rename fields | ✅ `rename`, `alias`, `rename_all` | ⚠ [11] | ⚠ [11] |
| Skip fields (either direction, if empty or zero) | ✅ | ⚠ [12] | ⚠ [12] |
| Missing field is an error unless the field has `default` | ✅ | ❌ [13] | ❌ [13] |
| Unknown field: ignore or deny | ✅ both | ✅ both | ⚠ [14] |
| Duplicate key: error or last wins | ✅ both | ❌ [15] | ❌ |
| `null`, absent and value told apart | ✅ `Json_Maybe` | ❌ | ❌ |
| Enums | ✅ by name | ✅ by name, flags as `A\|B` | ⚠ [16] |
| Tagged unions | ✅ four layouts (`Variant`) | ❌ | ❌ |
| Maps | ✅ `Hash_Table` with string keys | ❌ | ❌ |
| Flatten and custom coders | ✅ `flatten`, `transparent`, `with=` | ⚠ [17] | ❌ |
| Recursive types | ✅ | ✅ through pointers | ❌ [18] |
| Limit on nesting depth | ✅ `max_depth` 256 | ❌ | ❌ |
| Decode from a stream | ⚠ [19] | ❌ | ❌ |
| Free everything with one call | ⚠ [20] | ❌ | ✅ [21] |
| Query-string and form decoding by field type | ✅ | ❌ | ❌ |
| Builds for wasm | ✅ | not checked | not checked |

1. Contiguous writes a string between quotes as it is (`write_output` for `string` appends the bytes), so a quote, a backslash or a newline in a value makes invalid JSON. Floats are printed with `formatFloat(width = 16)`. Output is compact unless `$formatted`.
2. Literals, strings, arrays and objects are checked, and a trailing comma is refused. Anything else goes to Basic's `string_to_float64` or `string_to_int`, so the number grammar is Basic's, and a document is not checked for a BOM or invalid UTF-8.
3. The reader checks the structure with `assert`; with the module parameter `ENABLE_ASSERT = false` it checks nothing, and with it on a bad document stops the program. Failures that do not assert return only `false`.
4. A failed parse returns `false` and writes a `log_error` line without a position (for example "Cannot parse STRING value into type ..."); a few cases `assert`.
5. `\uXXXX` is decoded to one to three bytes from a 16-bit value, so a surrogate pair (`😀`) becomes two three-byte sequences, not one four-byte character.
6. The reader has no unescape step; the README says Unicode is not supported. A string with `\n` keeps the backslash and the `n`.
7. It escapes `"`, `\`, `\n`, `\r`, `\t` and other control characters, and leaves bytes of 0x80 and above as they are; a source comment marks invalid multi-byte characters as to do.
8. Digits are collected and converted by Basic's `string_to_int` into the field's type, without a message for an out-of-range value.
9. Printing goes through Basic's `print`; parsing through `string_to_float64`. Neither is documented as shortest or correctly rounded.
10. The README says results are within "an extra digit or two" of JavaScript's.
11. Per member only: `@JsonName(x)` (jaison) and `@json(x)` (Contiguous). There is no rule for a whole struct.
12. `@JsonIgnore` (jaison) and `@jsonignore` (Contiguous) skip a member in both directions; Contiguous can leave out `null` members when writing (`$write_null_members`).
13. jaison keeps a member at its default when the key is absent; Contiguous leaves it as it was.
14. Contiguous always skips an unknown key (`skip_unknown_type`); there is no option to refuse it.
15. A repeated key decodes twice into the same member and the second wins; the first string is not freed.
16. Contiguous reads and writes an enum as its integer type.
17. jaison flattens `using` members; there is nothing for custom coders.
18. A struct with a pointer to itself (the `Node` of the benchmark) did not compile with Contiguous in these benchmarks, so its configuration workloads are blank.
19. `from_json(T, source)` takes the document in pieces but gathers it in one buffer before decoding.
20. Free procedures per type (`free_json`) and a `borrow` option that points strings into the input, so a decode in an arena frees with the arena.
21. Everything a decode allocates is one block; the README calls freeing it one call.

### How it was measured

`benchmarks/compare/json/benches/json.jai`, run by `tools/bench/library_compare.py json` (see [library comparisons](../tools/library-comparisons.md)). Every library decodes the same text into the same Jai types and encode the same values. Documents are generated with a fixed seed:

- **api**: a page of 100 issues as a service returns them (minified, about 69 KB): nested users and labels, an optional assignee, escape sequences and non-ASCII text in the bodies.
- **records**: 20,000 telemetry records (about 2 MB, minified): numbers, short strings, an array of four integers.
- **config**: a hand-written style configuration nested 100 levels deep, pretty-printed.

`decode_*` is the document into typed values (`decode_tree_*`: into each library's untyped tree), `encode_*` writes values decoded before the timer started, `roundtrip_api` is decode then encode. The `borrow` column is Jacket with `Json_Options.borrow` (strings without escapes are slices of the input; strings with escapes are still unescaped into new memory). Each iteration allocates from a bump pool that is emptied afterwards, for every library, because jaison has no free procedure for a typed decode and the others free in different ways. Inputs and results go through `black_box`.

Apple M5 (10 cores: 4 Super and 6 Efficiency), 16 GB, macOS 27.0 arm64. The machine was a working desktop, not an idle one: the load average when the run began was 4.13, so absolute times carry a few percent of noise (the intervals show the part that was measured within one run) and the ratios are the result to read.

Time per iteration, with the 95% interval of the bootstrap in brackets, and the ratio to Jacket (above 1: slower than Jacket).

| Workload | jacket | borrow | jaison | contiguous |
| --- | --- | --- | --- | --- |
| decode_api | 153 us (152 us to 155 us) | 139 us (138 us to 140 us) 0.91x | 364 us (363 us to 364 us) 2.37x | 122 us (122 us to 122 us) 0.80x |
| decode_records | 6.04 ms (6.03 ms to 6.05 ms) | 5.79 ms (5.77 ms to 5.8 ms) 0.96x | 16.5 ms (16.4 ms to 16.6 ms) 2.73x | 6.46 ms (6.44 ms to 6.47 ms) 1.07x |
| decode_config | 74.3 us (73.6 us to 75.3 us) | 71.4 us (71.4 us to 71.5 us) 0.96x | 128 us (128 us to 129 us) 1.73x | n/a |
| decode_tree_api | 226 us (226 us to 226 us) | 166 us (166 us to 167 us) 0.74x | 335 us (334 us to 335 us) 1.48x | 240 us (240 us to 240 us) 1.06x |
| decode_tree_records | 9.88 ms (9.87 ms to 9.89 ms) | 6.9 ms (6.89 ms to 6.92 ms) 0.70x | 15.9 ms (15.9 ms to 15.9 ms) 1.61x | 11.9 ms (11.9 ms to 12 ms) 1.21x |
| encode_api | 38.2 us (38.1 us to 38.3 us) | n/a | 118 us (117 us to 118 us) 3.08x | 59.3 us (59.2 us to 59.5 us) 1.55x |
| encode_records | 2.94 ms (2.92 ms to 2.96 ms) | n/a | 9.92 ms (9.82 ms to 10 ms) 3.38x | 6.21 ms (6.2 ms to 6.23 ms) 2.11x |
| encode_config | 8.65 us (8.61 us to 8.69 us) | n/a | 24.1 us (24 us to 24.2 us) 2.78x | n/a |
| roundtrip_api | 188 us (188 us to 189 us) | 178 us (178 us to 178 us) 0.95x | 479 us (478 us to 479 us) 2.54x | 193 us (191 us to 196 us) 1.03x |
| decode_tree_config | 85.7 us (85.5 us to 86 us) | 75.2 us (75.1 us to 75.3 us) 0.88x | 120 us (120 us to 120 us) 1.40x | 4.64 ms (4.63 ms to 4.66 ms) 54.15x |

Reading the table:

- **jaison walks the type information at run time.** `json_write_native` and `parse_value` branch on `Type_Info.type` for every value and, for every object, build a `Table` of the struct's member names (the source comment says "@Speed: Building this table every time is pretty silly"). Jacket's generated code has no such work, which is the 1.7x to 2.7x on typed decodes and about 3x on encoding.
- **Contiguous does less work per string.** It does not unescape (escape sequences stay in the strings), only checks the input with `assert` (bad input stops the program instead of returning an error), and never copies a string. On `api`, whose bodies are long and full of escapes, it takes 0.80x of Jacket's default time and 0.88x of `borrow`'s; on `records` both Jacket columns are faster. On the untyped trees and on encoding Jacket is ahead, and on the nested configuration tree Contiguous is 50 times slower (relative offsets that are fixed up in a pass that depends on depth).
- **`borrow` matters most for untyped trees** (0.70x to 0.88x), where every key and number is a slice too. Typed decodes gain 4% to 9%: member names are matched, not stored, and numbers are converted either way.
- **Encoding** is 1.5x (`api`) to 2.1x (`records`) faster than Contiguous: Jacket prints floats with Schubfach and integers without going through `print`.
