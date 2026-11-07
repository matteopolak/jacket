// Canonical gRPC status numbers, including OK and explicit application errors.
Status_Code :: enum u8 {
    OK;
    CANCELLED;
    UNKNOWN;
    INVALID_ARGUMENT;
    DEADLINE_EXCEEDED;
    NOT_FOUND;
    ALREADY_EXISTS;
    PERMISSION_DENIED;
    RESOURCE_EXHAUSTED;
    FAILED_PRECONDITION;
    ABORTED;
    OUT_OF_RANGE;
    UNIMPLEMENTED;
    INTERNAL;
    UNAVAILABLE;
    DATA_LOSS;
    UNAUTHENTICATED;
}

// Locally observed cancellation, deadline, transport/protocol failure or limit,
// recorded separately from the RPC status code.
Local_Cause :: enum u8 {
    NONE;
    CANCELLED;
    DEADLINE;
    TRANSPORT;
    PROTOCOL;
    LIMIT;
}

// Final application code and independently preserved local failure cause.
// Returned messages and trailing metadata are owned; release with free_status.
Status :: struct {
    code: Status_Code = .OK;
    message: string;
    status_details: string; // owned google.rpc.Status wire bytes, if supplied
    metadata: [..] Metadata_Entry;
    cause: Local_Cause;
    http2_code: u32;
    not_processed: bool;
}

// One ordered application metadata entry. Names may repeat; values for names
// ending in -bin contain raw bytes rather than Base64 text.
Metadata_Entry :: struct {
    name: string;
    value: string; // raw bytes for names ending -bin
}

// Parses authoritative final status; HTTP mapping applies only when grpc-status
// is missing or invalid. Repeated status fields are a protocol failure.
// Example: `status := parse_status(fields, 200); defer free_status(*status);`.
parse_status :: (fields: [] Http.Header_Field, http_status: int, limit := 64 * 1024) -> Status {
    if !metadata_headers_fit(fields, limit) return .{ code = .RESOURCE_EXHAUSTED, cause = .LIMIT };

    result: Status;
    result.code = http_status_code(http_status);
    result.cause = .PROTOCOL;
    found := false;

    for fields {
        if it.name == "grpc-status" {
            if found {
                result.code = .UNKNOWN;
                result.cause = .PROTOCOL;
                return result;
            }

            found = true;
            code := parse_code(it.value);
            if code < 0 {
                result.code = .UNKNOWN;
                result.cause = .PROTOCOL;
            } else {
                result.code = cast(Status_Code) code;
                result.cause = .NONE;
            }
        }
    }

    for fields if it.name == "grpc-message" {
        result.message = decode_status_message(it.value);
        break;
    }

    for fields if it.name == "grpc-status-details-bin" {
        if result.status_details.data || !metadata_base64_valid(it.value) {
            result.code = .INTERNAL;
            result.cause = .PROTOCOL;
            break;
        }

        details, decoded := Base64.base64_decode(it.value);
        if !decoded || result.code == .OK || !valid_status_details(details, result.code) {
            if details.data free(details.data);
            result.code = .INTERNAL;
            result.cause = .PROTOCOL;
            break;
        }

        result.status_details = details;
    }

    metadata, valid := decode_metadata(fields, limit);

    result.metadata = metadata;

    if !valid {
        result.code = .INTERNAL;
        result.cause = .PROTOCOL;
    }

    return result;
}

// Releases owned status text and metadata. Safe on a cleared status.
// Example: `free_status(*status);`.
free_status :: (status: *Status) {
    if status.message.data free(status.message.data);
    if status.status_details.data free(status.status_details.data);

    free_metadata(*status.metadata);
    status.* = .{};
}

