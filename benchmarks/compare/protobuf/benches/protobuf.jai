// Extensions/Protobuf against segcore/jai-protobuf on five shapes: a mixed message, 1 MiB of packed
// fixed32, 64 levels of nesting, 1,000 string-heavy log lines and a batch of 2,000 tracing spans
// (the realistic one; docs/extensions/protobuf.md). Encode and decode are measured separately.
//
// Both libraries get the same schema (proto/shapes.proto, and proto/shapes_flat.proto for
// jai-protobuf) and the same values. Before anything is timed, `prepare` checks that the two agree:
// each library's bytes must decode in the other into a value that encodes back to the same bytes.
//
// jai-protobuf keeps a message field by value, so it has no Deep (a message containing itself does
// not compile there): its decode_deep and encode_deep are left out and show as n/a.
//
// Each iteration allocates from a bump pool that is emptied after the call, for every library alike.
// jai-protobuf's decoder points its strings and bytes into the input, so `borrow` is the Jacket
// decode that does the same. Allocations per call are counted in a pass of their own before the
// timer, and written to `allocs-NAME.tsv` (the check's result to `notes.txt`) in the benchmark folder
// for tools/bench/library_compare.py to collect: the runner drops what a benchmark prints.
//
// The community library is imported from corpus/upstream through `-import_dir` (jp/shapes.jai).
// Run through tools/bench/library_compare.py, which keeps it inside the sandbox.
#import "Basic";
#import "String";
#import "Flat_Pool";
#import "File";
#import "Extensions/Testing";

Proto :: #import "Extensions/Protobuf";
Api :: #import "Extensions/Protobuf/Schema"(PATH = #run Proto.beside("../proto/shapes.proto"));
jp :: #import,file "../jp/shapes.jai";

// ---------------------------------------------------------------------------------- the data

pool: Flat_Pool;
prepared := false;
counted := false;

// Our values, jai-protobuf's values, and the bytes each library writes.
our_mixed: Api.Mixed;
our_child: Api.Leaf;
our_packed: Api.Packed;
our_deep: [65] Api.Deep;
our_logs: Api.Logs;
our_spans: Api.Spans;

jp_mixed: jp.Mixed;
jp_packed: jp.Packed;
jp_logs: jp.Logs;
jp_spans: jp.Spans;

our_mixed_bytes: [] u8;
our_packed_bytes: [] u8;
our_deep_bytes: [] u8;
our_logs_bytes: [] u8;
our_spans_bytes: [] u8;

jp_mixed_bytes: [] u8;
jp_packed_bytes: [] u8;
jp_logs_bytes: [] u8;
jp_spans_bytes: [] u8;

// ---------------------------------------------------------------------------------- the Spans generator

// The workload of the perf round, bytes for bytes: xorshift64* from a fixed seed.
rng_state: u64 = 0x9E3779B97F4A7C15;

next :: () -> u64 {
    x := rng_state;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    rng_state = x;

    return x * 0x2545F4914F6CDD1D;
}

below :: (n: u64) -> u64 {
    return next() % n;
}

SPAN_NAMES :: string.["GET /api/users/{id}", "POST /api/orders", "GET /api/orders/{id}", "db.query", "cache.get", "cache.set", "render.page", "auth.verify", "queue.publish", "queue.consume", "GET /healthz", "PUT /api/users/{id}"];
SPAN_KEYS :: string.["http.method", "http.status_code", "http.route", "db.system", "db.statement", "net.peer.name", "net.peer.port", "user.id", "retry.count", "error.type"];
EVENT_NAMES :: string.["exception", "retry", "cache.miss", "message"];

// The same batch in neutral types, converted for each library below.
Plan_Attribute :: struct {
    key: string;
    value: string;
    number: s64;
}

Plan_Event :: struct {
    time_ns: u64;
    name: string;
    attributes: [] Plan_Attribute;
}

Plan_Span :: struct {
    trace_id: [] u8;
    span_id: [] u8;
    parent_id: [] u8;
    name: string;
    kind: s64;
    start_ns: u64;
    end_ns: u64;
    attributes: [] Plan_Attribute;
    events: [] Plan_Event;
    status: u32;
}

