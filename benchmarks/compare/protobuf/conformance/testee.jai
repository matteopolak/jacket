// Binary conformance protocol. Supply PROTOBUF_ROOT with jaic -add at build time.
#import "Basic";
Proto :: #import "Extensions/Protobuf";
Protocol :: #import "Extensions/Protobuf/Schema"(PATH = #run tprint("%/conformance/conformance.proto", PROTOBUF_ROOT));
P2 :: #import "Extensions/Protobuf/Schema"(PATH = #run tprint("%/src/google/protobuf/test_messages_proto2.proto", PROTOBUF_ROOT));
P3 :: #import "Extensions/Protobuf/Schema"(PATH = #run tprint("%/src/google/protobuf/test_messages_proto3.proto", PROTOBUF_ROOT));
E2 :: #import "Extensions/Protobuf/Schema"(PATH = #run tprint("%/editions/golden/test_messages_proto2_editions.proto", PROTOBUF_ROOT));
E3 :: #import "Extensions/Protobuf/Schema"(PATH = #run tprint("%/editions/golden/test_messages_proto3_editions.proto", PROTOBUF_ROOT));
E23 :: #import "Extensions/Protobuf/Schema"(PATH = #run tprint("%/conformance/test_protos/test_messages_edition2023.proto", PROTOBUF_ROOT));

C :: Protocol.conformance;

libc :: #system_library "c";
read_fd :: (fd: s32, data: *u8, count: u64) -> s64 #foreign libc "read";
write_fd :: (fd: s32, data: *u8, count: u64) -> s64 #foreign libc "write";

read_exact :: (bytes: [] u8) -> bool {
    pos: s64;
    while pos < bytes.count {
        n := read_fd(0, bytes.data + pos, cast(u64) (bytes.count - pos));
        if n <= 0 return false;
        pos += n;
    }

    return true;
}

write_exact :: (bytes: [] u8) -> bool {
    pos: s64;
    while pos < bytes.count {
        n := write_fd(1, bytes.data + pos, cast(u64) (bytes.count - pos));
        if n <= 0 return false;
        pos += n;
    }

    return true;
}

transcode :: ($T: Type, bytes: [] u8) -> C.ConformanceResponse {
    response: C.ConformanceResponse;
    value, error := Proto.decode(T, bytes);

    defer Proto.free(*value);
    defer Proto.free(*error);

    if error.kind != .NONE {
        response.result.which = .PARSE_ERROR;
        response.result.parse_error = sprint("protobuf error % at %: %", error.kind, error.offset, error.path);
        return response;
    }

    encoded, ee := Proto.encode(value);

    defer Proto.free(*ee);

    if ee.kind != .NONE {
        response.result.which = .SERIALIZE_ERROR;
        response.result.serialize_error = sprint("protobuf encode error %: %", ee.kind, ee.path);
    } else {
        response.result.which = .PROTOBUF_PAYLOAD;
        response.result.protobuf_payload = encoded;
    }

    return response;
}

respond :: (request: C.ConformanceRequest) -> C.ConformanceResponse {
    result: C.ConformanceResponse;

    if request.message_type == "conformance.FailureSet" {
        result.result.which = .PROTOBUF_PAYLOAD;
        return result;
    }

    if request.payload.which != .PROTOBUF_PAYLOAD || request.requested_output_format != .PROTOBUF {
        result.result.which = .SKIPPED;
        result.result.skipped = copy_string("JSON, text and JSPB formats are out of scope");
        return result;
    }

    bytes := request.payload.protobuf_payload;
    if request.message_type == "protobuf_test_messages.proto2.TestAllTypesProto2" return transcode(P2.protobuf_test_messages.proto2.TestAllTypesProto2, bytes);
    if request.message_type == "protobuf_test_messages.proto3.TestAllTypesProto3" return transcode(P3.protobuf_test_messages.proto3.TestAllTypesProto3, bytes);
    if request.message_type == "protobuf_test_messages.editions.proto2.TestAllTypesProto2" return transcode(E2.protobuf_test_messages.editions.proto2.TestAllTypesProto2, bytes);
    if request.message_type == "protobuf_test_messages.editions.proto3.TestAllTypesProto3" return transcode(E3.protobuf_test_messages.editions.proto3.TestAllTypesProto3, bytes);
    if request.message_type == "protobuf_test_messages.editions.TestAllTypesEdition2023" return transcode(E23.protobuf_test_messages.editions.TestAllTypesEdition2023, bytes);

    result.result.which = .SKIPPED;
    result.result.skipped = sprint("unsupported message/edition: %", request.message_type);

    return result;
}

main :: () {
    while true {
        header: [4] u8;
        if !read_exact(header) break;

        count: u32;
        for header count |= (cast(u32) it) << cast(u32) (8 * it_index);
        if count > 67108864 break; // Bound framing independently of the protobuf decoder.

        bytes := NewArray(cast(s64) count, u8);

        defer free(bytes.data);
        if !read_exact(bytes) break;

        request, error := Proto.decode(C.ConformanceRequest, bytes);

        defer Proto.free(*request);
        defer Proto.free(*error);
        response: C.ConformanceResponse;

        if error.kind != .NONE {
            response.result.which = .RUNTIME_ERROR;
            response.result.runtime_error = copy_string("invalid ConformanceRequest");
        } else response = respond(request);

        defer Proto.free(*response);
        encoded, ee := Proto.encode(response);

        defer free(encoded.data);
        defer Proto.free(*ee);
        if ee.kind != .NONE break;
        for *header it.* = cast(u8) ((cast(u64) encoded.count >> cast(u64) (8 * it_index)) & 255);
        if !write_exact(header) || !write_exact(encoded) break;
    }
}
