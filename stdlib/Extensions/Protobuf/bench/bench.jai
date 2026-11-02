// jaic -O2 bench.jai -exe <scratch>/bench; bench [--generation-only]
// bench [workload] [encode|decode [short]] [--borrow] isolates operations/modes.
// `string-logs-json` and `spans-json` time proto3 JSON next to Extensions/Json on plain
// structs holding the same data.
// packed-varints is an optional workload for repeated-field pre-count experiments.
// Generation is also measured with jaic -interp bench.jai -- --generation-only.
#import "Basic";
Proto :: #import "Extensions/Protobuf";
Json :: #import "Extensions/Json";
#import "Base64";
Schema :: #import "Extensions/Protobuf/Schema";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = #string END
syntax="proto3";
message Leaf { int32 x=1; string label=2; }
message Mixed { uint64 id=1; string name=2; Leaf child=3; repeated Leaf children=4; bytes data=5; bool active=6; }
message Packed { repeated fixed32 numbers=1; }
message Varints { repeated uint64 numbers=1; }
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

// Allocation headers are used only for the single counted operation, outside timing.
Counter :: struct {
    parent: Allocator;
    allocations: s64;
    bytes: s64;
    live: s64;
    peak: s64;
}

borrow: bool;

counting :: (mode: Allocator_Mode, size: s64, old_size: s64, old: *void, data: *void) -> *void {
    c := cast(*Counter) data;
    memory := cast(*u8) old;
    previous: s64;

    if memory {
        memory -= 16;
        previous = (cast(*s64) memory).*;
    }

    if mode == .FREE {
        c.live -= previous;
        return c.parent.proc(mode, 0, previous + 16, memory, c.parent.data);
    }

    if mode == .ALLOCATE || mode == .RESIZE {
        c.allocations += 1;
        c.bytes += size;
        c.live += size - previous;
        // Track client-visible live bytes; backend realloc scratch is excluded.
        c.peak = max(c.peak, c.live);
        result := cast(*u8) c.parent.proc(mode, size + 16, previous + 16, memory, c.parent.data);

        assert(result != null);
        (cast(*s64) result).* = size;

        return result + 16;
    }

    return c.parent.proc(mode, size, old_size, old, c.parent.data);
}

bench_decode :: ($T: Type, input: [] u8) -> T, Proto.Error {
    return Proto.decode(T, input, .{ borrow = borrow });
}

// Optional workload and operation filters keep profiler stacks isolated.
workload: string;
operation: string;
profile_short: bool;

