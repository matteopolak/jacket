#import "Basic";

aligned_alias :: (p: *u8) -> *u8 {
    return cast(*u8) align_forward(cast(s64) p, 64);
}

identity :: (p: *u8) -> *u8 {
    return p;
}

main :: () {
    base := cast(*u8) alloc(128);

    free(base);
    alias := aligned_alias(base);

    print("%", alias.*);

    other := cast(*u8) alloc(16);

    free(other);
    forwarded := identity(other);

    print("%", forwarded.*);

    temporary := cast(*u8) talloc(16);

    reset_temporary_storage();
    old_temporary := identity(temporary);

    print("%", old_temporary.*);
}
