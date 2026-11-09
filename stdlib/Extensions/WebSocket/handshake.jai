// Computes RFC 6455's allocated accept value. Example: accept := ws_accept_key(key); free(accept).
ws_accept_key :: (key: string) -> string {
    input := alloc_string(key.count + WS_GUID.count);

    defer free(input);
    if key.count memcpy(input.data, key.data, key.count);

    memcpy(input.data + key.count, WS_GUID.data, WS_GUID.count);
    digest := Sha.sha1(input);

    return Base64.base64_encode(string.{ digest.count, digest.data });
}

// Negotiates a bounded extension header; response=true forbids offers without a window value.
// Example: compression, error := ws_negotiate("permessage-deflate; client_max_window_bits").
ws_negotiate :: (value: string, response := false) -> Ws_Deflate, Ws_Error {
    if value.count > 8192 return .{}, .{ .LIMIT, "extension header too large", 0 };

    remaining := value;
    result: Ws_Deflate;
    offset := 0;

    while remaining.count {
        more, offer, rest := cut(remaining, #char ",");
        if more && trim(rest, " \t").count == 0 return .{}, .{ .HANDSHAKE, "empty extension", offset + offer.count };

        has_params, name, params := cut(offer, #char ";");
        if has_params && trim(params, " \t").count == 0 return .{}, .{ .HANDSHAKE, "empty extension parameter", offset + offer.count };

        if trim(name, " \t") != "permessage-deflate" {
            if response return .{}, .{ .HANDSHAKE, "unsolicited extension", offset };
            remaining = rest;
            offset = value.count - rest.count;
            continue;
        }

        if result.enabled return .{}, .{ .HANDSHAKE, "duplicate extension", offset };

        result.enabled = true;
        seen: u8;

        while params.count {
            more_params, param, after := cut(params, #char ";");
            if more_params && trim(after, " \t").count == 0 return .{}, .{ .HANDSHAKE, "empty extension parameter", offset + offer.count };

            has_value, key, val := cut(trim(param, " \t"), #char "=");
            key = trim(key, " \t");
            val = trim(val, " \t");
            bit: u8;
            if key == "server_no_context_takeover" bit = 1;
            else if key == "client_no_context_takeover" bit = 2;
            else if key == "server_max_window_bits" bit = 4;
            else if key == "client_max_window_bits" bit = 8;
            else return .{}, .{ .HANDSHAKE, "unknown compression parameter", offset + offer.count - params.count };
            if seen & bit return .{}, .{ .HANDSHAKE, "duplicate compression parameter", offset + offer.count - params.count };

            seen |= bit;

            if bit <= 2 {
                if has_value return .{}, .{ .HANDSHAKE, "unexpected parameter value", offset };
                if bit == 1 result.server_no_context_takeover = true;
                else result.client_no_context_takeover = true;
            } else {
                bits := 15;

                if !has_value {
                    if bit == 4 || response return .{}, .{ .HANDSHAKE, "missing window bits", offset };
                } else {
                    if val.count >= 2 && val[0] == #char "\"" && val[val.count - 1] == #char "\"" val = slice(val, 1, val.count - 2);
                    if val == "8" bits = 8;
                    else if val == "9" bits = 9;
                    else if val == "10" bits = 10;
                    else if val == "11" bits = 11;
                    else if val == "12" bits = 12;
                    else if val == "13" bits = 13;
                    else if val == "14" bits = 14;
                    else if val != "15" return .{}, .{ .HANDSHAKE, "invalid window bits", offset };
                }

                if bit == 4 {
                    result.server_max_window_bits = bits;
                    result.server_window_present = true;
                } else {
                    result.client_max_window_bits = bits;
                    result.client_window_present = true;
                }
            }

            params = after;
        }

        remaining = rest;
        offset = value.count - rest.count;
    }

    return result, .{};
}

#scope_module
#import "String";
Base64 :: #import "Base64";
Sha :: #import,file "../../internal/crypto/sha.jai";

WS_GUID :: "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

cut :: (value: string, separator: u8) -> bool, string, string {
    found, before, after := split_from_left(value, separator);
    if !found return false, value, "";
    return true, before, after;
}

extension_text :: (p: Ws_Deflate) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    append(*builder, "permessage-deflate");
    if p.server_no_context_takeover append(*builder, "; server_no_context_takeover");
    if p.client_no_context_takeover append(*builder, "; client_no_context_takeover");
    if p.server_window_present print_to_builder(*builder, "; server_max_window_bits=%", p.server_max_window_bits);
    if p.client_window_present print_to_builder(*builder, "; client_max_window_bits=%", p.client_max_window_bits);

    return builder_to_string(*builder,, temp);
}

valid_token :: (value: string) -> bool {
    if value.count == 0 return false;

    for value {
        if (it >= #char "a" && it <= #char "z") || (it >= #char "A" && it <= #char "Z") || (it >= #char "0" && it <= #char "9") continue;
        if contains("!#$%&'*+-.^_`|~", string.{ 1, *it }) continue;
        return false;
    }

    return true;
}
