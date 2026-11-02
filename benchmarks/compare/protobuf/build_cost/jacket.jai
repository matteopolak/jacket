// Extensions/Protobuf: the Spans schema generated at compile time, one batch encoded and decoded.
#import "Basic";

Proto :: #import "Extensions/Protobuf";
Api :: #import "Extensions/Protobuf/Schema"(PATH = #run Proto.beside("spans.proto"));

main :: () {
    batch: Api.Spans;
    trace: [16] u8;
    id: [8] u8;

    for i: 0..7 {
        span: Api.Span;
        span.trace_id = trace;
        span.span_id = id;
        span.name = "db.query";
        span.kind = .KIND_SERVER;
        span.start_ns = 1_700_000_000_000_000_000 + cast(u64) i;
        span.end_ns = span.start_ns + 100;
        array_add(*span.attributes, .{ key = "db.system", value = "postgres" });
        array_add(*span.attributes, .{ key = "retry.count", number = 3 });
        array_add(*span.events, .{ time_ns = span.start_ns + 5, name = "retry" });
        array_add(*batch.spans, span);
    }

    bytes, error := Proto.encode(batch);
    decoded, decode_error := Proto.decode(Api.Spans, bytes);

    print("bytes=% spans=% ok=%\n", bytes.count, decoded.spans.count, error.kind == .NONE && decode_error.kind == .NONE);
}
