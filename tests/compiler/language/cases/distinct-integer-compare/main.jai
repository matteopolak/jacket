#import "Basic";

Handle :: #type,distinct u32;

make :: () -> Handle {
    return 2;
}

main :: () {
    found := false;

    for value, index: int.[10, 20, 30] {
        if index == make() found = true;
    }

    assert(found, "an index compares with a distinct integer type by value");
    small: u32 = 2;

    assert(small == make(), "so does the integer type it is made from");
    print("ok\n");
}
