// Private Windows argument conversion unit. Placed byte-for-byte inside the
// staged library so the NEW platform-io dependency has its production path.
#load "../../../../internal/bindings/dxc-wide/module.jai";
TestBasic :: #import "Basic";

test_libc :: #library,system "c";
fail :: () #foreign test_libc "abort";

check :: (value: bool) {
    if !value fail();
}

main :: () {
    empty := argument("");

    check(empty != null && empty[0] == 0);
    defer TestBasic.free(empty);
    encoded := argument("café 😀");

    check(encoded != null);
    defer TestBasic.free(encoded);
    expected: [8] u16 = .[0x63, 0x61, 0x66, 0xe9, 0x20, 0xd83d, 0xde00, 0];
    for i: 0..expected.count - 1 check(encoded[i] == expected[i]);

    overlong: [2] u8 = .[0xc0, 0xaf];
    invalid: string = .{ data = overlong.data, count = 2 };

    check(argument(invalid) == null);
    embedded: [3] u8 = .[0x61, 0, 0x62];

    invalid = .{ data = embedded.data, count = 3 };
    check(argument(invalid) == null);
    invalid = .{ data = null, count = 0x8000_0000 };
    check(argument(invalid) == null);
    invalid = .{ data = null, count = -1 };
    check(argument(invalid) == null);
}