measure :: (name: string, value: $T, iterations: s64) {
    if workload.count && workload != name return;

    encoded, error := Proto.encode(value);

    assert(error.kind == .NONE);
    defer free(encoded.data);

    if operation.count {
        count := iterations;
        if profile_short count = ifx name == "mixed" then 10000 else ifx name == "string-logs" then 20 else 5;

        for 0..count - 1 {
            if operation == "encode" {
                bytes, ee := Proto.encode(value);
                assert(ee.kind == .NONE);
                free(bytes.data);
            } else {
                decoded, de := bench_decode(T, encoded);
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
        decoded, de := bench_decode(T, encoded);
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
    enc_allocs, enc_bytes, dec_allocs, dec_bytes, dec_peak: s64;

    push_context ctx {
        bytes, ee := Proto.encode(value);

        assert(ee.kind == .NONE);
        enc_allocs = c.allocations;
        enc_bytes = c.bytes;
        free(bytes.data);
        c.allocations = 0;
        c.bytes = 0;
        assert(c.live == 0);
        c.peak = 0;

        decoded, de := bench_decode(T, encoded);

        assert(de.kind == .NONE);
        dec_allocs = c.allocations;
        dec_bytes = c.bytes;
        dec_peak = c.peak;
        Proto.free(*decoded);
        assert(c.live == 0);
    }

    print("% bytes=% encode_ns=% decode_ns=% size_ns=% encode_MB_s=% decode_MB_s=% encode_allocs=% encode_alloc_bytes=% decode_allocs=% decode_alloc_bytes=% decode_peak_bytes=%\n",
        name, encoded.count, encode_ns, decode_ns, size_ns,
        cast(float64) encoded.count * 1000 / max(encode_ns, 1),
        cast(float64) encoded.count * 1000 / max(decode_ns, 1),
        enc_allocs, enc_bytes, dec_allocs, dec_bytes, dec_peak);
}

// The same data as plain structs for Extensions/Json; bytes travel as base64 text.
Plain_Log :: struct {
    time: string;
    level: string;
    source: string;
    text: string;
}

Plain_Logs :: struct {
    entries: [..] Plain_Log;
}

Plain_Attribute :: struct {
    key: string;
    value: string;
    number: s64;
}

Plain_Event :: struct {
    time_ns: u64;
    name: string;
    attributes: [..] Plain_Attribute;
}

Plain_Span :: struct {
    trace_id: string;
    span_id: string;
    parent_id: string;
    name: string;
    kind: Api.Kind;
    start_ns: u64;
    end_ns: u64;
    attributes: [..] Plain_Attribute;
    events: [..] Plain_Event;
    status: u32;
}

Plain_Spans :: struct {
    spans: [..] Plain_Span;
}

plain_logs :: (logs: Api.Logs) -> Plain_Logs {
    result: Plain_Logs;
    for logs.entries array_add(*result.entries, .{ copy_string(it.time), copy_string(it.level), copy_string(it.source), copy_string(it.text) });
    return result;
}

plain_attributes :: (attributes: [..] Api.Attribute) -> [..] Plain_Attribute {
    result: [..] Plain_Attribute;
    for attributes array_add(*result, .{ copy_string(it.key), copy_string(it.value), it.number });
    return result;
}

plain_spans :: (spans: Api.Spans) -> Plain_Spans {
    result: Plain_Spans;

    for spans.spans {
        span: Plain_Span;
        span.trace_id = base64_encode(cast(string) it.trace_id);
        span.span_id = base64_encode(cast(string) it.span_id);
        span.parent_id = base64_encode(cast(string) it.parent_id);
        span.name = copy_string(it.name);
        span.kind = it.kind;
        span.start_ns = it.start_ns;
        span.end_ns = it.end_ns;
        span.attributes = plain_attributes(it.attributes);
        for event: it.events array_add(*span.events, .{ event.time_ns, copy_string(event.name), plain_attributes(event.attributes) });

        span.status = it.status;
        array_add(*result.spans, span);
    }

    return result;
}

// Proto3 JSON of a generated message against Extensions/Json on a plain struct.
measure_json :: (name: string, value: $T, plain: $P, iterations: s64) {
    if workload.count && workload != name return;

    text, error := Proto.to_json(value);

    assert(error.kind == .NONE);
    defer free(text);
    plain_text := Json.to_json(plain);

    defer free(plain_text);

    start := current_time_monotonic();

    for 0..iterations - 1 {
        out := Proto.to_json(value);
        assert(out.count == text.count);
        free(out);
    }

    encode_ns := to_nanoseconds(current_time_monotonic() - start) / iterations;

    start = current_time_monotonic();

    for 0..iterations - 1 {
        decoded, de := Proto.from_json(T, text);
        assert(de.kind == .NONE);
        Proto.free(*decoded);
    }

    decode_ns := to_nanoseconds(current_time_monotonic() - start) / iterations;

    start = current_time_monotonic();

    for 0..iterations - 1 {
        out := Json.to_json(plain);
        assert(out.count == plain_text.count);
        free(out);
    }

    plain_encode_ns := to_nanoseconds(current_time_monotonic() - start) / iterations;

    start = current_time_monotonic();

    for 0..iterations - 1 {
        decoded, de := Json.from_json(P, plain_text);
        assert(de.kind == .NONE);
        Json.free_json(*decoded);
    }

    plain_decode_ns := to_nanoseconds(current_time_monotonic() - start) / iterations;

    c := Counter.{ parent = context.allocator };
    ctx := context;
    ctx.allocator = .{ counting, *c };
    counts: [4] s64;

    push_context ctx {
        out := Proto.to_json(value);

        counts[0] = c.allocations;
        free(out);
        c.allocations = 0;

        decoded := Proto.from_json(T, text);

        counts[1] = c.allocations;
        Proto.free(*decoded);
        c.allocations = 0;

        plain_out := Json.to_json(plain);

        counts[2] = c.allocations;
        free(plain_out);
        c.allocations = 0;

        plain_decoded := Json.from_json(P, plain_text);

        counts[3] = c.allocations;
        Json.free_json(*plain_decoded);
        assert(c.live == 0);
    }

    mb :: (bytes: s64, ns: s64) -> float64 {
        return cast(float64) bytes * 1000 / max(ns, 1);
    }

    print("% bytes=% encode_ns=% decode_ns=% encode_MB_s=% decode_MB_s=% encode_allocs=% decode_allocs=%\n",
        name, text.count, encode_ns, decode_ns, mb(text.count, encode_ns), mb(text.count, decode_ns), counts[0], counts[1]);
    print("%-plain bytes=% encode_ns=% decode_ns=% encode_MB_s=% decode_MB_s=% encode_allocs=% decode_allocs=%\n",
        name, plain_text.count, plain_encode_ns, plain_decode_ns, mb(plain_text.count, plain_encode_ns), mb(plain_text.count, plain_decode_ns), counts[2], counts[3]);
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

    positional: s64;

    for arg: args {
        if it_index == 0 continue;

        if arg == "--borrow" {
            borrow = true;
            continue;
        }

        if positional == 0 workload = arg;
        else if positional == 1 operation = arg;
        else if positional == 2 profile_short = arg == "short";

        positional += 1;
    }

    // A large multiplier is useful when attaching a sampling profiler.
    multiplier: s64 = 1;
    if operation.count multiplier = 100;

    spans: Api.Spans;

    if !workload.count || workload == "spans" {
        spans = spans_workload();
        encoded, error := Proto.encode(spans);

        assert(error.kind == .NONE);
        print("spans encoded bytes: %\n", encoded.count);
        free(encoded.data);
    }

    defer Proto.free(*spans);

    if !workload.count || workload == "mixed" {
        child := Api.Leaf.{ x = 42, label = "nested" };
        mixed := Api.Mixed.{ id = 123456, name = "an API response", child = *child, active = true };

        for 0..3 array_add(*mixed.children, child);

        defer array_free(mixed.children);
        measure("mixed", mixed, 1000000 * multiplier);
    }

    if !workload.count || workload == "packed-1MiB" {
        packed: Api.Packed;
        for i: 0..262143 array_add(*packed.numbers, cast(u32) i);

        defer array_free(packed.numbers);
        measure("packed-1MiB", packed, 11000 * multiplier);
    }

    if !workload.count || workload == "deep-64" {
        nodes: [65] Api.Deep;
        for i: 0..63 nodes[i].next = *nodes[i + 1];

        nodes[64].value = "leaf";
        measure("deep-64", nodes[0], 30000 * multiplier);
    }

    if !workload.count || workload == "string-logs" || workload == "string-logs-json" {
        logs: Api.Logs;
        for 0..999 array_add(*logs.entries, .{ time = "2026-11-02T12:00:00Z", level = "INFO", source = "api/request", text = "Request completed successfully with a useful response" });

        defer array_free(logs.entries);
        measure("string-logs", logs, 1000 * multiplier);

        plain := plain_logs(logs);

        defer Json.free_json(*plain);
        measure_json("string-logs-json", logs, plain, 300 * multiplier);
    }

    if workload == "packed-varints" {
        packed: Api.Varints;
        for i: 0..262143 array_add(*packed.numbers, cast(u64) i);
        defer array_free(packed.numbers);
        measure("packed-varints", packed, 100 * multiplier);
    }

    measure("spans", spans, 200 * multiplier);

    if !workload.count || workload == "spans-json" {
        if !spans.spans.count spans = spans_workload();
        plain := plain_spans(spans);

        defer Json.free_json(*plain);
        measure_json("spans-json", spans, plain, 50 * multiplier);
    }

    if !workload.count generation();
}