plan_text :: () -> string {
    length := 4 + cast(s64) below(21);
    text := alloc_string(length);

    for 0..length - 1 text[it] = cast(u8) (97 + below(26));

    return text;
}

plan_attribute :: () -> Plan_Attribute {
    result: Plan_Attribute;
    result.key = SPAN_KEYS[below(10)];

    if below(3) == 0 {
        result.number = cast(s64) below(100000);
    } else {
        result.value = plan_text();
    }

    return result;
}

plan_bytes :: (count: s64) -> [] u8 {
    bytes: [] u8 = NewArray(count, u8);

    for 0..count - 1 bytes[it] = cast(u8) (next() & 0xFF);

    return bytes;
}

plan_spans :: () -> [] Plan_Span {
    spans: [] Plan_Span = NewArray(2000, Plan_Span);
    trace: [] u8;

    for i: 0..1999 {
        span := *spans[i];

        if i % 8 == 0 trace = plan_bytes(16);

        span.trace_id = trace;
        span.span_id = plan_bytes(8);

        if i % 8 != 0 span.parent_id = spans[i - 1].span_id;

        span.name = SPAN_NAMES[below(12)];
        span.kind = i % 4;
        span.start_ns = 1_700_000_000_000_000_000 + cast(u64) i * 1000 + below(1000);
        span.end_ns = span.start_ns + below(5_000_000);

        attributes: [] Plan_Attribute = NewArray(3 + cast(s64) below(6), Plan_Attribute);

        for *attributes it.* = plan_attribute();

        span.attributes = attributes;

        events: [] Plan_Event = NewArray(cast(s64) below(3), Plan_Event);

        for *event: events {
            event.time_ns = span.start_ns + below(1_000_000);
            event.name = EVENT_NAMES[below(4)];

            event_attributes: [] Plan_Attribute = NewArray(1 + cast(s64) below(2), Plan_Attribute);

            for *event_attributes it.* = plan_attribute();

            event.attributes = event_attributes;
        }

        span.events = events;
        span.status = ifx below(10) == 0 then cast(u32) 2 else cast(u32) 0;
    }

    return spans;
}

// ---------------------------------------------------------------------------------- building the values

our_attributes :: (plan: [] Plan_Attribute) -> [..] Api.Attribute {
    result: [..] Api.Attribute;

    for plan array_add(*result, .{ key = it.key, value = it.value, number = it.number });

    return result;
}

jp_attributes :: (plan: [] Plan_Attribute) -> [..] jp.Attribute {
    result: [..] jp.Attribute;

    for plan array_add(*result, .{ key = it.key, value = it.value, number = it.number });

    return result;
}

build_our_spans :: (plan: [] Plan_Span) {
    for span: plan {
        out: Api.Span;
        out.trace_id = span.trace_id;
        out.span_id = span.span_id;
        out.parent_id = span.parent_id;
        out.name = span.name;
        out.kind = xx span.kind;
        out.start_ns = span.start_ns;
        out.end_ns = span.end_ns;
        out.attributes = our_attributes(span.attributes);
        out.status = span.status;

        for event: span.events {
            array_add(*out.events, .{ time_ns = event.time_ns, name = event.name, attributes = our_attributes(event.attributes) });
        }

        array_add(*our_spans.spans, out);
    }
}

build_jp_spans :: (plan: [] Plan_Span) {
    for span: plan {
        out: jp.Span;
        out.trace_id = span.trace_id;
        out.span_id = span.span_id;
        out.parent_id = span.parent_id;
        out.name = span.name;
        out.kind = xx span.kind;
        out.start_ns = xx span.start_ns;
        out.end_ns = xx span.end_ns;
        out.attributes = jp_attributes(span.attributes);
        out.status = span.status;

        for event: span.events {
            array_add(*out.events, .{ time_ns = xx event.time_ns, name = event.name, attributes = jp_attributes(event.attributes) });
        }

        array_add(*jp_spans.spans, out);
    }
}

LOG_TIME :: "2026-11-02T12:00:00Z";
LOG_TEXT :: "Request completed successfully with a useful response";

// ---------------------------------------------------------------------------------- the agreement check

fail :: (message: string, name: string) {
    print("the libraries disagree on %: %\n", name, message, to_standard_error = true);
    exit(1);
}

