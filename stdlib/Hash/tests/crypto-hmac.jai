#import "Basic";
H :: #import "internal/crypto/hmac";

hex :: (bytes: [] u8) -> string {
    digits := "0123456789abcdef";
    builder: String_Builder;

    for bytes {
        append(*builder, digits[it >> 4]);
        append(*builder, digits[it & 15]);
    }

    return builder_to_string(*builder);
}

repeat :: (byte: u8, count: int) -> string {
    data: *u8 = alloc(count);
    memset(data, byte, count);
    return string.{ count, data };
}

check_hmac :: (key: string, message: string, expected: string) {
    mac := H.hmac_sha256(key, message);
    got := hex(mac);
    if got != expected print("hmac mismatch: % vs %\n", got, expected);
    assert(got == expected);
}

main :: () {
    // RFC 4231 cases 1-4, 6, 7
    check_hmac(repeat(0x0b, 20), "Hi There", "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7");
    check_hmac("Jefe", "what do ya want for nothing?", "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843");
    check_hmac(repeat(0xaa, 20), repeat(0xdd, 50), "773ea91e36800e46854db8ebd09181a72959098b3ef8c122d9635514ced565fe");

    key4: [25] u8;
    for 0..24 key4[it] = cast(u8) (it + 1);

    check_hmac(string.{ 25, key4.data }, repeat(0xcd, 50), "82558a389a443c0ea4cc819899f2083a85f0faa3e578f8077a2e3ff46729665b");

    check_hmac(repeat(0xaa, 131), "Test Using Larger Than Block-Size Key - Hash Key First", "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54");
    check_hmac(repeat(0xaa, 131), "This is a test using a larger than block-size key and a larger than block-size data. The key needs to be hashed before being used by the HMAC algorithm.", "9b09ffa71b942fcb27635fbcd5b0e944bfdc63644f0713938a7f51535c3a35e2");

    // RFC 7914 section 11, first 32 bytes of the 64-byte output
    dk := H.pbkdf2_hmac_sha256("passwd", "salt", 1);

    assert(hex(dk) == "55ac046e56e3089fec1691c22544b605f94185216dde0465e68b9d57c20dacbc");

    // 4096 rounds (widely published vector)
    dk = H.pbkdf2_hmac_sha256("password", "salt", 4096);
    assert(hex(dk) == "c5e478d59288c841aa530db6845c4c8d962893a001ce4e11a4963873aa98134a");

    print("ok\n");
}
