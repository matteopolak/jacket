# Protobuf

Protocol Buffers in the binary wire format and the proto3 JSON mapping. Give it a `.proto` schema and the Jai types and their encoders and decoders are generated at compile time. No `protoc`, no build file. proto3, proto2 and editions 2023/2024; it passes Google's binary and JSON conformance suites.

```jai
Proto :: #import "Extensions/Protobuf";
Api   :: #import "Extensions/Protobuf/Schema"(PATH = #run Proto.beside("proto/person.proto"));

Inline :: #import "Extensions/Protobuf/Schema"(TEXT = #string END
syntax = "proto3";
message Ping { uint64 id = 1; string note = 2; }
END);

main :: () {
    p := Inline.Ping.{ id = 7, note = "hi" };
    bytes := Proto.encode(p);                          // [] u8 from context.allocator
    q, error := Proto.decode(Inline.Ping, bytes);      // error.kind == .NONE on success
    Proto.free(*q);
    Proto.free(*error);
}
```

Import the runtime with a name (`Proto ::`). A plain `#import` puts `encode`, `free` and `Error` in your scope.

## Schemas

`Extensions/Protobuf/Schema` takes exactly one of:

- `PATH`: a `.proto` file, absolute or relative to the compiler's working directory. `Proto.beside("x.proto")` makes it relative to the calling file.
- `DIR`: every `.proto` under a folder, into one module.
- `TEXT`: the schema itself.

```jai
Api :: #import "Extensions/Protobuf/Schema"(DIR = #run Proto.beside("proto"));
using Api.acme.billing;            // Invoice, Invoice.Line, Currency ...
```

Imports are looked up next to the importing file, then in `IMPORT_PATHS` (`;`-separated folders). `import public` re-exports; a plain import's own imports stay private. A cycle is an error at the `import` line.

`google/protobuf/*.proto` (`timestamp`, `duration`, `any`, `empty`, `wrappers`, `struct`, `field_mask`, `source_context`, `type`, `api`, `descriptor`) are built in, so they need no files. A file with the same import path on disk wins. The built-in `descriptor.proto` holds only the `*Options` messages, which is enough to declare custom options (`google.api.http`, `validate.rules`); put the real file on `IMPORT_PATHS` to use the descriptor types themselves. They are plain messages with no helper procs; `Any` becomes `Any_`.

Each import of the Schema module is its own set of types. A `.proto` that two imports both pull in gives two distinct copies of its types, so when types must be shared, use one `DIR` import, or one `PATH` that imports the rest.

Other parameters:

| Parameter | Default | Effect |
| --- | --- | --- |
| `MAP` | `.HASH_MAP` | map type: `.HASH_MAP` (`Extensions/Hash_Map`), `.TABLE` (`Hash_Table`), `.ARRAY` (`[..] Proto.Map_Entry(K, V)`, wire order, duplicates kept), `.CUSTOM` |
| `MAP_MODULE`, `MAP_TYPE`, `MAP_INSERT`, `MAP_DEINIT` | | with `.CUSTOM`: a module whose polymorphic `TYPE(K, V)` has an insert `(*map, key, value)`, a deinit, and a `for_expansion` that gives the key as `it_index` |
| `IMPORT_PATHS` | | extra folders to search for imports |
| `STRIP_ENUM_PREFIX` | `false` | `Status.STATUS_ACTIVE` becomes `Status.ACTIVE`, unless another value already has the short name |

`package a.b;` puts the types in nested namespaces: `Api.a.b.Person`. A package part that is a Jai keyword gets a trailing `_`: `google.type.Date` is `Api.google.type_.Date`. Nested messages and enums are constants inside their parent: `Person.Address`.

Schema errors point at the `.proto` file, line and column. Different parameters give different module instances, so their types are distinct.

`Schema.protobuf_generate(text, options)` returns the generated Jai as a string, for checked-in code or your own `#insert`. `jaic stdlib/Extensions/Protobuf/tools/gen.jai -interp -- schema.proto` prints it.

## Types

