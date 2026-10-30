// Reviewed constants/tails/XSH-RR/CRC polynomial; explicit checks survive assertions off.
#import "Basic";
B :: #import "Base64";
C :: #import "Crc";
P :: #import "PCG";

check_decode :: (encoded: string, expected: string, success: bool) {
    decoded, ok := B.base64_decode(encoded);
    defer free(decoded);
    if decoded != expected || ok != success exit(2);
}

main :: () {
    check_decode("", "", true);
    check_decode("T", "", true);
    check_decode("TQ", "M", true);
    check_decode("TWE", "Ma", true);
    check_decode("T\nQ\n=ignored\xff", "M", true);
    check_decode("TQ\r==", "M", false);
    check_decode("TQ ==", "M", false);
    check_decode("TQ\t==", "M", false);
    check_decode("TWE!rest", "Ma", false);

    for index: 0..255 {
        expected: u8 = 66;
        if index >= 65 && index <= 90 expected = cast(u8) (index - 65);
        if index >= 97 && index <= 122 expected = cast(u8) (index - 97 + 26);
        if index >= 48 && index <= 57 expected = cast(u8) (index - 48 + 52);
        if index == 10 expected = 64;
        if index == 61 expected = 65;

        standard := expected;
        url := expected;

        if index == 43 standard = 62;
        if index == 47 standard = 63;
        if index == 45 url = 62;
        if index == 95 url = 63;
        if B.BASE64_DECODER_TABLE[index] != standard || B.BASE64URL_DECODER_TABLE[index] != url exit(3);
    }

    custom := B.BASE64_DECODER_TABLE;
    custom[46] = 0;
    custom[13] = 64;
    decoded, ok := B.base64_decode(".\r.", custom);
    if !ok || decoded != "\0" exit(4);

    free(decoded);
    if B.BASE64_DECODER_TABLE[46] != 66 || B.BASE64_DECODER_TABLE[13] != 66 exit(5);

    reversed: [64] u8;
    for *byte, index: reversed byte.* = B.BASE64_ENCODER_TABLE[63 - index];

    encoded := B.base64_encode("\0", string.{ reversed.count, reversed.data });
    if encoded != "//==" exit(16);

    free(encoded);
    encoded = B.base64_encode("\0", pad = false);
    if encoded != "AA" exit(17);

    free(encoded);

    raw: [256] u8;
    for *byte, index: raw byte.* = cast(u8) index;

    bytes := string.{ raw.count, raw.data };

    if C.crc64("") != 0 || C.crc64("123456789") != 0x62ec59e3f1a4f00a exit(6);
    // Independently calculated with Python arbitrary-width arithmetic from
    // the reviewed polynomial/init/final contract, masking every step to 64 bits.
    if C.crc64(bytes) != 0x59c3325b2927a19a exit(7);

    encoded = B.base64url_encode(bytes);
    if encoded.count != 342 exit(18);

    decoded, ok = B.base64url_decode(encoded);
    if !ok || decoded != bytes exit(8);

    free(encoded);
    free(decoded);

    state: u64;

    P.random_seed(*state, 42);
    if state != 0x91778aed87ee5eb1 exit(9);

    outputs := u32.[0x75830bbd, 0x0e6dfdb2, 0xce19afdf, 0xd8cfe2c3, 0x012b061e];
    states := u64.[0x39b7f8a5c64cf56c, 0x69afc5a5e88b394b, 0xa161c43f85f3c87e, 0xae1e2aac8d0e4175, 0x06b6f019aa7b0de0];

    for value, index: outputs if P.random_get(*state) != value || state != states[index] exit(10);

    P.random_seed(*state, 42);
    P.random_seed(42);
    for 0..99 if P.random_get() != P.random_get(*state) exit(11);

    for bound: u32.[1, 2, 97, 0xffffffff] {
        for 0..99 {
            if P.random_get_within_bound(*state, bound) >= bound exit(12);
            if P.random_get_within_bound2(*state, bound) >= bound exit(13);
        }
    }

    for 0..99 {
        if P.random_get_within_bound(97) >= 97 exit(14);

        a := P.random_get_zero_to_one(*state);
        b := P.random_get_zero_to_one();
        c := P.random_get_within_range(*state, -3, 7);
        d := P.random_get_within_range(-3, 7);

        if a < 0 || a > 1 || b < 0 || b > 1 || c < -3 || c > 7 || d < -3 || d > 7 exit(15);
    }

    write_string("ok\n");
}
