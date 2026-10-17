run :: () {
    if a {
        b();
    }
    c();
    d();
    for items {
        e(it);
    }
    f();
    if x return;
    g();
    while h() {
        i();
    }
    ok = lookup();
    if !ok {
        return;
    }
    v := make();
    if v.ready {
        go(v);
    }
    w := 1;
    z := 2;
    if z {
        go(w);
    }
    call(a,
         b);
    if p {
        q();
    } else if r {
        s();
    } else {
        t();
    }
    defer {
        cleanup();
    }
    #if OS == .MACOS {
        mac();
    }
    inner :: () {
        deep();
    }
    inner();
    if k == {
        case 1;
            a();
            if y {
                b();
            }
            c();
        case 2;
            if y {
                b();
            }
        case 3;
            d();
    }
    last();
    if tail {
        end();
    }
}

empty :: () {
    if a {
        b();
    }
}

tiny :: () { if a { b(); } c(); }

guards :: (a: *int, b: int) -> bool {
    if !a {
        return false;
    }
    if b < 0 {
        return false;
    }
    if b > 10 {
        b = 10;
        log(b);
    }
    if b == 0 {
        return false;
    }
    return true;
}

branches :: (x: int) {
    if x return;
    else {
        y();
    }
}
