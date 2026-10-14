# Json: typed JSON generated at compile time (jaic extension)

Shared rules of Io, Tls, Http and Json (pointers or values, `close(x, now)`, errors, who frees, milliseconds): [conventions](extensions-conventions.md).

## What it is

`#import "Extensions/Json";` gives `to_json`, `from_json` and `free_json`, modelled on Rust's `serde_json`. Encoders and decoders for your own types are generated at compile time from `Type_Info` (`#insert` inside polymorphic procedures, the same technique as `Extensions/Regex` and `Extensions/Args`). A jaic extension, not official Jai.

```jai
User :: struct @json(rename_all=camelCase) {
    id: int;
    display_name: string;                       // "displayName"
    email: string;  @json(skip_serializing_if=empty,default)
    tags: [..] string;
}

text := to_json(user);                          // to_json(user, pretty = true) indents
user, error := from_json(User, text);           // error.kind == .NONE on success
if error.kind != .NONE print("%\n", error_text(error));
value, error := from_json(Json_Value, text);    // a Json_Value tree when the shape is unknown
free_json(*user);                               // one free for a value, a Json_Value and a Json_Error
```

## API

| Procedure | Purpose |
| --- | --- |
| `to_json(value, pretty := false) -> string` | encode any supported value; caller frees the string |
| `write_json(*builder, value, pretty := false)` | encode into a `String_Builder` |
| `from_json($T, text, options := .{}) -> T, Json_Error` | decode into `T` (`T` = `Json_Value` for an untyped tree); the error has `kind`, `message`, `offset`, `line`, `column`, `path` (`tags[2]`) |
| `from_json($T, Json_Source, options := .{}) -> T, Json_Error` | the same for a document that arrives in pieces (see below); `Io.read_json(reader, T)` is this over an `Io.Reader` |
| `json_null/bool/int/uint/float/string/array/object`, `json_push`, `json_set` | build a `Json_Value` |
| `json_get`, `json_at`, `json_count`, `json_as_bool`, `json_as_string` | read one |
| `free_json(*value)` | one overload set that releases what the decoders allocated: a decoded value of any type (strings, arrays, pointers and what they point to, tables, `Json_Value`s), a `Json_Value` tree, and a `Json_Error` (`message`, `path`) |
| `error_text` | format an error |
| `from_query($T, pairs)`, `from_param($T, text)` | typed values from a URL query or form and from a single string (not JSON; see below) |

`Json_Options` has `max_depth` (256) and `max_size` (1 GiB); exceeding either is a `.LIMIT` error.

**From a stream.** `Json_Source` is `{ data, fill }`: `fill(data, buffer)` puts the next piece of text in `buffer` and returns the byte count (0 at the end) and a failure text (empty when it worked). `from_json(T, source)` gathers the pieces in one buffer, then decodes exactly as for text, so a document cut at any byte gives the same value or error; a source that fails gives a `.READ` error with its text, and one past `max_size` a `.LIMIT` error without reading on. Json does not import Io (it builds for WebAssembly); `Io.read_json(reader, T)` ([io-streams.md](io-streams.md)) makes a `Json_Source` from an `Io.Reader`, and the HTTP client's `read_json(*response, T)` is that over the response body. The decoder wants the whole document in memory and copies its strings out, so the one buffer is the only extra copy.

## Types

Integers are exact over their full range (including 64-bit); a number that does not fit is `.OUT_OF_RANGE`. Floats print as the shortest string that round-trips (`0.1`, `1e21`; integral values get `.0`); `NaN` and infinities encode as `null`. Decoding floats is correctly rounded: after the RFC 8259 grammar check, a token of at most 19 significant digits is converted directly (one exact multiplication or division when digits and power of ten are exactly representable, otherwise Eisel-Lemire with the 128-bit powers of five in `pow5.jai`, as in fast_float), and everything else, including every overflow, goes to Basic's conversion (`scan_decimal_float` and `decimal_float_to_float64`/`32`, see `Float_Parse.jai`), with no exponent limit, so `1e999999` is `.OUT_OF_RANGE` and `1e-999999` is zero. Supported: strings, bools, enums (by name, or number with `as_number`), fixed/view/resizable arrays, pointers (`null` or a value), nested and recursive structs, `Hash_Table` with string keys, and `Json_Value`. 128-bit integers are not.

