// `using,only(...)` and `using,except(...)` inside an enum body bring in only the selected
// names of the other enum. The dropped ones are not reachable through this enum.
#import "Basic";

Scancode :: enum u32 {
    A;
    B;
    C;
    COUNT;
}

Key :: enum u32 {
    using,except(COUNT) Scancode;
    START :: Scancode.COUNT + 1;
    MOUSE;
    COUNT;
}

Narrow :: enum {
    using,only(B, C) Scancode;
    Z;
}

DROPPED :: string.["A", "B"];

Computed :: enum {
    using,except(DROPPED) Scancode;
    Z;
}

main :: () {
    assert(cast(int) Key.A == 0);
    assert(cast(int) Key.C == 2);
    assert(cast(int) Key.START == 4);
    assert(cast(int) Key.MOUSE == 5);
    assert(cast(int) Key.COUNT == 6);
    assert((cast(*Type_Info_Enum) type_info(Key)).names.count == 3);

    assert(cast(int) Narrow.B == 1);
    assert(cast(int) Narrow.C == 2);
    assert(cast(int) Narrow.Z == 0);

    assert(cast(int) Computed.C == 2);
    assert(cast(int) Computed.COUNT == 3);
}
