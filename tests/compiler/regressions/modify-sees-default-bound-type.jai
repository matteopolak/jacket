// A `#modify` block sees the type a parameter's default binds when the call omits the argument.
#import "Basic";

is_number :: (T: Type) -> bool {
    info := cast(*Type_Info) T;

    return info.type == .INTEGER || info.type == .FLOAT;
}

add :: inline ($$first: $T = 0.0, $$second: $U = 0.0) -> float64 #modify {
    return is_number(T) && is_number(U);
} {
    return cast(float64) first + cast(float64) second;
}

main :: () {
    value: float32 = 1.5;

    assert(add() == 0);
    assert(add(value) == 1.5);
    assert(add(2, 3) == 5);
    assert(add(value, 0.25) == 1.75);

    print("ok\n");
}
