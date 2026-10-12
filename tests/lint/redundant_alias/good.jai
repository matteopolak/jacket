#import "Basic";

two :: () -> int, int {
    return 1, 2;
}

// Several results: `return two()` would return both.
first_of :: () -> int {
    x := two();
    return x;
}

// The returned value is widened on the way out.
widened :: (a: int) -> int {
    small := cast(s32) a;
    return small;
}

// A named return.
named :: (n: int) -> r: int {
    v := n + 1;
    return v;
}

// An explicit type on the copy, a cast or `xx` on it.
typed :: (n: int) {
    a := 5;
    b: s64 = a;
    c := n;
    d := cast(s32) c;
    e := n;
    f: s32 = xx e;

    print("%\n", b + d + f);
}

// More than one use.
twice :: (n: int) -> int {
    a := n * 2;
    b := a;
    print("%\n", a);
    return b;
}

// Mutated or address-taken.
mutated :: (n: int) {
    a := n;
    a += 1;
    b := a;
    print("%\n", b);
}

addressed :: (n: int) {
    p := n;
    q := *p;
    print("%\n", q.*);
}

deferred :: () {
    c := 4;
    defer print("%\n", c);
    d := c;
    print("%\n", d);
}

// Statements between the two.
separated :: (n: int) {
    a := n + 1;
    print("between\n");
    b := a;
    print("%\n", b);
}

// A comment between the two.
commented :: (n: int) {
    a := n + 1;
    // Kept for the reader.
    b := a;
    print("%\n", b);
}

// No initializer, or an uninitialized one.
storage :: () {
    a: [4] int;
    b := a;
    u: int = ---;
    v := u;
    v = 2;
    print("%\n", b[0] + v);
}

// The new name appears in the first initializer.
shadows :: (b: int) {
    a := b + 1;

    {
        b := a;
        print("%\n", b);
    }
}

// Used by inserted or disabled code.
inserted :: () {
    count := 0;
    #insert "count += 1;";
    total := count;
    print("%\n", total);
}

disabled :: (n: int) {
    path := n;

    #if OS == .WINDOWS {
        print("%\n", path);
    }

    other := path;

    print("%\n", other);
}

// A macro body.
macro :: (n: int) #expand {
    a := n + 1;
    b := a;
    print("%\n", b);
}

// Declared into the caller's scope.
declare :: () #expand {
    `a := 1;
    `b := a;
}

// Loop variable and parameter.
others :: (n: int) {
    for 0..1 {
        c := it;
        print("%\n", c);
    }

    p := n;

    print("%\n", p);
}

// A returned value used elsewhere.
used_again :: (n: int) -> int {
    x := n + 1;
    print("%\n", x);
    return x;
}

main :: () {
    first_of();
    print("%\n", widened(1));
    named(1);
    typed(1);
    twice(1);
    mutated(1);
    addressed(1);
    deferred();
    separated(1);
    commented(1);
    storage();
    shadows(1);
    inserted();
    disabled(1);
    macro(1);
    declare();
    print("%\n", b);
    others(1);
    used_again(1);
}