## Attributes

Written as a note after the member's `;`, options comma-separated: `@json(rename=x,alias=y)`. Struct-level notes go before `{`.

| Option | Where | Effect |
| --- | --- | --- |
| `rename=x`, `alias=y` | member | JSON key; extra accepted key on decode |
| `rename_all=RULE` | struct | lowercase, UPPERCASE, camelCase, PascalCase, snake_case, SCREAMING_SNAKE_CASE, kebab-case, SCREAMING-KEBAB-CASE |
| `skip`, `skip_serializing`, `skip_deserializing` | member | omit in one or both directions |
| `skip_serializing_if=empty\|zero\|null` | member | omit when empty string/array/table, zero, or null pointer/`Json_Value` |
| `default` | member or struct | a missing field keeps its initializer instead of being an error |
| `flatten` | member | inline a struct member's fields; key clashes are compile errors |
| `with=name` | member | calls `T.name_encode(w, *field)` / `T.name_decode(r, *field)` declared inside the struct |
| `as_number` | enum member | encode as the enum's number |
| `deny_unknown_fields`, `transparent` | struct | unknown keys are errors; a one-field struct encodes as that field |

Deviation from the plan: `with=` names procedures declared inside the struct, not file-scope ones, because generated code cannot see user-scope identifiers.

Unknown fields are ignored unless denied; a missing field with no `default` is an error naming it; duplicate keys are an error. Attribute mistakes are compile errors reported through `compiler_report`.

## Parsing rules

Strict RFC 8259: no BOM, trailing commas, leading zeros, lone surrogates, invalid UTF-8 or raw control characters in strings. Line and column count characters. Encoding replaces invalid UTF-8 with U+FFFD. A failed decode frees everything it had allocated and returns a default `T`, wherever it fails.

**Freeing.** `free_json` is an overload set: `*Json_Value` (in `value.jai`) and `*Json_Error` (in `module.jai`) are plain procedures, and the generic `free_json(*value)` for every other type calls the module-private `free_json_fields(value, defaults)`, whose body is generated per type by `generate_freer` in `generate.jai`. It compares each field with the type's declared default first, so a field that kept `nick := "guest"` because the document omitted it is not freed (a string literal would crash `free`); it resets the fields it frees, so a second call is harmless. Fields with a `with=` handler belong to the handler, which must free its own allocations when it fails; `free_json` does not touch them. Do not call it on values that were not decoded (their strings may not come from the allocator). On failure a struct decoder tracks which fields it has finished (`seen[i] == 2`) and frees exactly those in a `defer`; arrays, views, tables and pointers decode elements into locals and free what they hold before reporting; a repeated key in a `Hash_Table` object replaces and frees the earlier value. The success path pays one byte store per struct field. `tests/leaks.jai` also checks that `free_json` on a decoded struct, a `Json_Value` and a `Json_Error` each leaves nothing allocated, and runs a valid document cut at every byte, with every byte replaced, and with trailing data under a counting allocator in both reader paths and checks that no block is left.

### JSONTestSuite

`tests/data/JSONTestSuite/` vendors `test_parsing` (MIT, LICENSE alongside; `.gitattributes` keeps it byte-exact). Results: all 95 `y_` accepted, all 188 `n_` rejected, and the 35 `i_` cases do not crash: 10 are accepted and 25 rejected. The per-case choice is recorded in `tests/suite.jai`; the rule is that anything outside RFC 8259 grammar, any invalid UTF-8 or invalid surrogate use, and nesting beyond the limit are rejected, while grammatical documents (large numbers, which become floats or range errors when typed) are accepted. `tools/gen_json_corpus.py` regenerates `tests/data/suite_cases.jai` from the files.

