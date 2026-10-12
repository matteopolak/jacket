run :: () {
    a := 1;
    b := a + 1;

    print("%\n", b);
    c, ok := lookup(b);
    if !ok return;
    x: int;
    y :: 4;

    if y > 1 {
        z := 2;
        use(z);
    }
    q := 3;
    r := q.field;

    if r {
        use(r);
    }
    count := 0;
    count += 1;
    p :: (n: int) -> int {
        return n;
    }
    w := p(1);

    using w;
    s := 0;

    for items {
        s += it;
    }
    last := 1;
}

other :: () {
    // a comment belongs to the statement under it
    n := 1;

    // so the blank goes above this one
    use(n);
    if n == {
        case 1;
            m := 2;

            use(m);
        case 2;
            k := 3;
        case;
            j := 4;

            use(j);
    }
    t := 1;
    if t {
        u := 2;
    } else {
        v := 3;
    }
    x := 1;
    y := 2;

    use(x, y);
}

Point :: struct {
    x: int;
    y: int;
    scale :: 2;
    describe :: (p: Point) {
        a := p.x;
        b := p.y;
        use(a, b);
    }
}

unused :: () {
    a := 1;
    // jaifmt: off
    b := 2;
    use(b);
    // jaifmt: on
    use(a);
}

short :: () -> int {
    c, size := next();
    pos += size;
    return c;
}

init :: () {
    source := get();
    result: Config;
    result.base = source.base;
    result.limit = source.limit;
    other: Config;

    unrelated = 1;
    other.base = 2;
}
