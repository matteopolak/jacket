// An `ifx` arm that only an expected type can type (`.B`, `.{}`, `.[...]`) takes the other arm's
// type whichever side it is on.
#import "Basic";

V :: struct {
    a: s32;
    b: s32;
}

E :: enum {
    A;
    B;
}

main :: () {
    yes := true;
    no := false;

    zero := ifx yes then .{} else V.{ 1, 1 };

    assert(type_of(zero) == V && zero.a == 0 && zero.b == 0);
    positional := ifx yes then .{ 3, 4 } else V.{ 1, 1 };

    assert(positional.a == 3 && positional.b == 4);
    named := ifx yes then .{ b = 5 } else V.{ 1, 1 };

    assert(named.a == 0 && named.b == 5);
    other := ifx no then .{ 3, 4 } else V.{ 1, 1 };

    assert(other.a == 1 && other.b == 1);

    member := ifx yes then .B else E.A;

    assert(type_of(member) == E && member == .B);
    member_else := ifx no then .B else E.A;

    assert(member_else == .A);

    og: [3] s32 = .[7, 8, 9];
    array := ifx yes then .[1, 2, 3] else og;

    assert(type_of(array) == [3] s32 && array[0] == 1 && array[2] == 3);

    print("ok\n");
}
