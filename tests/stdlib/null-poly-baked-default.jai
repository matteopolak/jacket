// `null` binds `$T` (as `*void`) only after the other arguments; a defaulted baked
// parameter whose type names `T` (`$is_equal: (T, T) -> bool = null`) waits for it.
#import "Basic";

Matcher :: struct(T: Type, given: (T, T) -> bool) {
    expected: T;
    #if given {
        equal :: given;
    } else {
        equal :: (a: T, b: T) -> bool {
            return a == b;
        };
    }
}

is_not :: (expected: $T, $is_equal: (T, T) -> bool = null) -> *Matcher(T, is_equal) {
    m := New(Matcher(T, is_equal));
    m.expected = expected;
    return m;
}

matches :: (m: *Matcher($T, $E), actual: T) -> bool {
    return !m.equal(actual, m.expected);
}

main :: () {
    m := is_not(null);
    assert(type_of(m.expected) == *void);
    x := 3;
    assert(matches(m, *x));
    assert(!matches(m, null));

    p := is_not(*x);
    assert(type_of(p.expected) == *s64);
    assert(!matches(p, *x));
    print("ok\n");
}
