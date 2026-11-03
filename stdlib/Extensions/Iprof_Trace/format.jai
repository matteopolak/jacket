// Writes JIPT v1: eight-byte magic, little-endian u64 name/frame counts, length-prefixed UTF-8
// names, then frames (duration, span count) and spans (zone, parent+1, start, end). Times are ns.
// Refuses invalid trees and files over 64 MiB. The caller owns the trace.
save_trace :: (trace: *Trace, path: string) -> bool {
    size: s64 = 24;

    for trace.names {
        if it.count > MAX_BYTES - size - 8 return false;
        size += 8 + it.count;
    }

    for trace.frames {
        if it.spans.count > (MAX_BYTES - size - 16) / 32 return false;
        size += 16 + it.spans.count * 32;
    }

    if size > MAX_BYTES return false;

    bytes: [..] u8;

    defer array_free(bytes);
    for MAGIC array_add(*bytes, it);

    put(*bytes, cast(u64) trace.names.count);
    put(*bytes, cast(u64) trace.frames.count);

    for trace.names {
        put(*bytes, cast(u64) it.count);
        for it array_add(*bytes, it);
    }

    for *trace.frames {
        if !valid_frame(it, trace.names.count) return false;

        put(*bytes, it.duration);
        put(*bytes, cast(u64) it.spans.count);

        for it.spans {
            put(*bytes, cast(u64) it.zone);
            put(*bytes, cast(u64) (it.parent + 1));
            put(*bytes, it.start);
            put(*bytes, it.end);
        }
    }

    if bytes.count > MAX_BYTES return false;

    return write_entire_file(path, bytes.data, bytes.count);
}

// Loads a bounded JIPT v1 file, validating lengths, indices, containment and sibling order.
// Returns an empty trace on failure. The successful result owns its storage: use free_trace.
//
// ```jai,no_run
// trace, ok := load_trace("work.jipt");
// if ok {
//     assert(trace.frames.count > 0);
//     free_trace(*trace);
// }
// ```
load_trace :: (path: string) -> Trace, bool {
    file, opened := file_open(path);
    if !opened return .{}, false;

    defer file_close(*file);
    length, measured := file_length(file);
    if !measured || length < 24 || length > MAX_BYTES return .{}, false;

    bytes := NewArray(length, u8);

    defer array_free(bytes);
    ok, count := file_read(file, bytes.data, length);
    if !ok || count != length return .{}, false;

    return decode_trace(string.{ bytes.count, bytes.data });
}

// Decodes bytes with the same validation as load_trace; the result never borrows input memory.
decode_trace :: (bytes: string) -> Trace, bool {
    trace: Trace;
    success := false;

    defer if !success free_trace(*trace);
    if bytes.count < 24 || bytes.count > MAX_BYTES return .{}, false;
    for MAGIC if bytes[it_index] != it return .{}, false;

    reader := Reader.{ bytes = bytes, at = 8 };
    names := get(*reader);
    frames := get(*reader);

    if names > cast(u64) bytes.count / 8 || frames > cast(u64) bytes.count / 16 return .{}, false;

    for 0..cast(s64) names - 1 {
        count := get(*reader);
        if reader.bad || count > cast(u64) (bytes.count - reader.at) return .{}, false;
        array_add(*trace.names, copy_string(slice(bytes, reader.at, cast(s64) count)));
        reader.at += cast(s64) count;
    }

    for 0..cast(s64) frames - 1 {
        duration := get(*reader);
        count := get(*reader);

        if reader.bad || count > cast(u64) (bytes.count - reader.at) / 32 return .{}, false;

        frame: Frame;
        frame.duration = duration;
        array_add(*trace.frames, frame);
        current := *trace.frames[trace.frames.count - 1];

        for 0..cast(s64) count - 1 {
            zone := get(*reader);
            parent := get(*reader);
            start := get(*reader);
            end := get(*reader);

            if zone >= names || parent > cast(u64) current.spans.count return .{}, false;

            array_add(*current.spans, .{ cast(s64) zone, cast(s64) parent - 1, start, end });
        }

        if !valid_frame(current, trace.names.count) return .{}, false;
    }

    if reader.bad || reader.at != bytes.count return .{}, false;

    success = true;

    return trace, true;
}

#scope_file
#import "File";
#import "String";

MAGIC :: "JIPT\x01\x00\x00\x00";
MAX_BYTES :: 64 * 1024 * 1024;

Reader :: struct {
    bytes: string;
    at: s64;
    bad: bool;
}

put :: (bytes: *[..] u8, value: u64) {
    for shift: 0..7 array_add(bytes, cast(u8) ((value >> (shift * 8)) & 255));
}

get :: (reader: *Reader) -> u64 {
    if reader.at > reader.bytes.count - 8 {
        reader.bad = true;
        return 0;
    }

    result: u64;
    for shift: 0..7 result |= (cast(u64) reader.bytes[reader.at + shift]) << (shift * 8);

    reader.at += 8;

    return result;
}

valid_frame :: (frame: *Frame, names: s64) -> bool {
    stack: [..] s64;

    defer array_free(stack);
    last_end: u64;
    total: u64;
    limit: u64 = 0x7fffffffffffffff;

    if frame.duration > limit return false;

    for span, index: frame.spans {
        if span.zone < 0 || span.zone >= names || span.parent < -1 || span.parent >= index return false;
        if span.start > span.end || span.end > frame.duration return false;

        elapsed := span.end - span.start;
        if elapsed > limit - total return false;

        total += elapsed;

        while stack.count > 0 && stack[stack.count - 1] != span.parent {
            last_end = frame.spans[stack[stack.count - 1]].end;
            stack.count -= 1;
        }

        if span.parent >= 0 {
            if stack.count == 0 return false;
            parent := frame.spans[span.parent];
            if span.start < parent.start || span.end > parent.end return false;
        }

        if span.start < last_end return false;

        last_end = span.start;
        array_add(*stack, index);
    }

    return true;
}
