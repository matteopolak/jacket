// Run with jacket bench --path stdlib/Extensions/Protobuf.
#import "Extensions/Testing";
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

measure :: (b: *Bench, value: $T, operation: int, borrow: bool = false, skip_utf8: bool = false) {
    encoded, error := Proto.encode(value);

    assert(error.kind == .NONE);
    defer free(encoded.data);
    if operation != 2 set_bytes(b, encoded.count);

    reset_timer(b);

    for 0..b.iterations - 1 {
        if operation == 0 {
            bytes, ee := Proto.encode(black_box(value));
            assert(ee.kind == .NONE && bytes.count == encoded.count);
            black_box(bytes);
            free(bytes.data);
        } else if operation == 1 {
            decoded, de := Proto.decode(T, black_box(encoded), .{ borrow = borrow, skip_utf8 = skip_utf8 });
            assert(de.kind == .NONE);
            black_box(decoded);
            Proto.free(*decoded);
        } else {
            black_box(Proto.size(black_box(value)));
        }
    }
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

measure_json :: (b: *Bench, value: $T, plain: $P, operation: int) {
    text, error := Proto.to_json(value);

    assert(error.kind == .NONE);
    defer free(text);
    plain_text := Json.to_json(plain);

    defer free(plain_text);
    set_bytes(b, ifx operation < 2 then text.count else plain_text.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        if operation == 0 {
            out, ee := Proto.to_json(black_box(value));
            assert(ee.kind == .NONE);
            black_box(out);
            free(out);
        } else if operation == 1 {
            decoded, de := Proto.from_json(T, black_box(text));
            assert(de.kind == .NONE);
            black_box(decoded);
            Proto.free(*decoded);
        } else if operation == 2 {
            out := Json.to_json(black_box(plain));
            black_box(out);
            free(out);
        } else {
            decoded, de := Json.from_json(P, black_box(plain_text));
            assert(de.kind == .NONE);
            black_box(decoded);
            Json.free_json(*decoded);
        }
    }
}

generation :: (b: *Bench) {
    builder: String_Builder;

    defer free_buffers(*builder);

    append(*builder, "syntax='proto3';\npackage timing;\n");

    for i: 0..99 {
        print(*builder, "message Message% {\n", i);
        append(*builder, "uint64 id=1;\nstring name=2;\nrepeated sint32 numbers=3;\noptional bool enabled=4;\nbytes data=5;\n");
        print(*builder, "Message% next=6;\n", (i + 1) % 100);
        append(*builder, "map<string,int32> scores=7;\noneof selection { string text=8; int64 code=9; }\n}\n");
    }

    text := builder_to_string(*builder);

    defer free(text);

    reset_timer(b);

    for 0..b.iterations - 1 {
        output, error := Schema.protobuf_generate(black_box(text));
        assert(!error.message.count);
        black_box(output);
        free(output);
    }
} @bench

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

protobuf_case :: (b: *Bench, kind: int, operation: int, borrow: bool = false, skip_utf8: bool = false) {
    rng = 0x9E3779B97F4A7C15;

    if kind == 0 {
        child := Api.Leaf.{ x = 42, label = "nested" };
        mixed := Api.Mixed.{ id = 123456, name = "an API response", child = *child, active = true };

        for 0..3 array_add(*mixed.children, child);

        defer array_free(mixed.children);
        measure(b, mixed, operation, borrow, skip_utf8);
    } else if kind == 1 {
        packed: Api.Packed;
        for i: 0..262143 array_add(*packed.numbers, cast(u32) i);
        defer array_free(packed.numbers);
        measure(b, packed, operation, borrow, skip_utf8);
    } else if kind == 2 {
        nodes: [65] Api.Deep;
        for i: 0..63 nodes[i].next = *nodes[i + 1];
        nodes[64].value = "leaf";
        measure(b, nodes[0], operation, borrow, skip_utf8);
    } else if kind == 3 {
        logs: Api.Logs;
        for 0..999 array_add(*logs.entries, .{ time = "2026-11-02T12:00:00Z", level = "INFO", source = "api/request", text = "Request completed successfully with a useful response" });

        defer array_free(logs.entries);

        if operation < 3 measure(b, logs, operation, borrow, skip_utf8);
        else {
            plain := plain_logs(logs);
            defer Json.free_json(*plain);
            measure_json(b, logs, plain, operation - 3);
        }
    } else if kind == 4 {
        packed: Api.Varints;
        for i: 0..262143 array_add(*packed.numbers, cast(u64) i);
        defer array_free(packed.numbers);
        measure(b, packed, operation, borrow, skip_utf8);
    } else {
        spans := spans_workload();

        defer Proto.free(*spans);

        if operation < 3 measure(b, spans, operation, borrow, skip_utf8);
        else {
            plain := plain_spans(spans);
            defer Json.free_json(*plain);
            measure_json(b, spans, plain, operation - 3);
        }
    }
}

mixed_encode :: (b: *Bench) {
    protobuf_case(b, 0, 0, false, false);
} @bench

mixed_size :: (b: *Bench) {
    protobuf_case(b, 0, 2, false, false);
} @bench

mixed_decode_owned_utf8 :: (b: *Bench) {
    protobuf_case(b, 0, 1, false, false);
} @bench

mixed_decode_owned_unchecked :: (b: *Bench) {
    protobuf_case(b, 0, 1, false, true);
} @bench

mixed_decode_borrow_utf8 :: (b: *Bench) {
    protobuf_case(b, 0, 1, true, false);
} @bench

mixed_decode_borrow_unchecked :: (b: *Bench) {
    protobuf_case(b, 0, 1, true, true);
} @bench

packed_1m_encode :: (b: *Bench) {
    protobuf_case(b, 1, 0, false, false);
} @bench

packed_1m_size :: (b: *Bench) {
    protobuf_case(b, 1, 2, false, false);
} @bench

packed_1m_decode_owned_utf8 :: (b: *Bench) {
    protobuf_case(b, 1, 1, false, false);
} @bench

packed_1m_decode_owned_unchecked :: (b: *Bench) {
    protobuf_case(b, 1, 1, false, true);
} @bench

packed_1m_decode_borrow_utf8 :: (b: *Bench) {
    protobuf_case(b, 1, 1, true, false);
} @bench

packed_1m_decode_borrow_unchecked :: (b: *Bench) {
    protobuf_case(b, 1, 1, true, true);
} @bench

deep_64_encode :: (b: *Bench) {
    protobuf_case(b, 2, 0, false, false);
} @bench

deep_64_size :: (b: *Bench) {
    protobuf_case(b, 2, 2, false, false);
} @bench

deep_64_decode_owned_utf8 :: (b: *Bench) {
    protobuf_case(b, 2, 1, false, false);
} @bench

deep_64_decode_owned_unchecked :: (b: *Bench) {
    protobuf_case(b, 2, 1, false, true);
} @bench

deep_64_decode_borrow_utf8 :: (b: *Bench) {
    protobuf_case(b, 2, 1, true, false);
} @bench

deep_64_decode_borrow_unchecked :: (b: *Bench) {
    protobuf_case(b, 2, 1, true, true);
} @bench

string_logs_encode :: (b: *Bench) {
    protobuf_case(b, 3, 0, false, false);
} @bench

string_logs_size :: (b: *Bench) {
    protobuf_case(b, 3, 2, false, false);
} @bench

string_logs_decode_owned_utf8 :: (b: *Bench) {
    protobuf_case(b, 3, 1, false, false);
} @bench

string_logs_decode_owned_unchecked :: (b: *Bench) {
    protobuf_case(b, 3, 1, false, true);
} @bench

string_logs_decode_borrow_utf8 :: (b: *Bench) {
    protobuf_case(b, 3, 1, true, false);
} @bench

string_logs_decode_borrow_unchecked :: (b: *Bench) {
    protobuf_case(b, 3, 1, true, true);
} @bench

string_logs_json_encode :: (b: *Bench) {
    protobuf_case(b, 3, 3, false, false);
} @bench

string_logs_json_decode :: (b: *Bench) {
    protobuf_case(b, 3, 4, false, false);
} @bench

string_logs_plain_json_encode :: (b: *Bench) {
    protobuf_case(b, 3, 5, false, false);
} @bench

string_logs_plain_json_decode :: (b: *Bench) {
    protobuf_case(b, 3, 6, false, false);
} @bench

packed_varints_encode :: (b: *Bench) {
    protobuf_case(b, 4, 0, false, false);
} @bench

packed_varints_size :: (b: *Bench) {
    protobuf_case(b, 4, 2, false, false);
} @bench

packed_varints_decode_owned_utf8 :: (b: *Bench) {
    protobuf_case(b, 4, 1, false, false);
} @bench

packed_varints_decode_owned_unchecked :: (b: *Bench) {
    protobuf_case(b, 4, 1, false, true);
} @bench

packed_varints_decode_borrow_utf8 :: (b: *Bench) {
    protobuf_case(b, 4, 1, true, false);
} @bench

packed_varints_decode_borrow_unchecked :: (b: *Bench) {
    protobuf_case(b, 4, 1, true, true);
} @bench

spans_encode :: (b: *Bench) {
    protobuf_case(b, 5, 0, false, false);
} @bench

spans_size :: (b: *Bench) {
    protobuf_case(b, 5, 2, false, false);
} @bench

spans_decode_owned_utf8 :: (b: *Bench) {
    protobuf_case(b, 5, 1, false, false);
} @bench

spans_decode_owned_unchecked :: (b: *Bench) {
    protobuf_case(b, 5, 1, false, true);
} @bench

spans_decode_borrow_utf8 :: (b: *Bench) {
    protobuf_case(b, 5, 1, true, false);
} @bench

spans_decode_borrow_unchecked :: (b: *Bench) {
    protobuf_case(b, 5, 1, true, true);
} @bench

spans_json_encode :: (b: *Bench) {
    protobuf_case(b, 5, 3, false, false);
} @bench

spans_json_decode :: (b: *Bench) {
    protobuf_case(b, 5, 4, false, false);
} @bench

spans_plain_json_encode :: (b: *Bench) {
    protobuf_case(b, 5, 5, false, false);
} @bench

spans_plain_json_decode :: (b: *Bench) {
    protobuf_case(b, 5, 6, false, false);
} @bench
