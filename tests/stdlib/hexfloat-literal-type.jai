// An untyped `0h` literal is float32 with up to 8 hex digits and float64 with 16, whatever the
// value: 0h7FEF... used to default to float32 and become inf, 0h0010... to become 0.
#import "Basic";

F64_MAX :: 0h7FEFFFFF_FFFFFFFF;
F64_MIN :: 0h00100000_00000000;
F64_ONE :: 0h3FF00000_00000000;
F32_ONE :: 0h3F800000;

pick :: (x: float64) -> string {
    return "float64";
}

pick :: (x: float32) -> string {
    return "float32";
}

main :: () {
    assert(type_of(F64_MAX) == float64, "type is %", type_of(F64_MAX));
    assert(pick(F64_MAX) == "float64", "overload picked %", pick(F64_MAX));
    inferred := F64_MIN;
    assert(type_of(inferred) == float64);
    assert(inferred != 0, "the smallest normal float64 became 0");
    big := F64_MAX;
    assert(big > 1e308);

    // A float64 pattern stays float64 even when float32 could hold its value.
    assert(type_of(F64_ONE) == float64);
    assert(F64_ONE == 1);

    assert(type_of(F32_ONE) == float32);
    assert(pick(F32_ONE) == "float32");
    assert(F32_ONE == 1);
    small: float32 = 0h00800000;
    assert(small > 0);
    print("ok\n");
}
