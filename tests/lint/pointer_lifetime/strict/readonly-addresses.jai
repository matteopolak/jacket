A :: struct {
    values: [..] int;
}

grow :: (array: *[..] int) {} @GrowsArray
readonly :: (array: *[..] int) -> int {
    return array.count;
}

read :: (p: *int) -> int {
    return p.*;
}

bad :: () -> int {
    array: [..] int;
    p := *array[0];

    readonly(*array);
    grow(*array);

    return read(p);
}

safe :: () -> int {
    array: [..] int;

    readonly(*array);
    grow(*array);
    p := *array[0];

    return read(p);
}

opaque :: (array: *[..] int) #foreign lib "opaque";
lib :: #library "c";

unproved :: () -> int {
    array: [..] int;
    p := *array[0];

    opaque(*array);
    grow(*array);

    return read(p);
}
