// The 64 symbols of the standard alphabet in value order: `A-Z`, `a-z`, `0-9`, `+`, `/`.
BASE64_ENCODER_TABLE :: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
// The 64 symbols of the URL-safe alphabet: the standard one with `-` and `_` in place of `+` and `/`.
BASE64URL_ENCODER_TABLE :: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
// Maps each byte to what `base64_decode` does with it, for the standard alphabet.
//
// A value from 0 to 63 is a symbol. 64 means skip the byte (only a line feed), 65 is the padding `=`,
// which ends the input, and 66 is invalid. Copy the table and change entries to accept other bytes:
//
// ```jai
// table := BASE64_DECODER_TABLE;
// table[#char " "] = 64;
// decoded, ok := base64_decode("TW Fu", table);
// defer free(decoded);
// assert(ok && decoded == "Man");
// ```
BASE64_DECODER_TABLE :: #run decoder_table(BASE64_ENCODER_TABLE);
// The decoder table for the URL-safe alphabet. See `BASE64_DECODER_TABLE` for the values.
BASE64URL_DECODER_TABLE :: #run decoder_table(BASE64URL_ENCODER_TABLE);

// The Base64 text of the bytes in `input`, using the 64 symbols in `table`.
//
// With `pad` set (the default) the text ends in `=` so its length is a multiple of 4; without it the
// last group is left short. Empty input gives an empty string. `table` must hold exactly 64 bytes,
// or an assert fails.
//
// The result is allocated with the context allocator. The caller frees it.
//
// ```jai
// encoded := base64_encode("Man");
// defer free(encoded);
// assert(encoded == "TWFu");
//
// padded := base64_encode("Ma");
// defer free(padded);
// assert(padded == "TWE=");
//
// bare := base64_encode("Ma", pad = false);
// defer free(bare);
// assert(bare == "TWE");
// ```
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
            result.data[count] = #char "=";
            count += 1;
        }

        if remaining > 2 {
            result.data[count] = table[c & 63];
            count += 1;
        } else if pad {
            result.data[count] = #char "=";
            count += 1;
        }

        i += 3;
    }

    result.count = count;

    return result;
}

// The bytes that `input` encodes, and whether all of it was valid.
//
// Line feeds are skipped. A `=` ends the decoding and anything after it is ignored. Any other byte
// that is not in `table` (a carriage return, a space or a byte outside the alphabet, for example)
// makes the call stop there and return `false`; the bytes decoded up to that point are still
// returned. Padding is optional. Bits left over at the end that do not fill a byte are dropped
// without an error, so a lone trailing symbol decodes to nothing.
//
// The result is allocated with the context allocator, even when `success` is false. The caller
// frees it.
//
// ```jai
// decoded, ok := base64_decode("TWFu\nTWE=");
// defer free(decoded);
// assert(ok && decoded == "ManMa");
//
// partial, partial_ok := base64_decode("TW!u");
// defer free(partial);
// assert(!partial_ok && partial == "M");
// ```
base64_decode :: (input: string, table: [256] u8 = BASE64_DECODER_TABLE) -> string, success: bool {
    result := alloc_string((input.count / 4) * 3 + 3);
    count: s64;
    bits: u32;
    accumulator: u32;
    valid := true;

    for byte: input {
        symbol := table[byte];
        if symbol == SKIPPED_SYMBOL continue;
        if symbol == PADDING_SYMBOL break;

        if symbol > PADDING_SYMBOL {
            valid = false;
            break;
        }

        accumulator = (accumulator << 6) | cast(u32) symbol;
        bits += BITS_PER_SYMBOL;
        if bits >= 8 {
            bits -= 8;
            result.data[count] = cast,trunc(u8) (accumulator >> bits);
            count += 1;
            accumulator &= (1 << bits) - 1;
        }
    }

    result.count = count;

    return result, valid;
}

// The URL-safe Base64 text of `input`, with no `=` padding.
//
// The result is allocated with the context allocator. The caller frees it.
//
// ```jai
// encoded := base64url_encode("\xfb\xff");
// defer free(encoded);
// assert(encoded == "-_8");
// ```
base64url_encode :: (input: string) -> string {
    return base64_encode(input, BASE64URL_ENCODER_TABLE, false);
}

// The bytes that URL-safe Base64 text encodes, and whether all of it was valid. Padding is optional.
// Errors and ownership are as for `base64_decode`: the caller frees the result.
//
// ```jai
// decoded, ok := base64url_decode("-_8");
// defer free(decoded);
// assert(ok && decoded == "\xfb\xff");
// ```
base64url_decode :: (input: string) -> string, success: bool {
    return base64_decode(input, BASE64URL_DECODER_TABLE);
}

// Checks a standard Base64 round trip, with asserts. A smoke test for the module.
test_base64 :: () {
    encoded := base64_encode("Man");

    defer free(encoded);
    assert(encoded == "TWFu");
    decoded, ok := base64_decode(encoded);

    defer free(decoded);
    assert(ok && decoded == "Man");
}

// Checks a URL-safe Base64 round trip, with asserts. A smoke test for the module.
test_base64_url :: () {
    encoded := base64url_encode("Ma");

    defer free(encoded);
    assert(encoded == "TWE");
    decoded, ok := base64url_decode(encoded);

    defer free(decoded);
    assert(ok && decoded == "Ma");
}

// Runs the module's smoke tests, `test_base64` and `test_base64_url`.
test_base64_module :: () {
    test_base64();
    test_base64_url();
}

#scope_file
#import "Basic";

BITS_PER_SYMBOL :: 6;
SKIPPED_SYMBOL :: 64;
PADDING_SYMBOL :: 65;
INVALID_SYMBOL :: 66;

decoder_table :: (alphabet: string) -> [256] u8 {
    result: [256] u8;
    for *byte: result byte.* = INVALID_SYMBOL;
    for byte, i: alphabet result[byte] = cast(u8) i;

    result[#char "\n"] = SKIPPED_SYMBOL;
    result[#char "="] = PADDING_SYMBOL;

    return result;
}