## What is specialized at compile time

- Per-type encoder and decoder bodies: straight-line code per struct, no runtime type switch. Each field's decoder is the code for that field's type, and a value of the wrong kind produces an error carrying the path.
- Key matching: for each field and alias the generator emits a length test followed by comparisons of 8/4/2/1-byte words against constants built at compile time (little-endian targets). No string table walk, no hashing. Keys with escapes are unescaped by the reader before matching.
- Key emission: compact output appends one prebuilt `"key":` or `,"key":` constant per field (pretty output goes through the indent-aware path).
- Rename rules, aliases, defaults, skip conditions and the size of the `seen` array are resolved at compile time.
- Floats: decoding converts short tokens directly (above); `float64` printing generates the shortest digits with Schubfach (`shortest_decimal` in `number.jai`, table in `pow10.jai`) and formats the text in a local buffer with one append. Subnormals and `float32` keep Basic's exact digit generator.
- Constant documents can be decoded at compile time with `#run` and baked in (see `tests/baked.jai`):

```jai
make :: () -> Config { config, error := from_json(Config, "{\"port\":80}"); return config; }
config :: #run make();
```

## Query and form text

`from_query($T, pairs)` and `from_param($T, text)` (`query.jai`) are not JSON: they decode the key/value pairs of a URL query or a urlencoded form (`Query_Pair`), and a single string such as a path segment or a header, into a typed value, for the Http server's `Query`, `Form`, `Path` and `Params` extractors. They live here because each value is read by the type of the field its key names, with the same `@json` notes (`rename`, `alias`, `default`, `deny_unknown_fields`) and the same errors as `from_json`: the pairs become the JSON object that says the same thing, which `from_json` decodes. A repeated key gives an array for an array field and is a "duplicate field" error for any other; keys no field has are ignored unless the struct denies them. They were `from_pairs` and `from_text`. Query strings and forms decode identically, so there is one function for both.

```jai
Search :: struct @json(default) { q: string; limit := 20; tags: [..] string; }
search, error := from_query(Search, pairs);        // pairs of "?q=jai&tags=a&tags=b"
page, error2 := from_param(int, "42");
```

## SIMD

Vector code (via `Extensions/SIMD`, `u8x16`: SSE2/NEON) accelerates the hot byte scans. Scalar code is the reference, and is used on wasm and wherever vectors are unavailable.

- Encoder: finds bytes that need escaping 16 at a time (with a scalar budget after high bytes, so UTF-8-heavy text does not regress).
- Parser: string bodies are scanned 16 bytes at a time for quote, backslash, control and non-ASCII bytes.
- Stage 1 index (`index.jai`): for texts of at least `json_index_min_size` (256) bytes, 64-byte blocks produce bit masks of quotes, backslashes, control and high bytes; simdjson's odd-backslash-run trick finds escaped quotes, a prefix XOR of the quote bits marks string interiors, and UTF-8 is validated at the high bytes in the same pass. The index lists string extents, so long strings are taken in one step. It is gated: if the first 32 KB has more than one entry per 48 bytes or fewer than one per 2 KB it is abandoned, because short strings and number arrays do not repay it. Any anomaly (or any later failure) makes the decoder re-parse with the scalar reader, so errors, positions and paths are identical on every path by construction.
- `json_use_simd = false` forces the scalar paths. `tests/differential.jai` compares both on thousands of generated and boundary texts; the whole suite, JSONTestSuite included, runs in interp, native, wasm-interp and wasm-native (SIMD is off on wasm, so those modes exercise scalar).

### Not done, and why

A full simdjson-style structural index (every `{}[],:` plus a stage 2 that walks it) was built experimentally and dropped: for documents of short strings and numbers it cost more to build than the scalar reader's own byte checks cost to repeat. `from_json(T)` is already a schema-guided single pass (generated decoder over the bytes, vector string scanning, no full index), and it measured equal to or faster than index-then-decode, so typed decoding does not build a structural index. A structural index or tape for untyped `from_json(Json_Value, ...)` remains a possible future step. The number fast paths that this section used to list as future work are done (floats above); an untyped number is kept as its text, so only typed decoding converts.

