// `xx a | b` as an argument takes the parameter's type, also inside a macro, and flag
// constants on the other side follow it (chess-jai's `add_piece(p, xx a1|h1)`).
#import "Basic";

using Squares :: enum_flags u64 {
    a1;
    b1;
    c1;
}

bits :: (x: u64) -> u64 {
    return x;
}
bits_macro :: (x: u64) -> u64 #expand {
    return x;
}

main :: () {
    assert(bits(xx a1 | c1) == 5);
    assert(bits_macro(xx b1 | c1) == 6);
    assert(bits(xx (a1 | b1)) == 3);
    mask: u64 = 7;
    assert(bits(mask & xx b1) == 2);
    print("ok\n");
}
