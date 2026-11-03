#import "Basic";

compare :: (value: *int) -> bool {
    return value != null;
}

read :: (value: *int) -> int {
    return value.*;
}

safe_value :: () -> bool {
    value := New(int);
    free(value);
    return compare(value);
}

bad_read :: () -> int {
    value := New(int);
    free(value);
    return read(value);
}

safe_indirect :: (choice: bool) -> bool {
    value := New(int);
    target := compare;
    free(value);
    return target(value);
}

bad_indirect :: () -> int {
    value := New(int);
    target := read;
    free(value);
    return target(value);
}

safe_copied_value :: (value: *int) -> int {
    number := read(value);
    free(value);
    return number;
}

address_bits :: (value: *int) -> u64 {
    return cast(u64) value;
}

safe_address_bits :: (value: *int) -> u64 {
    free(value);
    return address_bits(value);
}

no_release :: (value: *int) #foreign lib "read_without_release"; @NoLifetimeEffects
lib :: #library "c";

may_opaque_read :: () {
    value := New(int);
    free(value);
    no_release(value);
}
