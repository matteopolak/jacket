predicate :: (T: Type) -> bool {
    return T == s64;
}

select_value :: (value: $T) -> T #modify {
    return predicate(T);
} {
    return value;
}

Pair :: struct {
    left: s64;
    right: s64;
}

select_value :: (value: $T) -> T {
    #assert T == Pair;
    return Pair.{ select_value(value.left), select_value(value.right) };
}

main :: () {}
