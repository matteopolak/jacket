// A procedure changes its own copy of a struct or string parameter, never the caller's
// variable (sjorsdonkers/toml-jai's `string_to_int_checked` advances its `s` parameter).
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
    p.a += 10;
    q := *p;
    q.b = 99;
    return p.a + p.b;
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

    // At compile time too.
    #run {
        s := "abc";
        eat(s);
        assert(s == "abc");
    }
    print("ok\n");
}
