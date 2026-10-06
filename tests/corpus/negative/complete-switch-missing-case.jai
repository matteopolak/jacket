// `if #complete x == {` must name every member of x's enum; .BLUE has no case here.
// A plain switch on the same value, and a #complete switch on enum_flags, are fine.
Color :: enum { RED; GREEN; BLUE; }
Bits :: enum_flags { ONE; TWO; }

name :: (c: Color) -> int {
    if c == {
        case .RED; return 1;
    }
    flags := Bits.ONE;
    if #complete flags == {
        case .ONE; return 3;
    }
    if #complete c == {
        case .RED; return 1;
        case .GREEN; return 2;
    }
    return 0;
}

main :: () -> int { return name(.RED); }