// Ours into jai-protobuf and back, and the other way round. Each decode must succeed, and writing
// the decoded value must give the original bytes. `same_bytes` says whether the two encoders wrote
// the same bytes in the first place (expected: both write fields in number order).
agree :: (notes: *String_Builder, name: string, value: $A, other: $B, ours: [] u8, theirs: [] u8) {
    if ours.count != theirs.count fail(tprint("encoded sizes differ: % and %", ours.count, theirs.count), name);

    same_bytes := memcmp(ours.data, theirs.data, ours.count) == 0;

    // jai-protobuf reads what we wrote.
    read: B;
    ok, used := jp.Deserialize(*read, ours);

    if !ok || used != ours.count fail("jai-protobuf cannot read our bytes", name);

    again := jp.Serialize(read);
    if again.count != ours.count || memcmp(again.data, ours.data, ours.count) != 0 fail("jai-protobuf does not write back what it read from our bytes", name);

    // We read what jai-protobuf wrote.
    decoded, error := Proto.decode(A, theirs);
    if error.kind != .NONE fail("we cannot read the bytes of jai-protobuf", name);

    if !Proto.equal(decoded, value) fail("the value we read from jai-protobuf's bytes differs from the original", name);

    back, write_error := Proto.encode(decoded);
    if write_error.kind != .NONE || back.count != theirs.count || memcmp(back.data, theirs.data, theirs.count) != 0 fail("we do not write back what we read from jai-protobuf's bytes", name);

    print(notes, "agree % bytes=% same_bytes=%\n", name, ours.count, same_bytes);
}

prepare :: () {
    if prepared return;

    init(*pool, 1024 * 1024 * 1024);

    // Mixed
    our_child = .{ x = 42, label = "nested" };
    our_mixed = .{ id = 123456, name = "an API response", child = *our_child, active = true };

    jp_child: jp.Leaf = .{ x = 42, label = "nested" };

    jp_mixed = .{ id = 123456, name = "an API response", active = true, child = jp_child };

    for 0..3 {
        array_add(*our_mixed.children, our_child);
        array_add(*jp_mixed.children, jp_child);
    }

    // Packed
    for i: 0..262143 {
        array_add(*our_packed.numbers, cast(u32) i);
        array_add(*jp_packed.numbers, cast(s32) i);
    }

    // Deep
    for i: 0..63 our_deep[i].next = *our_deep[i + 1];

    our_deep[64].value = "leaf";

    // Logs
    for 0..999 {
        array_add(*our_logs.entries, .{ time = LOG_TIME, level = "INFO", source = "api/request", text = LOG_TEXT });
        array_add(*jp_logs.entries, .{ time = LOG_TIME, level = "INFO", source = "api/request", text = LOG_TEXT });
    }

    // Spans
    plan := plan_spans();

    build_our_spans(plan);
    build_jp_spans(plan);

    // The bytes of each library.
    error: Proto.Error;
    our_mixed_bytes, error = Proto.encode(our_mixed);
    assert(error.kind == .NONE);
    our_packed_bytes, error = Proto.encode(our_packed);
    assert(error.kind == .NONE);
    our_deep_bytes, error = Proto.encode(our_deep[0]);
    assert(error.kind == .NONE);
    our_logs_bytes, error = Proto.encode(our_logs);
    assert(error.kind == .NONE);
    our_spans_bytes, error = Proto.encode(our_spans);
    assert(error.kind == .NONE);

    jp_mixed_bytes = jp.Serialize(jp_mixed);
    jp_packed_bytes = jp.Serialize(jp_packed);
    jp_logs_bytes = jp.Serialize(jp_logs);
    jp_spans_bytes = jp.Serialize(jp_spans);

    notes: String_Builder;
    notes.allocator = temp;
    print(*notes, "spans encoded bytes: %\n", our_spans_bytes.count);
    agree(*notes, "mixed", our_mixed, jp_mixed, our_mixed_bytes, jp_mixed_bytes);
    agree(*notes, "packed", our_packed, jp_packed, our_packed_bytes, jp_packed_bytes);
    agree(*notes, "logs", our_logs, jp_logs, our_logs_bytes, jp_logs_bytes);
    agree(*notes, "spans", our_spans, jp_spans, our_spans_bytes, jp_spans_bytes);
    print(*notes, "deep encoded bytes: % (jai-protobuf has no recursive messages)\n", our_deep_bytes.count);
    write_entire_file("notes.txt", builder_to_string(*notes));

    prepared = true;
}

