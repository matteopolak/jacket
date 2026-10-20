// A procedure can change its own copy of a struct or string parameter, never the caller's
// variable: it copies the parameter first, or passes its address on.
// rules: proc.12
#import "Basic";

Pair :: struct {
    a, b: int;
}

eat :: (s: string) -> int {
    n := s.count;
    advance(*s, s.count);
    return n;
}

poly_eat :: (s: string, $T := s64) -> T {
    advance(*s, 1);
    return cast(T) s.count;
}

bump :: (p: Pair) -> int {
    p := p;
    p.a += 10;
    q := *p;
    q.b = 99;

    return p.a + p.b;
}

scalar :: (n: int) -> int {
    n += 1;
    return n;
}

clear_first :: (view: [] int) {
    view[0] = 0;
}

main :: () {
    text := "hello";

    assert(eat(text) == 5);
    assert(text == "hello");
    assert(poly_eat(text) == 4);
    assert(text == "hello");

    pair := Pair.{ 1, 2 };

    assert(bump(pair) == 110);
    assert(pair.a == 1 && pair.b == 2);

    assert(scalar(4) == 5);

    // The elements of a view lie behind its data pointer.
    numbers := int.[7, 8];

    clear_first(numbers);
    assert(numbers[0] == 0 && numbers[1] == 8);

    // At compile time too.
    #run {
        s := "abc";
        eat(s);
        assert(s == "abc");
    }

    print("ok\n");
}