| proto | Jai |
| --- | --- |
| `int32` `sint32` `sfixed32` / `int64` `sint64` `sfixed64` | `s32` / `s64` |
| `uint32` `fixed32` / `uint64` `fixed64` | `u32` / `u64` |
| `float` `double` `bool` | `float32` `float64` `bool` |
| `string` / `bytes` | `string` (UTF-8 checked on decode) / `[] u8` |
| enum | `enum s32`; an open enum keeps an unknown number as the value, a closed enum (proto2, `enum_type = CLOSED`) puts it in the unknown fields |
| message field | `*T`; null means absent |
| `optional` scalar (proto3), explicit presence (proto2, editions) | `Proto.Optional(T)`: `has`, `value`; `[default = x]` sets `value` while `has` stays false |
| `required` / `LEGACY_REQUIRED` | `Proto.Optional(T)` for scalars, `*T` for messages; checked on decode and encode (`.MISSING_REQUIRED` with the path) |
| proto2 `group` | a nested message, delimited on the wire |
| `repeated T` | `[..] T`; packed by default in proto3 and editions, expanded in proto2, both forms decoded |
| `map<K, V>` | set by `MAP` |
| `oneof` | `Variant(struct { none: void; ... })` with `which == .NONE` when unset |

Field names that are Jai keywords get a trailing `_` (`type_`). `json_name` becomes `@json(rename=...)`, so `Extensions/Json` uses it. Every message keeps unknown fields in `_unknown` and writes them back on encode.

Setting a oneof: `c.via.which = .EMAIL; c.via.email = "ada@example.com";`. Free the old member first if it owns memory.

## Runtime

| Proc | Does |
| --- | --- |
| `encode(value)` | bytes and `Error`: one exact-size allocation, or `.TOO_LARGE` past 2 GiB - 1 |
| `encode_into(*[..] u8 or *String_Builder, value)` | appends; on error the destination is unchanged |
| `size(value)` | encoded size in bytes |
| `decode(T, bytes, options)` | value plus `Error`; a failed decode frees what it made and returns zero |
| `merge(*value, bytes, options)` | protobuf merge: scalars replace, repeated append, messages merge, map keys replace |
| `free(*value)`, `free(*error)` | safe to call twice |
| `equal(a, b)` | deep, including unknown fields |
| `to_json(value, options)` | proto3 JSON text and `Error` |
| `write_json(*String_Builder, value, options)` | appends the JSON |
| `from_json(T, text, options)`, `merge_json(*value, text, options)` | read proto3 JSON; a failed read frees what it made |

`Decode_Options`:

- `max_depth` (100).
- `max_size` (2 GiB - 1).
- `borrow`: strings and bytes point into the input, which must outlive the value. `free` leaves them alone, so any allocator works.
- `discard_unknown`.
- `skip_utf8`: accept `string` fields without checking that they're UTF-8. Faster on text-heavy messages, but a string may then hold any bytes, so keep the check for input you don't trust.

### Extensions

Each `extend` field becomes a constant, in its package or inside the message that declares it. Its value lives in the extended message's unknown fields, so decode, encode, merge and equality carry it with no extra work:

```jai
Proto.set(*base, Api.ext.label, "hi");
label, found := Proto.get(*base, Api.ext.label);
Proto.add(*base, Api.ext.nums, 3);
nums := Proto.get(*base, Api.ext.nums);   // [..] s32
defer Proto.free(*nums);
```

- `get` returns the value and whether it was there (the `[default = ...]` when absent); a repeated extension returns every element, packed or not. The last scalar wins and message occurrences merge.
- `set`, `add` and `clear` rewrite the unknown bytes; `has` tests for a value.
- Strings and bytes from `get` point into the message's unknown fields: valid until the message changes or is freed. Free a message value with `Proto.free`.
- Extension strings are UTF-8 checked while decoding, like normal fields.
- The schema checks extension numbers against the `extensions` ranges and rejects duplicates, across imports too.
- In JSON, an extension is a `"[pkg.extension]"` key after the regular fields. Only extensions declared in the extendee's own schema module are known there.
- `discard_unknown` drops extension values, and `message_set_wire_format` is not supported.

### JSON

```jai
text, error := Proto.to_json(person);
back, read_error := Proto.from_json(Api.Person, text);
```

Output follows the proto3 JSON mapping: lowerCamelCase keys (or `json_name`), 64-bit integers as strings, enums by name, bytes as base64, and fields in declaration order. Fields with implicit presence print only when non-default. Input also takes the `.proto` field names, numbers for enums, and quoted or plain numbers.

