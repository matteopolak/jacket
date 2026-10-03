asn1_print :: (name: string, value: ASN1_Value) {
    asn1_print(name, value.content);
}

asn1_print :: (name: string, data: string) {
    diagnostic_banner(name);
    asn1_print(data);
    diagnostic_banner(name);
}

asn1_print :: (data: string, depth := 0, hack_skip_depth := false) {
    if depth < 0 || depth > 0x1fffffffffffffff {
        wire_failure("ASN.1 diagnostic depth is outside supported range");
        return;
    }

    frames: [..] Diagnostic_Frame;

    defer array_free(frames);
    array_add(*frames, .{ data, depth, hack_skip_depth });

    while frames.count > 0 {
        frame := *frames[frames.count - 1];
        if frame.remaining.count == 0 {
            frames.count -= 1;
            continue;
        }

        valid, value := parse_wire_value(*frame.remaining);
        if !valid return;

        if !frame.skip_indent {
            for 0..frame.depth - 1 print("    ");
        }

        next_depth := frame.depth + 1;
        if next_depth > 0x1fffffffffffffff {
            wire_failure("ASN.1 diagnostic nesting exceeds depth range");
            return;
        }

        if value.class == .CONTEXT_SPECIFIC {
            print("[%] ", value.number);

            if value.length_value == .CONSTRUCTED {
                if value.content.count == 0 print("\n");
                else array_add(*frames, .{ value.content, next_depth, true });
            } else {
                print("IMPLICT type (0x");
                diagnostic_hex_number(cast(u64) value.content.count);
                print(" bytes)\n");
            }

            continue;
        }

        kind := value.universal_type;
        if kind == .SEQUENCE || kind == .SET {
            print("%\n", ifx kind == .SEQUENCE then "SEQUENCE" else "SET");
            array_add(*frames, .{ value.content, next_depth, false });
        } else if kind == .OBJECT_IDENTIFIER {
            oid := asn1_decode_object_identifier(value.content);
            if oid.parts_count < 2 return;

            text := asn1_to_string(oid);

            print("OBJECT_IDENTIFIER %\n", text);
            free(text);
        } else if kind == .NULL {
            print("NULL ");

            if value.content.count != 0 {
                wire_failure("ASN.1 NULL requires empty content");
                return;
            }

            print("\n");
        } else if kind == .INTEGER {
            if value.length_value != .PRIMITIVE || value.content.count == 0 ||
               (value.content.count <= 8 && (value.content[0] & 128) != 0) {
                wire_failure("ASN.1 diagnostic INTEGER requires primitive nonnegative content");
                return;
            }

            if value.content.count <= 8 print("INTEGER %\n", asn1_decode_integer(value.content));
            else {
                print("INTEGER ");
                diagnostic_hex_bytes(value.content);
                print("\n");
            }
        } else if kind == .PRINTABLE_STRING || kind == .UTF8_STRING {
            print("% %\n", ifx kind == .PRINTABLE_STRING then "PRINTABLE_STRING" else "UTF8_STRING", value.content);
        } else if kind == .UTC_TIME {
            valid_time, calendar := diagnostic_utc_time(value.content);
            if !valid_time return;
            print("UTC_TIME %\n", calendar_to_string(calendar));
        } else if kind == .BIT_STRING {
            if value.content.count < 1 || value.content[0] != 0 {
                wire_failure("ASN.1 diagnostic BIT_STRING requires zero unused bits");
                return;
            }

            print("BIT_STRING ");
            diagnostic_hex_bytes(.{ value.content.count - 1, value.content.data + 1 });
            print("\n");
        } else if kind == .OCTET_STRING {
            print("OCTET_STRING (% bytes) ", value.content.count);
            diagnostic_hex_bytes(value.content);
            print("\n");
        } else {
            wire_failure("unsupported ASN.1 diagnostic universal type");
            return;
        }
    }
}

#scope_module
#import "Basic";

Diagnostic_Frame :: struct {
    remaining: string;
    depth: s64;
    skip_indent: bool;
}

diagnostic_banner :: (name: string) {
    Colors :: #import "Print_Color"(USE_ANSI_CODES_ON_WINDOWS = true);
    Colors.print_color("--- % ---\n", name, color = .YELLOW);
}

diagnostic_hex_bytes :: (bytes: string) {
    digits :: "0123456789abcdef";

    for byte, index: bytes {
        if index > 0 print(" ");
        pair: [2] u8 = .[digits[byte >> 4], digits[byte & 15]];
        print("%", string.{ 2, pair.data });
    }
}

diagnostic_hex_number :: (number: u64) {
    digits :: "0123456789abcdef";
    bytes: [16] u8;
    index := bytes.count;
    value := number;

    while true {
        index -= 1;
        bytes[index] = digits[value & 15];
        value >>= 4;
        if value == 0 break;
    }

    print("%", string.{ bytes.count - index, bytes.data + index });
}

diagnostic_utc_time :: (text: string) -> bool, Calendar_Time {
    value: Calendar_Time;

    if text.count != 13 || text[12] != #char "Z" {
        wire_failure("ASN.1 diagnostic UTC_TIME requires YYMMDDhhmmssZ");
        return false, value;
    }

    parts: [6] s32;

    for index: 0..11 {
        byte := text[index];
        if byte < #char "0" || byte > #char "9" {
            wire_failure("ASN.1 diagnostic UTC_TIME contains a nondecimal field");
            return false, value;
        }

        parts[index / 2] = parts[index / 2] * 10 + cast(s32) (byte - #char "0");
    }

    if parts[1] < 1 || parts[1] > 12 || parts[2] < 1 || parts[2] > 31 || parts[3] > 23 || parts[4] > 59 || parts[5] > 60 {
        wire_failure("ASN.1 diagnostic UTC_TIME fields are out of range");
        return false, value;
    }

    value.year = parts[0] + (ifx parts[0] >= 50 then 1900 else 2000);
    value.month_starting_at_0 = cast(s8) (parts[1] - 1);
    value.day_of_month_starting_at_0 = cast(s8) (parts[2] - 1);
    value.hour = cast(s8) parts[3];
    value.minute = cast(s8) parts[4];
    value.second = cast(s8) parts[5];
    value.time_zone = .UTC;

    return true, value;
}
