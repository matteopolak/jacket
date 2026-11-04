#import "Basic";

make_view :: () -> [] u8 #expand {
    bytes: [3] u8 = .[1, 2, 3];
    return bytes;
}

local_use :: () {
    view := make_view();
    print("%", view[0]);
}

bad_return :: () -> [] u8 {
    return make_view();
}

nested_address :: () -> *int #expand {
    pointer: *int;
    {
        value := 5;
        pointer = *value;
    }
    return pointer;
}

nested_use :: () {
    pointer := nested_address();
    print("%", pointer.*);
}
