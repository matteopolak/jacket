# Protobuf

Protocol Buffers in the binary wire format. Give it a `.proto` schema and the Jai types and their encoders and decoders are generated at compile time. No `protoc`, no build file. `@experimental`.

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
- `TEXT`: the schema itself.

Other parameters:

| Parameter | Default | Effect |
| --- | --- | --- |
| `MAP` | `.HASH_MAP` | map type: `.HASH_MAP` (`Extensions/Hash_Map`), `.TABLE` (`Hash_Table`), `.ARRAY` (`[..] Proto.Map_Entry(K, V)`, wire order, duplicates kept), `.CUSTOM` |
| `MAP_MODULE`, `MAP_TYPE`, `MAP_INSERT`, `MAP_DEINIT` | | with `.CUSTOM`: a module whose polymorphic `TYPE(K, V)` has an insert `(*map, key, value)`, a deinit, and a `for_expansion` that gives the key as `it_index` |
| `STRIP_ENUM_PREFIX` | `false` | `Status.STATUS_ACTIVE` becomes `Status.ACTIVE` |

`package a.b;` puts the types in nested namespaces: `Api.a.b.Person`. Nested messages and enums are constants inside their parent: `Person.Address`.

Schema errors point at the `.proto` file, line and column. Different parameters give different module instances, so their types are distinct.

`Schema.protobuf_generate(text, options)` returns the generated Jai as a string, for checked-in code or your own `#insert`. `jaic stdlib/Extensions/Protobuf/tools/gen.jai -interp -- schema.proto` prints it.

## Types

| proto | Jai |
| --- | --- |
| `int32` `sint32` `sfixed32` / `int64` `sint64` `sfixed64` | `s32` / `s64` |
| `uint32` `fixed32` / `uint64` `fixed64` | `u32` / `u64` |
| `float` `double` `bool` | `float32` `float64` `bool` |
| `string` / `bytes` | `string` (UTF-8 checked on decode) / `[] u8` |
| enum | `enum s32`; an unknown number is kept as the value |
| message field | `*T`; null means absent |
| `optional` scalar | `Proto.Optional(T)`: `has`, `value` |
| `repeated T` | `[..] T`; packed by default, both forms decoded |
| `map<K, V>` | set by `MAP` |
| `oneof` | `Variant(struct { ... })` with `which == .NONE` when unset |

Field names that are Jai keywords get a trailing `_` (`type_`). `json_name` becomes `@json(rename=...)`, so `Extensions/Json` uses it. Every message keeps unknown fields in `_unknown` and writes them back on encode.

Setting a oneof: `c.via.which = .EMAIL; c.via.email = "ada@example.com";`. Free the old member first if it owns memory.

## Runtime

| Proc | Does |
| --- | --- |
| `encode(value)` | one exact-size allocation |
| `encode_into(*[..] u8 or *String_Builder, value)` | appends |
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

`Error` has `kind` (`.TRUNCATED`, `.BAD_TAG`, `.BAD_WIRE_TYPE`, `.BAD_GROUP`, `.VARINT_TOO_LONG`, `.TOO_DEEP`, `.TOO_LARGE`, `.BAD_UTF8`, `.FIELD_ZERO`...), `offset` and `path` (`address.lines[2]`).

## Not supported yet

- proto2, editions, imports, `DIR` and the well-known types: each gives a schema error.
- Groups and `extend` in a schema. Unknown groups on the wire are skipped correctly.
- `service` blocks are read and skipped.
- The proto3 JSON mapping and the text format.

## How it works

- **Generation:** `Schema/module.jai` runs `#insert #run` over the module parameters. The pipeline is lexer → parser → `resolve.jai` (scoping, names, checks) → `emit.jai`.
- **Generated code:** each message gets constants `proto_size`, `proto_encode`, `proto_merge`, `proto_free` and `proto_equal`. `Proto.encode(T)` calls `T.proto_encode`, so one runtime serves every schema module.
- **Encoding:** a size pass fills a cache of nested message sizes, then one exact-size write runs front to back.
- **Decoding:** a loop per message checks tags in field order, then falls back to a switch on the field number. Every length is checked against the bytes left before anything is sliced or allocated.

## How to change it

- **Grammar:** `Schema/lexer.jai` and `Schema/parser.jai`.
- **Schema checks and naming:** `Schema/resolve.jai`.
- **What a message generates:** `Schema/emit.jai`.
- **Wire primitives, bounds, depth and UTF-8:** `wire.jai`.
- **Tests:** `stdlib/Extensions/Protobuf/tests/` (hex fixtures, spec cases and Fuzz properties), with schema errors in `negative/`.
