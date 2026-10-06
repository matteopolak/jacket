// A bitwise operation with an inferred member on the left takes the expected enum type when
// there is one, and the other operand's type when the expected type is not an enum.
#import "Basic";

Flags :: enum_flags u32 {
    A;
    B;
    C;
}

Holder :: struct {
    flags: Flags;
}

take :: (flags: Flags) -> u32 {
    return cast(u32) flags;
}

main :: () {
    // The parameter type names the members on both sides.
    assert(take(.A & ~ .B) == 1);
    on := false;
    picked: Flags = .C | ifx on then .B else cast(Flags) 0;
    assert(picked == .C);
    // The expected `int` does not name members; the right operand's type does.
    h := Holder.{ flags = .A | .C };
    assert(cast(int) (.C & h.flags) == 4);
    print("ok\n");
}
