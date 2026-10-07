// Checks the rules of docs/language/using.md.
// rules: using.1 using.2 using.3 using.5 using.6 using.7 using.8 using.9 using.10 using.11 using.12 using.13 using.14 using.15 using.16
#import "Basic";

Inner :: struct {
    n: int = 5;
    K :: 3;
}

Plain :: struct {
    using i: Inner;
}

Base :: struct {
    x: int = 1;
}

Derived :: struct {
    using #as b: Base;
    y: int;
}

Flipped :: struct {
    #as using b: Base;
}

get_x :: (b: *Base) -> int {
    return b.x;
}

get_x_value :: (b: Base) -> int {
    return b.x;
}

Meta :: struct {
    count: int;
}

Owner :: struct {
    using meta: *Meta;
}

Mode :: enum {
    A;
    B;
    C;
}

Settings :: struct {
    mode: Mode;
    extra: int = 7;
    limit: int = 2;
}

Tuned :: struct {
    using settings: Settings;
    mode = .B;
}

Nested :: struct {
    using #as settings: Settings;
    settings.limit = 40;
}

Shadow :: struct {
    using i: Inner;
    n: int = 99;
}

Pt :: struct {
    x: int;
    y: int;
}

move :: (using p: *Pt, dx: int) {
    x += dx;
}

Config :: struct {
    level: int = 4;
}

using file_config := Config.{ level = 6 };

Fruit :: enum {
    APPLE;
    MANGO;
}

Basket :: struct {
    tag: Fruit;
}

main :: () {
    // using.1, using.2: promoted members and constants.
    p: Plain;
    assert(p.n == 5 && p.i.n == 5, "using.1");
    assert(Plain.K == 3, "using.2");

    // using.3: #as converts to the embedded type, by pointer and by value.
    d: Derived;
    d.x = 8;
    assert(get_x(*d) == 8 && get_x_value(d) == 8, "using.3");

    // using.5: the member carries USING and AS flags.
    f := type_info(Derived).members[0].flags;
    assert((f & .USING) != 0 && (f & .AS) != 0, "using.5");

    // using.6, using.7: a pointer member promotes the pointee and writes through it.
    m: Meta;
    o: Owner;
    o.meta = *m;
    o.count = 12;
    assert(m.count == 12 && o.count == 12, "using.6 using.7");

    // using.8, using.10: default override of a promoted member; others keep theirs.
    t: Tuned;
    nt := New(Tuned);
    assert(t.mode == .B && nt.mode == .B, "using.8");
    assert(t.extra == 7 && nt.extra == 7, "using.10");

    // using.9: override through a dotted path.
    nd: Nested;
    assert(nd.limit == 40 && nd.extra == 7, "using.9");

    // using.11: using on a parameter and on a local.
    pt := Pt.{ 1, 2 };
    move(*pt, 4);
    assert(pt.x == 5, "using.11 parameter");
    {
        using pt;
        y = 30;
    }
    assert(pt.y == 30, "using.11 local");

    // using.12: using a literal makes a writable local.
    {
        using Pt.{ 7, 8 };
        x += 1;
        assert(x == 8 && y == 8, "using.12");
    }

    // using.13: file-scope using of a variable.
    assert(level == 6, "using.13");
    level = 9;
    assert(file_config.level == 9, "using.13 same variable");

    // using.14: using an enum-typed value brings in the enum members.
    basket := Basket.{ .MANGO };
    using basket.tag;
    hit := false;
    if basket.tag == {
        case APPLE; hit = false;
        case MANGO; hit = true;
    }
    assert(hit, "using.14");

    // using.15: a direct member shadows a promoted one.
    s: Shadow;
    assert(s.n == 99 && s.i.n == 5, "using.15");

    // using.16: both modifier orders parse and act the same.
    fl: Flipped;
    assert(get_x(*fl) == 1, "using.16");

    print("ok\n");
}