// ---------------------------------------------------------------------------------- the loops

Counting :: struct {
    allocations: s64;
    bytes: s64;
}

counting_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old: *void, data: *void) -> *void {
    c := cast(*Counting) data;

    if mode == .ALLOCATE || mode == .RESIZE {
        c.allocations += 1;
        c.bytes += size;
    }

    return flat_pool_allocator_proc(mode, size, old_size, old, *pool);
}

// One call of `work` under a counting allocator, outside the timer; once per process.
count_allocations :: (name: string, work: ()) {
    if counted return;

    counted = true;
    c: Counting;

    {
        push_allocator(counting_proc, *c);
        work();
        reset(*pool);
    }

    write_entire_file(tprint("allocs-%.tsv", name), tprint("%\t%\t%\n", name, c.allocations, c.bytes));
}

// The loop of every benchmark: `work` takes nothing, with the pool emptied after each call.
looped :: (b: *Bench, name: string, bytes: int, work: ()) {
    set_bytes(b, bytes);
    count_allocations(name, work);
    reset_timer(b);

    for 0..b.iterations - 1 {
        push_allocator(flat_pool_allocator_proc, *pool);
        work();
        reset(*pool);
    }
}

// A library that fails on the benchmark input would make its benchmark measure the failure path.
require :: (ok: bool) {
    if !ok {
        print("the library rejected the benchmark input\n", to_standard_error = true);
        exit(1);
    }
}

// ---------------------------------------------------------------------------------- Jacket

encode_mixed__jacket :: (b: *Bench) {
    prepare();
    looped(b, "encode_mixed__jacket", our_mixed_bytes.count, () {
        bytes, error := Proto.encode(black_box(our_mixed));
        black_box(bytes);
        require(error.kind == .NONE);
    });
} @bench

