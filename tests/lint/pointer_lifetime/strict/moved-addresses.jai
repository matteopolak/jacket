#import "Basic";

compare :: (value: *int) -> bool {
    return value != null;
}

read :: (value: *int) -> int {
    return value.*;
}

// The same distinction applies to generations of resizable storage.
safe_moved_address :: () -> bool {
    values: [..] int;

    array_add(*values, 1);
    pointer := *values[0];

    values.count = values.allocated;
    array_add(*values, 2);

    return compare(pointer);
}

bad_moved_read :: () -> int {
    values: [..] int;

    array_add(*values, 1);
    pointer := *values[0];

    values.count = values.allocated;
    array_add(*values, 2);

    return read(pointer);
}
