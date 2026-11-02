# Protobuf

Protocol Buffers in the binary wire format. Give it a `.proto` schema and the Jai types and their encoders and decoders are generated at compile time. No `protoc`, no build file. proto3, proto2 and editions 2023/2024; it passes Google's binary conformance suite.

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
| `oneof` | `Variant(struct { ... })` with `which == .NONE` when unset |

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

`Decode_Options`:

- `max_depth` (100).
- `max_size` (2 GiB - 1).
- `borrow`: strings and bytes point into the input, which must outlive the value. Use an arena and reset it instead of calling `free`, as with Json.
- `discard_unknown`.

### Memory

A copying decode allocates a handful of blocks, not one allocation per field. Strings and bytes are copied into chunks that grow with the input (never more than the bytes left to read), and repeated fields and nested messages come from an arena owned by the top-level message. Decoding 2,000 tracing spans makes 18 allocations.

- Free only the top-level value. Nested messages, repeated fields and decoded strings are views into its blocks: don't free them, and don't move them into another message.
- `merge` and decoding into an existing value add blocks; values from earlier decodes stay valid until `free`.
- Assigning a field directly is fine if the new value is your own allocation from the same allocator. To take a string or bytes from another decoded message, copy it with `Proto.set_string(*owner, *owner.field, value)` or `Proto.set_bytes`. A replaced value that lives in a block is never freed early.
- A message you build by hand frees field by field, as before.

`Error` has `kind` (`.TRUNCATED`, `.BAD_TAG`, `.BAD_WIRE_TYPE`, `.BAD_GROUP`, `.VARINT_TOO_LONG`, `.TOO_DEEP`, `.TOO_LARGE`, `.BAD_UTF8`, `.FIELD_ZERO`...), `offset` and `path` (`address.lines[2]`).

## Not supported yet

- Typed extensions: `extensions` ranges are accepted and `extend` blocks are skipped, so extension values arrive in `_unknown` and round-trip unchanged.
- Editions newer than 2024. Unknown features are a schema error; custom feature extensions (`pb.cpp`...) are ignored.
- `service` blocks are read and skipped.
- The proto3 JSON mapping and the text format.

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

The runner is upstream code, so it runs only inside the `tools/bench` sandbox. It runs with `--maximum_edition 2024 --enforce_recommended --failure_list failure_list.txt`. Every binary test passes (2,863), proto2, proto3 and editions alike. The one listed failure is the UTF-8 check on an extension string, since extensions are untyped. JSON and text-format tests are skipped.

`stdlib/Extensions/Protobuf/bench/bench.jai`, built with `-O2`, covers four shapes: a mixed message, 1 MiB of packed numbers, 64 levels of nesting, and string-heavy logs. Running it with `-interp -- --generation-only` times generation instead. A 100-message, 1,000-line schema generates in about 190 ms in the interpreter and about 4 KB of Jai per 9-field message.

A nightly job in `.github/workflows/fuzz.yml` fuzzes the decoder under ASan (`tools/fuzz/properties/Protobuf.jai`).

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
