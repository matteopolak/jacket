#import "Basic";

collect :: () {
    values: [..] u32;
    defer array_free(values);
    array_add(*values, cast(u32) 42);
    assert(values.count == 1 && values[0] == 42);
}

main :: () {
    collect();
    print("ok\n");
}