The well-known types use their own forms: Timestamp as RFC 3339, Duration as `"1.500s"`, FieldMask as one string, the wrappers as bare values, and Struct, Value and ListValue as plain JSON. An Any prints `@type` and the message's fields, so the reader needs to know its types: pass `registry = Api.proto_registry` (each schema module generates one, listing every message).

`Json_Options`:

- `emit_defaults`: print default and empty fields too.
- `proto_names`: keys are the `.proto` names.
- `enum_numbers`: enums print as numbers.
- `pretty`: indent by two spaces.
- `ignore_unknown_fields`: skip unknown keys and unknown enum names instead of failing.
- `max_depth` (100), `registry`.

JSON costs nothing until a program calls it: the codec for a message is built from its Jai type on first use, and a schema module only adds a few notes and the registry.

### Memory

A copying decode allocates a handful of blocks, not one allocation per field. Strings and bytes are copied into chunks that grow with the input (never more than the bytes left to read), and repeated fields and nested messages come from an arena owned by the top-level message. Decoding 2,000 tracing spans makes 18 allocations; a `borrow` decode uses the same arena and makes 14.

- Free only the top-level value. Nested messages, repeated fields and decoded strings are views into its blocks: don't free them, and don't move them into another message.
- `merge` and decoding into an existing value add blocks; values from earlier decodes stay valid until `free`.
- Assigning a field directly is fine if the new value is your own allocation from the same allocator. To take a string or bytes from another decoded message, copy it with `Proto.set_string(*owner, *owner.field, value)` or `Proto.set_bytes`. A replaced value that lives in a block is never freed early.
- A message you build by hand frees field by field, as before.

`Error` has `kind` (`.TRUNCATED`, `.BAD_TAG`, `.BAD_WIRE_TYPE`, `.BAD_GROUP`, `.VARINT_TOO_LONG`, `.TOO_DEEP`, `.TOO_LARGE`, `.BAD_UTF8`, `.FIELD_ZERO`, and for JSON `.BAD_JSON`, `.BAD_VALUE`, `.UNKNOWN_FIELD`, `.DUPLICATE_FIELD`, `.UNKNOWN_TYPE`), `offset`, `path` (`address.lines[2]`) and, from JSON, a `message`.

## Not supported yet

- Editions newer than 2024. Unknown features are a schema error; custom feature extensions (`pb.cpp`...) are ignored.
- `service` blocks are read and skipped. gRPC is planned as a separate Extensions module.
- Not planned: the text format and reflection.

## Syntax and features

`syntax = "proto3"`, `syntax = "proto2"` and `edition = "2023"` or `"2024"` all resolve to one set of features per field:

- `field_presence`: `EXPLICIT`, `IMPLICIT` or `LEGACY_REQUIRED`.
- `enum_type`: `OPEN` or `CLOSED`.
- `repeated_field_encoding`: `PACKED` or `EXPANDED`.
- `utf8_validation`: `VERIFY` or `NONE`. Strings are checked in proto3 and editions, not in proto2.
- `message_encoding`: `LENGTH_PREFIXED` or `DELIMITED`, the group encoding.
- `json_format`: read and ignored.

proto2 and proto3 are fixed defaults for these. In editions, `option features.x = ...` on a file, message, field, oneof or enum overrides them for everything inside.

## Conformance and speed

`benchmarks/compare/protobuf/conformance/testee.jai` runs Google's `conformance_test_runner` against this module. It generates the test messages from a protobuf checkout given at build time: `jaic -O2 testee.jai -add 'PROTOBUF_ROOT :: "/path/to/protobuf";'`.

The runner is upstream code, so it runs only inside the `tools/bench` sandbox. It runs with `--maximum_edition 2024 --enforce_recommended --failure_list failure_list.txt`. Every binary test passes (2,864), proto2, proto3 and editions alike, and so does every JSON test in both directions (5,786 tests in all); the failure list is empty. Text-format tests are skipped.

`stdlib/Extensions/Protobuf/bench/bench.jai`, built with `-O2`, covers four shapes: a mixed message, 1 MiB of packed numbers, 64 levels of nesting, and string-heavy logs. Running it with `-interp -- --generation-only` times generation instead. A 100-message, 1,000-line schema generates in about 190 ms in the interpreter and about 4 KB of Jai per 9-field message.