// Parses grpc-timeout into milliseconds, rounding fractional milliseconds up.
// Wire values have at most eight digits, so every supported unit fits s64.
// Example: `milliseconds, valid := parse_timeout("1S"); // 1000, true`.
parse_timeout :: (text: string) -> milliseconds: s64, ok: bool {
    if text.count < 2 || text.count > 9 return 0, false;

    value: s64;

    for i: 0..text.count - 2 {
        if text[i] < #char "0" || text[i] > #char "9" return 0, false;
        value = value * 10 + cast(s64) (text[i] - #char "0");
    }

    unit := text[text.count - 1];
    if unit == {
        case #char "H"; return value * 3600000, true;
        case #char "M"; return value * 60000, true;
        case #char "S"; return value * 1000, true;
        case #char "m"; return value, true;
        case #char "u"; return (value + 999) / 1000, true;
        case #char "n"; return (value + 999999) / 1000000, true;
    }

    return 0, false;
}

// Maps documented HTTP/2 reset codes. STREAM_CLOSED and unknown codes have no
// open-call mapping and report UNKNOWN. No code authorizes automatic replay.
// Example: `assert(http2_reset_status(8) == .CANCELLED);`.
http2_reset_status :: (code: u32) -> Status_Code {
    mapping :: Status_Code.[.INTERNAL, .INTERNAL, .INTERNAL, .INTERNAL, .INTERNAL, .UNKNOWN, .INTERNAL, .UNAVAILABLE, .CANCELLED, .INTERNAL, .INTERNAL, .RESOURCE_EXHAUSTED, .PERMISSION_DENIED];
    if code >= mapping.count return .UNKNOWN;
    return mapping[code];
}

#scope_module

http_status_code :: (status: int) -> Status_Code {
    if status == {
        case 400; return .INTERNAL;
        case 401; return .UNAUTHENTICATED;
        case 403; return .PERMISSION_DENIED;
        case 404; return .UNIMPLEMENTED;
        case 429; return .UNAVAILABLE;
        case 502; return .UNAVAILABLE;
        case 503; return .UNAVAILABLE;
        case 504; return .UNAVAILABLE;
    }

    return .UNKNOWN;
}

parse_code :: (text: string) -> int {
    if text.count == 0 || text.count > 2 return -1;
    if text.count > 1 && text[0] == #char "0" return -1;

    result := 0;

    for text {
        if it < #char "0" || it > #char "9" return -1;
        result = result * 10 + cast(int) (it - #char "0");
    }

    if result > 16 return -1;

    return result;
}

status_hex :: (c: u8) -> int {
    if c >= #char "0" && c <= #char "9" return c - #char "0";
    if c >= #char "a" && c <= #char "f" return c - #char "a" + 10;
    if c >= #char "A" && c <= #char "F" return c - #char "A" + 10;
    return -1;
}

decode_status_message :: (text: string) -> string {
    out := alloc_string(text.count);
    at := 0;
    i := 0;

    while i < text.count {
        c := text[i];
        if c == #char "%" && i + 2 < text.count && status_hex(text[i + 1]) >= 0 && status_hex(text[i + 2]) >= 0 {
            c = cast(u8) (status_hex(text[i + 1]) * 16 + status_hex(text[i + 2]));
            i += 2;
        }

        out[at] = c;
        at += 1;
        i += 1;
    }

    out.count = at;

    return out;
}

// The shared Protobuf reader handles field framing and limits; only the optional
// google.rpc.Status code field is interpreted here to check its required agreement.
valid_status_details :: (bytes: string, code: Status_Code) -> bool {
    reader := Proto.Reader.{ bytes = .{ bytes.count, bytes.data }, options = .{ max_size = 65536, max_depth = 16 } };
    found := false;
    value: u64;

    while reader.pos < bytes.count && reader.error.kind == .NONE {
        tag := Proto.read_tag(*reader);
        if tag == 8 {
            value = Proto.read_varint(*reader);
            found = true;
        } else if !Proto.skip_unknown(*reader, tag) break;
    }

    valid := reader.error.kind == .NONE && (!found || cast,trunc(u32) value == cast(u32) code);

    Proto.free(*reader.error);

    return valid;
}
