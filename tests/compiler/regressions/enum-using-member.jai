// `using` inside an enum body makes another enum's names reachable through this one, with
// their own values. They are not members of this enum, and do not shift its numbering.
#import "Basic";

A :: enum {
    X;
    Y;
}

E :: enum {
    using A;
    Z;
}

B :: enum {
    X :: 5;
    Y;
}

F :: enum {
    Q;
    using B;
    Z;
}

main :: () {
    assert(cast(int) E.X == 0);
    assert(cast(int) E.Y == 1);
    assert(cast(int) E.Z == 0);

    assert(cast(int) F.Q == 0);
    assert(cast(int) F.X == 5);
    assert(cast(int) F.Y == 6);
    assert(cast(int) F.Z == 1);

    assert(sprint("%", E.Z) == "Z");
    assert(sprint("%", E.Y) == "(enum out of range: 1)");
    assert((cast(*Type_Info_Enum) type_info(E)).names.count == 1);
}
