// test: should_fail("unwrap: the Variant holds `s64`, not `string`")
#import "Basic";
#import "Extensions/Variant";

// unwrap asserts that the case it is asked for is the live one.
main :: () {
    v := variant(Variant(s64, string), 5);

    print("%\n", unwrap(v, string));
}
