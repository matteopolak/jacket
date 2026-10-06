// A `$$` parameter whose argument is omitted takes its constant default baked, so the body
// can test it with `#if` (smari/jai-xml's `lexer_skip_whitespace`).
// rules: poly.17
#import "Basic";

Parser :: struct {
    pos: int;
}

skip :: inline (p: *$T/Parser, $$must: bool = false) -> int {
    #if must return 1;
    else return 2;
}

step :: (p: *$T, $$by := 4) -> int {
    #if by == 4 {
        p.pos += 40;
    } else {
        p.pos += by;
    }
    return ifx is_constant(by) then 1 else 0;
}

main :: () {
    p: Parser;
    assert(skip(*p) == 2);
    assert(skip(*p, true) == 1);
    assert(skip(*p, must = false) == 2);

    assert(step(*p) == 1);
    assert(p.pos == 40);
    assert(step(*p, 2) == 1);
    assert(p.pos == 42);
    print("ok\n");
}
