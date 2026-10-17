// Original language reproducer: default generic arguments on a pointer type.
#import "Basic";

Box :: struct(T: Type = void) {
    value: s64;
}

is_null :: (pointer: *$B/Box) -> bool {
    return pointer == null;
}

main :: () {
    pointer: *Box;
    assert(is_null(pointer));
}
