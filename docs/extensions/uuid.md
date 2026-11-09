# UUID

`#import "Extensions/Uuid"` provides 16-byte UUID values, strict text conversion,
and v3, v4, v5 and v7 generation following [RFC 9562](https://www.rfc-editor.org/rfc/rfc9562.html).
`UUID_NIL` and `UUID_MAX` are the all-zero and all-one sentinels.

## Text and ownership

`parse_uuid(text)` returns `(Uuid, Uuid_Error)`. It accepts exactly canonical
8-4-4-4-12 hexadecimal text (either hex case), a lowercase `urn:uuid:` prefix,
or one pair of braces. It rejects whitespace, compact hex and combined wrappers.
Parsing preserves any version and variant: recognizing a value does not imply
support for generating that version. Input is capped at 45 bytes and never copied.

```jai
id, error := parse_uuid("{919108F7-52D1-4320-9BAC-F847DB4148A8}");
text := format_uuid(id); // 919108f7-52d1-4320-9bac-f847db4148a8
free(text);
// Or append_uuid(*builder, id).
```

`Uuid` owns no memory. `format_uuid` allocates 36 bytes using its optional
allocator (default `context.allocator`). Errors contain a kind, static message and
zero-based byte offset; neither errors nor parsed values need freeing. Generation
errors use offset zero; name-limit errors point at the limit. Length errors point
at the input end, capped at 45. Syntax errors point at the offending byte.

## Generation

`uuid_v4()` uses the system CSPRNG. `uuid_v7()` combines Unix milliseconds,
a 12-bit counter and 62 fresh random bits. Both return `(Uuid, Uuid_Error)`;
randomness failure returns `.RANDOM` and nil.

V7 uses RFC 9562 section 6.2 **method 1**. An atomic word holds the timestamp and
counter, shared by all threads importing the module. On a newer millisecond the
counter starts at a random value from 1 through 1024; subsequent calls increment
it. A backward clock holds the previous timestamp and increments the counter.
At 4095, further calls return `.EXHAUSTED` until the clock passes that timestamp.
No waiting or counter wrap occurs. Clocks outside the unsigned 48-bit Unix
millisecond range return `.CLOCK`. Successful calls are ordered by their atomic
state update; thread scheduling can reorder when callers observe completion.
State lasts for the process and is not persisted across restarts.

`uuid_v3(namespace, name)` and `uuid_v5(namespace, name)` hash
namespace bytes followed by the exact name bytes, then set version and variant.
Names are not normalized or interpreted as text. V3 uses existing MD5 for legacy
compatibility; v5 uses existing SHA-1. Neither is a security primitive.

## Comparison, maps and JSON

`uuid_version` reads the raw version nibble; `uuid_variant` returns `.NCS`, `.RFC`,
`.MICROSOFT` or `.FUTURE`. `uuid_compare(a, b)` returns -1, 0 or 1 in network-byte
order, which also orders canonical strings. Equality compares all bytes.

```jai
Map :: #import "Extensions/Hash_Map";
map: Map.Map(Uuid, int, uuid_hash, uuid_equal);
Map.insert(*map, UUID_NIL, 7);
Map.deinit(*map);
```

Use Json's field handler aliases in the containing struct. Encoding always emits
a canonical string; decoding accepts the same forms as `parse_uuid` and reports
invalid UUIDs as Json `.INVALID_VALUE`. Json's options bound the enclosing input.

```jai
U :: #import "Extensions/Uuid";
Item :: struct {
    id: U.Uuid; @json(with=uuid_json)
    uuid_json_encode :: U.uuid_json_encode;
    uuid_json_decode :: U.uuid_json_decode;
}
```

## Changing and checking it

Text/layout logic, bounded name hashing and atomic v7 state live in `module.jai`.
Keep version and variant rewriting centralized and preserve the atomic timestamp
and counter pair when changing monotonicity. Extend `tests/uuid.jai` for changes;
it embeds Appendix A/B vectors and checks rejection offsets, allocation balance,
JSON and map integration. V1/v6/v8 generation is outside this module's scope.

```sh
jacket test --path stdlib/Extensions/Uuid
jacket test --path stdlib/Extensions/Uuid --interp
jailint -D warnings stdlib/Extensions/Uuid/module.jai stdlib/Extensions/Uuid/tests/*.jai stdlib/Extensions/Uuid/benches/bench.jai
jacket bench --path stdlib/Extensions/Uuid
python3 stdlib/Extensions/Uuid/bench/reference.py
```

Dependencies are `Basic`, `String`, `Hash`, `Extensions/Json`, internal SHA-1,
`md5` and the internal OS random source (`getentropy` on Unix,
`BCryptGenRandom` on Windows). Generation requires an OS CSPRNG; no seed or
external service is configured. The reference benchmark requires Python with
`uuid.uuid7` for its v7 comparison.
