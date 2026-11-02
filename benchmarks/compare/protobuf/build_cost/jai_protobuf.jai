// jai-protobuf: the Spans schema generated at compile time (`#insert #run`), one batch encoded and decoded.
#import "Basic";
#import "segcore--jai-protobuf";

#insert #run generate_jai(tprint("%/spans.proto", #filepath));

main :: () {
    batch: Spans;
    trace: [16] u8;
    id: [8] u8;

    for i: 0..7 {
        span: Span;
        span.trace_id = trace;
        span.span_id = id;
        span.name = "db.query";
        span.kind = .KIND_SERVER;
        span.start_ns = 1_700_000_000_000_000_000 + i;
        span.end_ns = span.start_ns + 100;
        array_add(*span.attributes, .{ key = "db.system", value = "postgres" });
        array_add(*span.attributes, .{ key = "retry.count", number = 3 });
        array_add(*span.events, .{ time_ns = span.start_ns + 5, name = "retry" });
        array_add(*batch.spans, span);
    }

    bytes := Serialize(batch);
    decoded: Spans;
    ok := Deserialize(*decoded, bytes);

    print("bytes=% spans=% ok=%\n", bytes.count, decoded.spans.count, ok);
}
