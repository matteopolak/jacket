// `#type (*T)` is a procedure type with one unnamed `*T` parameter, not a parenthesized type.
Unload :: #type (*int);
Count :: #type (int, int) -> int;
unload_count := 0;

drop :: (p: *int) {
    unload_count += p.*;
}

main :: () -> s64 {
    f: Unload = drop;
    x := 5;
    f(*x);
    g: Count = (a: int, b: int) -> int {
        return a + b;
    };
    return ifx unload_count == 5 && g(1, 2) == 3 then 0 else 1;
}
