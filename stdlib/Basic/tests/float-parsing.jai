#import "Basic";

check :: (text: string, bits: u64) {
    value, success, remainder := string_to_float64(text);
    assert(success && remainder.count == 0);
    assert((cast(*u64) *value).* == bits);
}

check_single :: (text: string, bits: u32) {
    value, success, remainder := string_to_float(text);
    assert(success && remainder.count == 0);
    assert((cast(*u32) *value).* == bits);
}

main :: () {
    check("0.1", 0x3fb999999999999a);
    check("-0", 0x8000000000000000);
    check("1.00000000000000011102230246251565404236316680908203125", 0x3ff0000000000000);
    check("1.000000000000000111022302462515654042363166809082031251", 0x3ff0000000000001);
    check("9007199254740993", 0x4340000000000000);
    check("2.4703282292062327e-324", 0);
    check("2.4703282292062328e-324", 1);
    check("1.7976931348623157e308", 0x7fefffffffffffff);
    check("1.7976931348623159e308", 0x7ff0000000000000);
    check("-1e-999999999999999999999999999999", 0x8000000000000000);
    check("1e999999999999999999999999999999", 0x7ff0000000000000);
    check("0x1.8p+2", 0x4018000000000000);
    check("0x1p-1074", 1);
    check("+INFINITY", 0x7ff0000000000000);
    check("-nan(payload)", 0xfff8000000000000);
    check_single("1.000000059604644775390625000000001", 0x3f800001);
    check_single("1.000000178813934326171874999999999", 0x3f800001);
    check_single("340282356779733661637539395458142568447", 0x7f7fffff);
    check_single("340282356779733661637539395458142568448", 0x7f800000);
    check_single("0x1.00000000000000000001p-150", 1);
    check_single("0x1.7fffffffffffffffffffP-149", 1);
    value, success, remainder := string_to_float64(" \t-.25e2tail");

    assert(success && value == -25 && remainder == "tail");
    value, success, remainder = string_to_float64("1e+bad");
    assert(success && value == 1 && remainder == "e+bad");
    value, success, remainder = string_to_float64(" +.bad");
    assert(!success && value == 0 && remainder == " +.bad");
    print("ok\n");
}
