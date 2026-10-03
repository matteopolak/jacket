asn1_decode_integer :: (data: string) -> s64 {
    if data.count < 1 || data.count > 8 || (data[0] & 128) != 0 {
        wire_failure("ASN.1 INTEGER requires one through eight nonnegative content octets");
        return 0;
    }

    bits: u64;
    for byte: data bits = (bits << 8) | byte;

    value: s64;

    memcpy(*value, *bits, 8);

    return value;
}

asn1_decode_object_identifier :: (data: string) -> ASN1_Object_Identifier {
    result: ASN1_Object_Identifier;

    if data.count < 1 {
        wire_failure("empty ASN.1 object identifier");
        return result;
    }

    high: u64;
    low: u64;
    complete := false;

    for byte: data {
        if high > (0xffffffffffffffff >> 7) {
            wire_failure("ASN.1 object identifier arc overflow");
            return .{};
        }

        high = (high << 7) | (low >> 57);
        low = (low << 7) | (byte & 127);
        complete = (byte & 128) == 0;
        if !complete continue;

        if result.parts_count == 0 {
            if high == 0 && low < 80 {
                result.parts[0] = low / 40;
                result.parts[1] = low % 40;
            } else {
                if high > 1 || (high == 1 && low >= 80) {
                    wire_failure("ASN.1 object identifier second arc overflow");
                    return .{};
                }

                result.parts[0] = 2;
                // Subtracting80 across the carry word yields a valid u64 arc.
                if high == 1 result.parts[1] = 0xffffffffffffffff - (79 - low);
                else result.parts[1] = low - 80;
            }

            result.parts_count = 2;
        } else {
            if high != 0 || result.parts_count >= ASN1_Object_Identifier.MAX_SUBIDENTIFIERS {
                wire_failure("ASN.1 object identifier arc exceeds public representation");
                return .{};
            }

            result.parts[result.parts_count] = low;
            result.parts_count += 1;
        }

        high = 0;
        low = 0;
    }

    if !complete {
        wire_failure("truncated ASN.1 object identifier arc");
        return .{};
    }

    return result;
}

asn1_to_string :: (oid: ASN1_Object_Identifier) -> string {
    if oid.parts_count < 2 || oid.parts_count > ASN1_Object_Identifier.MAX_SUBIDENTIFIERS ||
       oid.parts[0] > 2 || (oid.parts[0] < 2 && oid.parts[1] >= 40) {
        wire_failure("invalid ASN.1 object identifier representation");
        return "";
    }

    builder: String_Builder;

    defer free_buffers(*builder);

    for index: 0..oid.parts_count - 1 {
        digits: [20] u8;
        cursor := digits.count;
        value := oid.parts[index];

        while true {
            cursor -= 1;
            digits[cursor] = cast(u8) (48 + value % 10);
            value /= 10;
            if value == 0 break;
        }

        append(*builder, digits.data + cursor, digits.count - cursor);
        append(*builder, ".");
    }

    return builder_to_string(*builder);
}