decode_mixed__jacket :: (b: *Bench) {
    prepare();
    looped(b, "decode_mixed__jacket", our_mixed_bytes.count, () {
        value, error := Proto.decode(Api.Mixed, black_box(our_mixed_bytes));
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

encode_packed__jacket :: (b: *Bench) {
    prepare();
    looped(b, "encode_packed__jacket", our_packed_bytes.count, () {
        bytes, error := Proto.encode(black_box(our_packed));
        black_box(bytes);
        require(error.kind == .NONE);
    });
} @bench

decode_packed__jacket :: (b: *Bench) {
    prepare();
    looped(b, "decode_packed__jacket", our_packed_bytes.count, () {
        value, error := Proto.decode(Api.Packed, black_box(our_packed_bytes));
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

encode_deep__jacket :: (b: *Bench) {
    prepare();
    looped(b, "encode_deep__jacket", our_deep_bytes.count, () {
        bytes, error := Proto.encode(black_box(our_deep[0]));
        black_box(bytes);
        require(error.kind == .NONE);
    });
} @bench

decode_deep__jacket :: (b: *Bench) {
    prepare();
    looped(b, "decode_deep__jacket", our_deep_bytes.count, () {
        value, error := Proto.decode(Api.Deep, black_box(our_deep_bytes));
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

encode_logs__jacket :: (b: *Bench) {
    prepare();
    looped(b, "encode_logs__jacket", our_logs_bytes.count, () {
        bytes, error := Proto.encode(black_box(our_logs));
        black_box(bytes);
        require(error.kind == .NONE);
    });
} @bench

decode_logs__jacket :: (b: *Bench) {
    prepare();
    looped(b, "decode_logs__jacket", our_logs_bytes.count, () {
        value, error := Proto.decode(Api.Logs, black_box(our_logs_bytes));
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

encode_spans__jacket :: (b: *Bench) {
    prepare();
    looped(b, "encode_spans__jacket", our_spans_bytes.count, () {
        bytes, error := Proto.encode(black_box(our_spans));
        black_box(bytes);
        require(error.kind == .NONE);
    });
} @bench

decode_spans__jacket :: (b: *Bench) {
    prepare();
    looped(b, "decode_spans__jacket", our_spans_bytes.count, () {
        value, error := Proto.decode(Api.Spans, black_box(our_spans_bytes));
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

// ---------------------------------------------------------------------------------- jacket, borrow
// The decodes with `borrow`: strings and bytes are slices of the input, as in jai-protobuf. The encoders
// do not differ, so there are none here.

decode_mixed__borrow :: (b: *Bench) {
    prepare();
    looped(b, "decode_mixed__borrow", our_mixed_bytes.count, () {
        value, error := Proto.decode(Api.Mixed, black_box(our_mixed_bytes), .{ borrow = true });
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

decode_packed__borrow :: (b: *Bench) {
    prepare();
    looped(b, "decode_packed__borrow", our_packed_bytes.count, () {
        value, error := Proto.decode(Api.Packed, black_box(our_packed_bytes), .{ borrow = true });
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

decode_deep__borrow :: (b: *Bench) {
    prepare();
    looped(b, "decode_deep__borrow", our_deep_bytes.count, () {
        value, error := Proto.decode(Api.Deep, black_box(our_deep_bytes), .{ borrow = true });
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

decode_logs__borrow :: (b: *Bench) {
    prepare();
    looped(b, "decode_logs__borrow", our_logs_bytes.count, () {
        value, error := Proto.decode(Api.Logs, black_box(our_logs_bytes), .{ borrow = true });
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

decode_spans__borrow :: (b: *Bench) {
    prepare();
    looped(b, "decode_spans__borrow", our_spans_bytes.count, () {
        value, error := Proto.decode(Api.Spans, black_box(our_spans_bytes), .{ borrow = true });
        black_box(value);
        require(error.kind == .NONE);
    });
} @bench

// ---------------------------------------------------------------------------------- jai-protobuf
// Deep is missing: a message that contains itself does not compile (fields are stored by value).

encode_mixed__jai_protobuf :: (b: *Bench) {
    prepare();
    looped(b, "encode_mixed__jai_protobuf", jp_mixed_bytes.count, () {
        black_box(jp.Serialize(black_box(jp_mixed)));
    });
} @bench

decode_mixed__jai_protobuf :: (b: *Bench) {
    prepare();
    looped(b, "decode_mixed__jai_protobuf", jp_mixed_bytes.count, () {
        value: jp.Mixed;
        ok, used := jp.Deserialize(*value, black_box(jp_mixed_bytes));
        black_box(value);
        require(ok);
    });
} @bench

encode_packed__jai_protobuf :: (b: *Bench) {
    prepare();
    looped(b, "encode_packed__jai_protobuf", jp_packed_bytes.count, () {
        black_box(jp.Serialize(black_box(jp_packed)));
    });
} @bench

decode_packed__jai_protobuf :: (b: *Bench) {
    prepare();
    looped(b, "decode_packed__jai_protobuf", jp_packed_bytes.count, () {
        value: jp.Packed;
        ok, used := jp.Deserialize(*value, black_box(jp_packed_bytes));
        black_box(value);
        require(ok);
    });
} @bench

encode_logs__jai_protobuf :: (b: *Bench) {
    prepare();
    looped(b, "encode_logs__jai_protobuf", jp_logs_bytes.count, () {
        black_box(jp.Serialize(black_box(jp_logs)));
    });
} @bench

decode_logs__jai_protobuf :: (b: *Bench) {
    prepare();
    looped(b, "decode_logs__jai_protobuf", jp_logs_bytes.count, () {
        value: jp.Logs;
        ok, used := jp.Deserialize(*value, black_box(jp_logs_bytes));
        black_box(value);
        require(ok);
    });
} @bench

encode_spans__jai_protobuf :: (b: *Bench) {
    prepare();
    looped(b, "encode_spans__jai_protobuf", jp_spans_bytes.count, () {
        black_box(jp.Serialize(black_box(jp_spans)));
    });
} @bench

decode_spans__jai_protobuf :: (b: *Bench) {
    prepare();
    looped(b, "decode_spans__jai_protobuf", jp_spans_bytes.count, () {
        value: jp.Spans;
        ok, used := jp.Deserialize(*value, black_box(jp_spans_bytes));
        black_box(value);
        require(ok);
    });
} @bench
