F :: #type (*[..] int);

grow :: (array: *[..] int) {} @GrowsArray
also_grow :: (array: *[..] int) {} @GrowsArray
read :: (p: *int) -> int {
    return p.*;
}

public_callback: F = grow;

unproved_public :: () -> int {
    array: [..] int;
    p := *array[0];
    public_callback(*array);
    return read(p);
}

#scope_file

closed: F = grow;
open: F = grow;

select :: (flag: bool) {
    if flag closed = grow;
    else closed = also_grow;
}

expose :: () {
    opaque(*open);
}

opaque :: (slot: *F) #foreign lib "unknown_global_writer";
lib :: #library "c";

bad_closed :: () -> int {
    array: [..] int;
    p := *array[0];
    closed(*array);
    return read(p);
}

unproved_open :: () -> int {
    array: [..] int;
    p := *array[0];
    open(*array);
    return read(p);
}