JSON is close to `Extensions/Json` reading the same data into a plain struct. On the string-heavy logs it decodes at about 770 MB/s (Extensions/Json: about 670 MB/s) with 7 allocations against 4,010, and encodes at about 1.6 GB/s, roughly 1.4 times slower than Extensions/Json.

A nightly job in `.github/workflows/fuzz.yml` fuzzes the binary and JSON decoders under ASan (`tools/fuzz/properties/Protobuf.jai`).

## Compared with jai-protobuf and C++

`benchmarks/compare/protobuf/benches/protobuf.jai` (`tools/bench/library_compare.py protobuf`, see [library comparisons](../tools/library-comparisons.md)) runs the same schema and values through Jacket and jai-protobuf, after checking that each library decodes the other's bytes. `benchmarks/compare/protobuf/cpp/build.py` runs the same shapes through Google's C++ library. Apple M5, 16 GB, `-O2`; the fastest of three runs, per call:

| Workload | Jacket | `borrow` | `borrow` + `skip_utf8` | jai-protobuf | C++ |
| --- | --- | --- | --- | --- | --- |
| encode mixed (83 B) | 98 ns | | | 250 ns | 106 ns |
| decode mixed | 151 ns | 132 ns | 127 ns | 129 ns² | 380 ns |
| encode 1 MiB packed | 13.6 µs | | | 13.6 µs | 29.4 µs |
| decode 1 MiB packed | 13.6 µs | 13.5 µs | | 13.4 µs² | 18.8 µs |
| encode 64-deep | 1.48 µs | | | n/a¹ | 0.89 µs |
| decode 64-deep | 1.47 µs | 1.46 µs | 1.47 µs | n/a¹ | 2.48 µs |
| encode logs (98 KB) | 23.9 µs | | | 69.8 µs | 33.9 µs |
| decode logs | 40.1 µs | 27.7 µs | 22.1 µs | 36.3 µs² | 130 µs |
| encode 2,000 spans (574 KB) | 392 µs | | | 1.14 ms | 405 µs |
| decode 2,000 spans | 745 µs | 637 µs | 415 µs | 576 µs² | 1.68 ms |

¹ A message that contains itself doesn't compile in jai-protobuf.

² jai-protobuf decodes less: strings always point into the input, unknown fields are dropped, and it doesn't check UTF-8 or required fields, accepts field 0 and stray end-group tags, and has no depth limit. `borrow` + `skip_utf8` is the closest Jacket decode.

Jacket encodes two to three times as fast as jai-protobuf, except packed numbers, where both copy at memory speed. A Jacket decode of the spans makes 18 allocations (14 with `borrow`); jai-protobuf makes 5,346 and C++ 57,931. A program that encodes and decodes one message adds 89 KiB to an `-O2` executable, against 26 KiB for jai-protobuf, and compiles in about the same time.

| Feature | Jacket | jai-protobuf |
| --- | --- | --- |
| proto3 | ✅ | ✅ |
| proto2: presence, defaults, closed enums | ✅ | ⚠️ defaults only |
| editions 2023/2024 | ✅ | ⚠️ parsed as proto3 |
| groups | ✅ | ❌ |
| oneof, maps, packed and unpacked | ✅ | ✅ |
| recursive messages | ✅ | ⚠️ only through `repeated` |
| imports, whole folders | ✅ | ✅ |
| well-known types without the files | ✅ | ❌ |
| unknown fields kept | ✅ | ❌ |
| required-field check | ✅ | ❌ |
| UTF-8 check | ✅ | ❌ |
| malformed input rejected | ✅ | ⚠️ accepts stray groups and field 0; no depth limit |
| merge | ✅ | ✅ |
| equality | ✅ | ❌ |
| copying or borrowed strings | ✅ both | ⚠️ borrowed only |
| proto3 JSON | ✅ | ❌ |
| text format, reflection | not planned | ❌ |
| services | gRPC planned as a separate module | ❌ |
| extensions | ✅ | ❌ |

## Real-world schemas

`tools/protobuf_schemas` fetches hash-pinned public schemas into a cache and generates each one (Apple M5, interpreter generation, then an `-O2` build of a program that encodes and decodes a message):

