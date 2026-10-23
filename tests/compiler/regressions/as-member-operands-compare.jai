// A struct with a scalar `#as` member stands for that member as an operand no `operator` takes,
// also when the member is itself a struct with a scalar `#as` member.
#import "Basic";

Handle :: struct {
    #as value: s32;
}

Connection :: struct {
    #as handle: Handle;
    bytes: int;
}

operator == :: inline (a: Connection, b: Connection) -> bool {
    return a.handle == b.handle;
}

main :: () {
    a: Connection;
    b: Connection;

    a.handle.value = 3;
    b.handle.value = 3;

    assert(a == b);
    assert(a.handle == 3);
    assert(!(a.handle != b.handle));

    b.handle.value = 4;

    assert(a != b);
    assert(a.handle < b.handle);

    print("ok\n");
}
