BASE64_ENCODER_TABLE :: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
BASE64URL_ENCODER_TABLE :: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
BASE64_DECODER_TABLE :: #run decoder_table(BASE64_ENCODER_TABLE);
BASE64URL_DECODER_TABLE :: #run decoder_table(BASE64URL_ENCODER_TABLE);

base64_encode :: (input: string, table: string = BASE64_ENCODER_TABLE, pad := true) -> string {
    assert(table.count == 64);
    result := alloc_string(((input.count + 2) / 3) * 4);
    i: s64;
    count: s64;

    while i < input.count {
        a := cast(u32) input[i];
        b: u32;
        c: u32;
        remaining := input.count - i;

        if remaining > 1 b = input[i + 1];
        if remaining > 2 c = input[i + 2];

        result.data[count] = table[a >> 2];
        count += 1;
        result.data[count] = table[((a & 3) << 4) | (b >> 4)];
        count += 1;

        if remaining > 1 {
            result.data[count] = table[((b & 15) << 2) | (c >> 6)];
            count += 1;
        } else if pad {
            result.data[count] = 61;
            count += 1;
        }

        if remaining > 2 {
            result.data[count] = table[c & 63];
            count += 1;
        } else if pad {
            result.data[count] = 61;
            count += 1;
        }

        i += 3;
    }

    result.count = count;

    return result;
}

base64_decode :: (input: string, table: [256] u8 = BASE64_DECODER_TABLE) -> string, success: bool {
    result := alloc_string((input.count / 4) * 3 + 3);
    count: s64;
    bits: u32;
    accumulator: u32;
    valid := true;

    for byte: input {
        symbol := table[byte];
        if symbol == 64 continue;
        if symbol == 65 break;

        if symbol > 65 {
            valid = false;
            break;
        }

        accumulator = (accumulator << 6) | cast(u32) symbol;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            result.data[count] = cast,truncate(u8) (accumulator >> bits);
            count += 1;
            accumulator &= (1 << bits) - 1;
        }
    }

    result.count = count;

    return result, valid;
}

base64url_encode :: (input: string) -> string {
    return base64_encode(input, BASE64URL_ENCODER_TABLE, false);
}

base64url_decode :: (input: string) -> string, success: bool {
    return base64_decode(input, BASE64URL_DECODER_TABLE);
}

test_base64 :: () {
    encoded := base64_encode("Man");

    defer free(encoded);
    assert(encoded == "TWFu");
    decoded, ok := base64_decode(encoded);

    defer free(decoded);
    assert(ok && decoded == "Man");
}

test_base64_url :: () {
    encoded := base64url_encode("Ma");

    defer free(encoded);
    assert(encoded == "TWE");
    decoded, ok := base64url_decode(encoded);

    defer free(decoded);
    assert(ok && decoded == "Ma");
}

test_base64_module :: () {
    test_base64();
    test_base64_url();
}

#scope_file
#import "Basic";

decoder_table :: (alphabet: string) -> [256] u8 {
    result: [256] u8;
    for *byte: result byte.* = 66;
    for byte, i: alphabet result[byte] = cast(u8) i;

    result[10] = 64;
    result[61] = 65;

    return result;
}