| Schema | Generate | `-O2` build | Jai emitted | protoc C++ |
| --- | --- | --- | --- | --- |
| ONNX | 9 ms | 0.8 s | 0.1 MB | 1.3 MB |
| OpenTelemetry | 31 ms | 0.7 s | 0.2 MB | 2.5 MB |
| Pub/Sub | 31 ms | 0.9 s | 0.3 MB | 5.4 MB |
| TensorFlow framework | 43 ms | 1.1 s | 0.3 MB | 4.6 MB |
| Kubernetes core/v1 | 62 ms | 3.1 s | 0.8 MB | 13.3 MB |
| googleapis, 299 files | 0.6 s | 1.9 s | 6.7 MB | 114 MB |
| Envoy API, 824 files | 0.7 s | 9.0 s | 5.6 MB | 97 MB |

All 25 variants in `schemas.json` generate, compile and round-trip. jai-protobuf builds `descriptor.proto` and the TensorFlow config; the others stop on recursive messages, packages repeated across files, or fields named like Jai types (`float`).

```sh
python3 -I tools/protobuf_schemas/fetch.py
python3 -I tools/protobuf_schemas/measure.py --jaic target/release/jaic --json out.json
```

## How it works

- **Generation:** `Schema/module.jai` runs `#insert #run` over the module parameters. Each file is lexed and parsed, and imports are loaded into one graph, deduplicated by real path. Then `resolve.jai` handles scoping, names and checks, and `emit.jai` writes the Jai.
- **Generated code:** each message gets constants `proto_size`, `proto_encode`, `proto_merge`, `proto_free` and `proto_equal`. `Proto.encode(T)` calls `T.proto_encode`, so one runtime serves every schema module. A field is one call to a polymorphic runtime helper (`size_field`, `encode_field`...) with the tag folded to a constant. Map entries share one `Entry_Codec(K, V, ...)`.
- **Encoding:** a size pass fills a cache of nested message sizes, then one exact-size write runs front to back.
- **Decoding:** a loop per message checks tags in field order, then falls back to a switch on the field number. Every length is checked against the bytes left before anything is sliced or allocated. Copied payloads and the decode arena live in `storage.jai`; a message's `proto_storage` links its blocks.

## How to change it

- **Grammar:** `Schema/lexer.jai` and `Schema/parser.jai`.
- **Schema checks and naming:** `Schema/resolve.jai`.
- **Features and syntax defaults:** `Schema/features.jai`. A new edition is a new set of defaults there.
- **What a message generates:** `Schema/emit.jai`.
- **Wire primitives, bounds, depth and UTF-8:** `wire.jai`.
- **Decode memory:** `storage.jai` (chunk sizes, arena growth, which frees are skipped). Run the ownership tests in `tests/suite.jai` and compare peak memory with `bench.jai` after any change.
- **Well-known types:** `Schema/well_known/schemas.jai`, as our own text.
- **Generated size:** keep it small. Every program that imports a schema compiles what `emit.jai` writes, so new per-field logic goes into a runtime helper, not into the generated text.
- **Tests:** `stdlib/Extensions/Protobuf/tests/` (hex fixtures, spec cases and Fuzz properties), with schema errors in `negative/`.

## Generated gRPC services

Message-only generation remains the default and does not import Http or Tls.
Enable `GRPC = true` on a Schema import, `Generate_Options.grpc = true` with
`protobuf_generate`, or `--grpc` in the standalone `stdlib/Extensions/Protobuf/tools/gen.jai` command to emit
[gRPC services](grpc.md). Service/method options are parsed, endpoints must resolve
to visible messages, and independent client/server streaming flags support all four
RPC cardinalities. Original package/service/method names define wire paths; Jai
identifier escaping does not change them. Service/member/package collisions are
reported at schema locations. Generated transport is native-only.

Each service exposes typed client/server contexts, method descriptors, unary call
conveniences and a `Handlers`/`register` server API. Handlers and their state belong
to the caller and outlive all registered requests. Generated adapters use the same
message types and common Grpc runtime; they do not regenerate a separate Schema
module or duplicate framing. When GRPC is false, service bodies retain the previous
skip behavior. See the [independent peer procedure](../tools/grpc-interop.md) for the
required generated-client/generated-server interoperability matrix.
