// `#overlay(f)` as a struct-body statement: inside `#if`, inside `#insert`ed members, in unions and
// in nested anonymous structs, and pointing at a `using` member.
// rules: struct.18 struct.22
#import "Basic";

Cfg :: struct($Wide: bool) {
    lo: u8;
    #if Wide {
        #overlay(lo) wide: u32;
    } else {
        #overlay(lo) narrow: u16;
    }
    after: u8;
}

Ins :: struct {
    a: u8;
    #insert "#overlay(a) b: u16;";
    c: u8;
}

Nest :: struct {
    head: u8;
    inner: struct {
        x: u8;
        #overlay(x) y: u16;
    };
    tail: u8;
}

Un :: union {
    a: u8;
    #overlay(a) b: u16;
    c: u32;
}

Two :: struct {
    first: u8;
    #overlay(first) second: u8;
    #overlay(first) third: u8;
    last: u8;
}

main :: () {
    w: Cfg(true);
    w.wide = 0x04030201;
    assert(w.lo == 1 && size_of(Cfg(true)) == 4);
    assert(offset_of(Cfg(true), "after") == 1);
    n: Cfg(false);
    n.narrow = 0x0302;
    assert(n.lo == 2 && size_of(Cfg(false)) == 2);

    i: Ins;
    i.b = 0x0101;
    assert(i.a == 1 && offset_of(Ins, "c") == 1 && size_of(Ins) == 2);

    ne: Nest;
    ne.inner.y = 0x0201;
    assert(ne.inner.x == 1);
    assert(offset_of(Nest, "tail") == 4 && size_of(Nest) == 6);

    u: Un;
    u.b = 0x7f7f;
    assert(u.a == 0x7f && size_of(Un) == 4);

    t: Two;
    t.second = 5;
    assert(t.first == 5 && t.third == 5 && offset_of(Two, "last") == 1);
    assert(size_of(Two) == 2);
    print("ok\n");
}
