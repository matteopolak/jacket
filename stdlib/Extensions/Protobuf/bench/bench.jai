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
enum Kind { KIND_UNSPECIFIED=0; KIND_SERVER=1; KIND_CLIENT=2; KIND_INTERNAL=3; }
message Attribute { string key=1; string value=2; int64 number=3; }
message Event { fixed64 time_ns=1; string name=2; repeated Attribute attributes=3; }
message Span {
  bytes trace_id=1; bytes span_id=2; bytes parent_id=3;
  string name=4; Kind kind=5; fixed64 start_ns=6; fixed64 end_ns=7;
  repeated Attribute attributes=8; repeated Event events=9; uint32 status=10;
}
message Spans { repeated Span spans=1; }
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

// Optional workload and operation filters keep profiler stacks isolated.
workload: string;
operation: string;
profile_once: bool;

measure :: (name: string, value: $T, iterations: s64) {
    if workload.count && workload != name return;

    encoded, error := Proto.encode(value);

    assert(error.kind == .NONE);
    defer free(encoded.data);

    if operation.count {
        count := iterations;
        if profile_once count = ifx name == "mixed" then 10000 else ifx name == "string-logs" then 20 else 5;

        for 0..count - 1 {
            if operation == "encode" {
                bytes, ee := Proto.encode(value);
                assert(ee.kind == .NONE);
                free(bytes.data);
            } else {
                decoded, de := Proto.decode(T, encoded);
                assert(de.kind == .NONE);
                Proto.free(*decoded);
            }
        }

        return;
    }

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

// xorshift64*: update the state before returning the multiplied output.
rng: u64 = 0x9E3779B97F4A7C15;

next :: () -> u64 {
    rng ^= rng >> 12;
    rng ^= rng << 25;
    rng ^= rng >> 27;
    return rng * 0x2545F4914F6CDD1D;
}

below :: (n: u64) -> u64 {
    return next() % n;
}

NAMES :: string.["GET /api/users/{id}", "POST /api/orders", "GET /api/orders/{id}", "db.query", "cache.get", "cache.set", "render.page", "auth.verify", "queue.publish", "queue.consume", "GET /healthz", "PUT /api/users/{id}"];
KEYS :: string.["http.method", "http.status_code", "http.route", "db.system", "db.statement", "net.peer.name", "net.peer.port", "user.id", "retry.count", "error.type"];
EVENT_NAMES :: string.["exception", "retry", "cache.miss", "message"];

text :: () -> string {
    n := cast(s64) (4 + below(21));
    bytes := NewArray(n, u8);
    for *bytes it.* = cast(u8) (97 + below(26));
    return .{ n, bytes.data };
}

attribute :: () -> Api.Attribute {
    a: Api.Attribute;
    // Copy literals so the generated ownership-aware free can release the fixture.
    a.key = copy_string(KEYS[below(10)]);
    if below(3) == 0 a.number = cast(s64) below(100000);
    else a.value = text();

    return a;
}

spans_workload :: () -> Api.Spans {
    result: Api.Spans;
    trace: [16] u8;

    for i: 0..1999 {
        span: Api.Span;

        if i % 8 == 0 for *trace it.* = cast,trunc(u8) next();

        span.trace_id = NewArray(16, u8);
        memcpy(span.trace_id.data, trace.data, 16);
        span.span_id = NewArray(8, u8);
        for *span.span_id it.* = cast,trunc(u8) next();

        if i % 8 != 0 {
            span.parent_id = NewArray(8, u8);
            memcpy(span.parent_id.data, result.spans[i - 1].span_id.data, 8);
        }

        span.name = copy_string(NAMES[below(12)]);
        span.kind = cast(Api.Kind) (i % 4);
        span.start_ns = 1700000000000000000 + cast(u64) i * 1000 + below(1000);
        span.end_ns = span.start_ns + below(5000000);
        count := cast(s64) (3 + below(6));
        for 0..count - 1 array_add(*span.attributes, attribute());

        count = cast(s64) below(3);
        for 0..count - 1 {
            event: Api.Event;
            event.time_ns = span.start_ns + below(1000000);
            event.name = copy_string(EVENT_NAMES[below(4)]);
            attributes := cast(s64) (1 + below(2));
            for 0..attributes - 1 array_add(*event.attributes, attribute());

            array_add(*span.events, event);
        }

        if below(10) == 0 span.status = 2;

        array_add(*result.spans, span);
    }

    return result;
}

main :: () {
    args := get_command_line_arguments();

    defer free(args.data);

    if args.count > 1 && args[1] == "--generation-only" {
        generation();
        return;
    }

    if args.count > 1 workload = args[1];
    if args.count > 2 operation = args[2];

    profile_once = args.count > 3 && args[3] == "once";

    // A large multiplier is useful when attaching a sampling profiler.
    multiplier: s64 = 1;
    if operation.count multiplier = 100;

    if !workload.count || workload == "mixed" {
        child := Api.Leaf.{ x = 42, label = "nested" };
        mixed := Api.Mixed.{ id = 123456, name = "an API response", child = *child, active = true };

        for 0..3 array_add(*mixed.children, child);

        defer array_free(mixed.children);
        measure("mixed", mixed, 200000 * multiplier);
    }

    if !workload.count || workload == "packed-1MiB" {
        packed: Api.Packed;
        for i: 0..262143 array_add(*packed.numbers, cast(u32) i);

        defer array_free(packed.numbers);
        measure("packed-1MiB", packed, 300 * multiplier);
    }

    if !workload.count || workload == "deep-64" {
        nodes: [65] Api.Deep;
        for i: 0..63 nodes[i].next = *nodes[i + 1];

        nodes[64].value = "leaf";
        measure("deep-64", nodes[0], 10000 * multiplier);
    }

    if !workload.count || workload == "string-logs" {
        logs: Api.Logs;
        for 0..999 array_add(*logs.entries, .{ time = "2026-11-02T12:00:00Z", level = "INFO", source = "api/request", text = "Request completed successfully with a useful response" });

        defer array_free(logs.entries);
        measure("string-logs", logs, 500 * multiplier);
    }

    if !workload.count || workload == "spans" {
        spans := spans_workload();

        defer Proto.free(*spans);
        print("spans encoded bytes: %\n", Proto.size(spans));
        measure("spans", spans, 100 * multiplier);
    }

    if !workload.count generation();
}
