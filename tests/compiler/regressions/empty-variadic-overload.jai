#import "Basic";

append_first :: (array: *[..] $T, items: ..T) {}

append_first :: (array: *[..] $T) -> *T {
    return null;
}

fixed_first :: (array: *[..] $T) -> *T {
    return null;
}

fixed_first :: (array: *[..] $T, items: ..T) {}

main :: () {
    values: [..] u32;
    first := append_first(*values);
    second := fixed_first(*values);

    assert(first == null && second == null);
    append_first(*values, 1, 2);
    fixed_first(*values, 1, 2);
}