Tried and dropped (measured on the machine below, kept only if a row gained about 5% beyond noise): writing a string that needs no escaping with `ensure_contiguous_space` in one reservation instead of three appends (7% slower on twitter, citm and the catalog); encoding the escapes of a string and the run before each in one reservation, and the same with room for the worst case (six bytes per byte) in pieces of 512 bytes (a wash: the catalog gained 4%, twitter and citm lost about 5%, since a reservation that does not fit starts a new segment and leaves the end of the old one unused); a key written with its comma and colon by the string encoder (no gain); the separator and a number's text of a `Json_Value` array written in one reservation (canada 3 to 8% faster but twitter 9 to 13% slower, which is code layout, not the change); Schubfach for subnormals (it gives two digits where one reads back, e.g. `4.9e-323` for `5e-323`) and for `float32`. `#no_abc` is used on the loops that guard every index themselves (number, whitespace and string scans, the escaper, the float conversion): tests run under `jaic -sanitize address,undefined` stay clean.

### Benchmark

`stdlib/Extensions/Json/bench/bench.jai` (build with `-O2`); `tools/json_bench_data.py` downloads twitter.json, canada.json and citm_catalog.json, which are not vendored. `bench floats` is a generated 2 MB array of 17-digit coordinates decoded into and encoded from a typed struct. "vector" is the vector scans; "+ index" adds stage 1 (forced for every size). MB/s of JSON text.

