// jaic -O2 bench.jai -exe <scratch>/bench; bench [--generation-only]
// Generation is also measured with jaic -interp bench.jai -- --generation-only.
#import "Basic";
Proto :: #import "Extensions/Protobuf";
Schema :: #import "Extensions/Protobuf/Schema";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = #string END
syntax="proto3";
message Leaf { int32 x=1; string label=2; }
message Mixed { uint64 id=1; string name=2; Leaf child=3; repeated Leaf children=4; bytes data=5; bool active=6; }
message Packed { repeated fixed32 numbers=1; }
message Deep { Deep next=1; string value=2; }
message Log { string time=1; string level=2; string source=3; string text=4; }
message Logs { repeated Log entries=1; }
END);

Counter :: struct {
    parent: Allocator;
    allocations: s64;
    bytes: s64;
}

counting :: (mode: Allocator_Mode, size: s64, old_size: s64, old: *void, data: *void) -> *void {
    c := cast(*Counter) data;

    if mode == .ALLOCATE || mode == .RESIZE {
        c.allocations += 1;
        c.bytes += size;
    }

    return c.parent.proc(mode, size, old_size, old, c.parent.data);
}

measure :: (name: string, value: $T, iterations: s64) {
    encoded, error := Proto.encode(value);

    assert(error.kind == .NONE);
    defer free(encoded.data);
    start := current_time_monotonic();

    for 0..iterations - 1 {
        bytes, ee := Proto.encode(value);
        assert(ee.kind == .NONE && bytes.count == encoded.count);
        free(bytes.data);
    }

    encode_ns := to_nanoseconds(current_time_monotonic() - start) / iterations;

    start = current_time_monotonic();

    for 0..iterations - 1 {
        decoded, de := Proto.decode(T, encoded);
        assert(de.kind == .NONE);
        Proto.free(*decoded);
    }

    decode_ns := to_nanoseconds(current_time_monotonic() - start) / iterations;

    start = current_time_monotonic();
    checksum: s64;
    for 0..iterations - 1 checksum += Proto.size(value);

    size_ns := to_nanoseconds(current_time_monotonic() - start) / iterations;

    assert(checksum == encoded.count * iterations);
    c := Counter.{ parent = context.allocator };
    ctx := context;
    ctx.allocator = .{ counting, *c };
    enc_allocs, enc_bytes, dec_allocs, dec_bytes: s64;

    push_context ctx {
        bytes, ee := Proto.encode(value);

        assert(ee.kind == .NONE);
        enc_allocs = c.allocations;
        enc_bytes = c.bytes;
        free(bytes.data);
        c.allocations = 0;
        c.bytes = 0;
        decoded, de := Proto.decode(T, encoded);

        assert(de.kind == .NONE);
        dec_allocs = c.allocations;
        dec_bytes = c.bytes;
        Proto.free(*decoded);
    }

    print("% bytes=% encode_ns=% decode_ns=% size_ns=% encode_MB_s=% decode_MB_s=% encode_allocs=% encode_alloc_bytes=% decode_allocs=% decode_alloc_bytes=%\n",
        name, encoded.count, encode_ns, decode_ns, size_ns,
        cast(float64) encoded.count * 1000 / max(encode_ns, 1),
        cast(float64) encoded.count * 1000 / max(decode_ns, 1),
        enc_allocs, enc_bytes, dec_allocs, dec_bytes);
}

generation :: () {
    b: String_Builder;

    append(*b, "syntax='proto3';\npackage timing;\n");

    for i: 0..99 {
        print(*b, "message Message% {\n", i);
        append(*b, "uint64 id=1;\nstring name=2;\nrepeated sint32 numbers=3;\noptional bool enabled=4;\nbytes data=5;\n");
        print(*b, "Message% next=6;\n", (i + 1) % 100);
        append(*b, "map<string,int32> scores=7;\noneof selection { string text=8; int64 code=9; }\n}\n");
    }

    text := builder_to_string(*b);

    defer free(text);

    for 0..4 {
        start := current_time_monotonic();
        output, error := Schema.protobuf_generate(text);
        elapsed := to_nanoseconds(current_time_monotonic() - start);

        assert(!error.message.count);
        lines: s64;
        for output if it == 10 lines += 1;

        print("generation_ns=% source_bytes=% output_bytes=% output_lines=% messages=100\n", elapsed, text.count, output.count, lines);
        free(output);
        reset_temporary_storage();
    }
}

main :: () {
    args := get_command_line_arguments();

    defer free(args.data);

    if args.count > 1 && args[1] == "--generation-only" {
        generation();
        return;
    }

    child := Api.Leaf.{ x = 42, label = "nested" };
    mixed := Api.Mixed.{ id = 123456, name = "an API response", child = *child, active = true };

    for 0..3 array_add(*mixed.children, child);

    defer array_free(mixed.children);
    measure("mixed", mixed, 20000);
    packed: Api.Packed;
    for i: 0..262143 array_add(*packed.numbers, cast(u32) i);

    defer array_free(packed.numbers);
    measure("packed-1MiB", packed, 30);
    nodes: [65] Api.Deep;
    for i: 0..63 nodes[i].next = *nodes[i + 1];

    nodes[64].value = "leaf";
    measure("deep-64", nodes[0], 1000);
    logs: Api.Logs;
    for 0..999 array_add(*logs.entries, .{ time = "2026-11-02T12:00:00Z", level = "INFO", source = "api/request", text = "Request completed successfully with a useful response" });

    defer array_free(logs.entries);
    measure("string-logs", logs, 100);
    generation();
}
