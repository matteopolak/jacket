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

- `from_json($T, text, options)` returns `T, Json_Error` (`kind`, `message`, `offset`, `line`, `column`, `path` such as `tags[2]`). `Json_Options`: `max_depth` 256, `max_size` 1 GiB (`.LIMIT`). `write_json(*builder, value)` encodes into a builder. `Json_Value` is built with `json_null/bool/int/...`, `json_push`, `json_set`, read with `json_get`, `json_at`, `json_count`, `json_as_*`.
- **From a stream.** `Json_Source { data, fill }` gives the document in pieces; `from_json(T, source)` gathers it in one buffer then decodes, so any cut gives the same value or error. Json does not import Io; `Io.read_json(reader, T)` and the HTTP client's `read_json` build a source.
- **Types.** Integers are exact over their range (`.OUT_OF_RANGE` otherwise). Floats print as the shortest round-trip string; NaN/infinity encode as `null`; decoding is correctly rounded (direct conversion for up to 19 digits, Eisel-Lemire, else Basic's conversion; `1e999999` is `.OUT_OF_RANGE`). Supported: strings, bools, enums (by name), arrays, pointers, nested/recursive structs, `Hash_Table` with string keys, `Json_Value`. Not 128-bit integers.
- **Parsing** is strict RFC 8259 (no BOM, trailing commas, lone surrogates, bad UTF-8). A failed decode frees everything it allocated and returns a default `T`. Unknown fields are ignored unless denied; a missing field without `default` is an error; duplicate keys are an error.
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
| `deny_unknown_fields`, `transparent` | struct | unknown keys are errors; one-field struct encodes as its field |

Written as a note after the member's `;` (struct-level before `{`). Mistakes are compile errors.

### Query and form text

`from_query($T, pairs)` and `from_param($T, text)` (`query.jai`) decode URL query/form pairs and single strings (path segment, header) by the type of the field each key names, with the same `@json` notes and errors as `from_json` (the pairs become the equivalent JSON object). A repeated key fills an array field and is a "duplicate field" error otherwise. The HTTP server's `Query`, `Form`, `Path` and `Params` use them.

```jai
Search :: struct @json(default) { q: string; limit := 20; tags: [..] string; }
search, error := from_query(Search, pairs);        // "?q=jai&tags=a&tags=b"
page, error2 := from_param(int, "42");
```

### Specialization and SIMD

Straight-line code per struct and field, no runtime type switch. Keys match by length test then 8/4/2/1-byte word compares against compile-time constants; compact output appends prebuilt `"key":` constants. Constant documents can be decoded with `#run` and baked in. Float printing uses Schubfach (`number.jai`, table in `pow10.jai`); float parsing uses `pow5.jai`. Vector scans (`Extensions/SIMD`, `u8x16`) accelerate escaping and string scanning; for texts of at least `json_index_min_size` (256) bytes a simdjson-style stage 1 index (`index.jai`) finds string extents, abandoned when entries are too dense or sparse, and any anomaly re-parses with the scalar reader so errors are identical on every path. A full structural index for typed decoding was tried and dropped: the schema-guided single pass is as fast. Benchmarks: `stdlib/Extensions/Json/bench/bench.jai` (`-O2`); `tools/json_bench_data.py` fetches the large documents. The vector scans are worth about 1.0 to 1.1x; the parser is bound by value construction, allocation and number conversion.

## How to change it

- **New attribute:** parse in `collect_leaves` (`generate.jai`), carry on `Leaf`, emit in `generate_encoder`/`generate_decoder`. **New type:** cases in `emit_encode` and `emit_decode`; a new owning type also needs `emit_free` or failed decodes leak (`tests/leaks.jai`).
- `query.jai` field kinds need a `Pair_Kind` and a case in `generate_pair_kinds`. Runtime helpers are in `reader.jai`/`writer.jai`; stage 1 in `index.jai`.
- Writer pieces (`put`, `builder_reserve`, `copy_bytes`): a reserved window is valid only until the next append; write at most what you reserved, then `builder_commit`. `tests/encode.jai` pins output text exactly.
- Vector paths stay behind `simd_enabled()` and must match scalar results (`tests/differential.jai`). Tables in `pow5.jai`/`pow10.jai` are generated; headers say how. `tests/numbers.jai` sweeps are small by default: set `FULL_SWEEP` for the large run.
- JSONTestSuite (`tests/fixtures/JSONTestSuite/`, MIT) must stay byte-exact; `tools/gen_json_corpus.py` regenerates `tests/fixtures/suite_cases.jai`; per-case choices for `i_` cases are in `tests/suite.jai`.

## Configuration

`json_use_simd` (true), `json_index_min_size` (256), `Json_Options.max_depth`/`max_size`, `json_index_stats` (counters).

## Dependencies

`Basic`, `String`, `Compiler`, `Hash_Table`, `Extensions/SIMD`. Nothing from Io (`Io.read_json` imports Json). The structural-index design follows simdjson (Langdale and Lemire); the code is written for this repo.