**This machine** (x86-64 Linux, 4 cores, Intel Xeon @ 2.10 GHz, shared with other jobs, so expect a few percent of noise; canada's `to_json` alternates between two speeds from run to run). Median of five runs of the whole benchmark, default configuration (vector scans, no index), before and after the performance work:

| Document | Operation | before | after | |
| --- | --- | --- | --- | --- |
| twitter.json (632 KB) | from_json (untyped) | 276 | 382 | 1.38x |
| twitter.json | to_json (Json_Value) | 368 | 501 | 1.36x |
| canada.json (2.2 MB) | from_json (untyped) | 103 | 179 | 1.74x |
| canada.json | to_json (Json_Value) | 365 | 443 | 1.21x |
| citm_catalog.json (1.7 MB) | from_json (untyped) | 326 | 464 | 1.42x |
| citm_catalog.json | to_json (Json_Value) | 215 | 322 | 1.50x |
| generated catalog (4 MB) | from_json (untyped) | 119 | 162 | 1.36x |
| generated catalog | from_json(Catalog) | 195 | 245 | 1.26x |
| generated catalog | to_json(Catalog) | 220 | 262 | 1.19x |
| coordinates (2 MB, float64) | from_json(Floats) | 37 | 238 | 6.4x |
| coordinates | to_json(Floats) | 174 | 242 | 1.39x |

The `to_json` rows are from a second pass (encoder only): the best of repeated runs inside a process, median of 11 processes, old and new binaries interleaved on the same machine while another job was running, so absolute numbers sit below the quiet-machine ones above (canada's `to_json` was 583 in the first pass, on a quiet run of its faster speed). The `from_json` rows are from the first pass.

What moved them: `Json_Value` arrays and objects are built in a reusable stack and allocated once at their exact size (a growing `[..]` reserves at least eight 104-byte elements, so canada's pairs took 832 bytes each); the whitespace test is inline and number scanning has no bounds checks (citm is mostly indentation); floats take the direct conversion and Schubfach printing; integers of up to 19 digits skip the per-digit overflow test (typed integers, about 1.2x). String scanning and escaping without bounds checks gave 3 to 9%. The remaining cost is allocation (a text per number in a `Json_Value`) and `String_Builder.append`.

The encoder pass: the writer's small pieces (a key with its separator and colon, `{`, `,`, `null`, a number's text) are appended through `put`, which reserves room at the end of the builder's current segment (`builder_reserve`), copies short runs with two overlapping word moves (`copy_bytes`) and commits the count, instead of `String_Builder.append`'s initialisation test, copy loop and segment loop; the whole path is inlined into the generated encoders. Integers count their digits and write them two at a time from a 200-byte table straight into the reserved room; `float64` writes the Schubfach digits as characters (two at a time) and builds the text in the reserved room, so a float is one commit. A string that needs no escaping is quote, bytes and quote in one reservation. Output is byte-for-byte what it was: `tests/encode.jai` compares a struct with every field kind and attribute, compact and pretty, against fixed texts that were taken before the change.

**Apple silicon**, before this work (native `-O2`, best of two; not rerun):

| Document | Operation | scalar | vector | + index |
| --- | --- | --- | --- | --- |
| twitter.json (632 KB) | from_json (untyped) | 571 | 627 | 623 |
| twitter.json | to_json (Json_Value) | 730 | 814 | 813 |
| canada.json (2.2 MB) | from_json (untyped) | 419 | 422 | 403 |
| canada.json | to_json (Json_Value) | 1207 | 1300 | 1194 |
| citm_catalog.json (1.7 MB) | from_json (untyped) | 755 | 773 | 772 |
| citm_catalog.json | to_json (Json_Value) | 590 | 580 | 583 |
| long strings (5.7 MB) | from_json (untyped) | 1039 | 1080 | 1422 |
| generated catalog (4 MB) | from_json(Catalog) | 440 | 457 | 458 |
| generated catalog | to_json(Catalog) | 503 | 529 | 524 |

The vector scans themselves are worth about 1.0 to 1.1x (1.4x for long clean strings); the parser is bound by value construction, allocation and number conversion, not byte scanning.

## How to change it

- New attribute: parse it in `collect_leaves` (`generate.jai`), carry it on `Leaf`, and emit it in `generate_encoder` / `generate_decoder`.
- New type: add cases to `emit_encode` and `emit_decode`.
- `query.jai` is the non-JSON text decoders; a new field kind there needs a `Pair_Kind` and a case in `generate_pair_kinds` (`generate.jai`).
- Runtime helpers (`json_read_*`, `json_key`, ...) live in `reader.jai` and `writer.jai`; float printing in `number.jai` (reading a validated token calls Basic); stage 1 in `index.jai`.
- `pow5.jai` (Eisel-Lemire) and `pow10.jai` (Schubfach) are generated tables of exact big-integer arithmetic; their header comments say how. `tests/numbers.jai` checks the direct conversions against Basic's exact one (`fast_path_agrees`) and the printed digits against Basic's exact generator (`shortest_matches_exact`); both sweeps grow with `FULL_SWEEP`.
- Encoder output is pinned by `tests/encode.jai` (every field kind and attribute, compact and pretty, exact text); changing the writer's pieces (`put`, `builder_reserve`, `copy_bytes`) must leave it passing. A reserved window is only valid until the next call that appends to the builder: write at most what you reserved, then `builder_commit` the count.
- Vector paths stay behind `simd_enabled()` and must give identical results; run `tests/differential.jai`.
- A new owning type in the decoder needs a matching case in `emit_free` (generate.jai), or a failed decode leaks it; `tests/leaks.jai` shows it.
- The random sweeps in `tests/numbers.jai` and `tests/stdlib/basic-float-parse-correct-rounding.jai` are small by default so they run in seconds under a debug jaic; set their `FULL_SWEEP` constant to `true` for the large sweep.

## Configuration

`json_use_simd` (default true), `json_index_min_size` (256), `Json_Options.max_depth` / `max_size`, and `json_index_stats` (counters).

## Dependencies

`Basic`, `String`, `Compiler`, `Hash_Table`, `Extensions/SIMD`. Nothing from Io; `Io.read_json` imports Json, not the other way round.

## Credits

The structural-index design follows simdjson (Langdale and Lemire, "Parsing Gigabytes of JSON per Second") and its Rust port simd-json, both used as design references. The code here is written for this repo.
