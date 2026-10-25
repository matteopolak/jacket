#import "Basic";

Pair :: struct {
    x, y: s64;
}

Allowed :: Type.[Pair, string];

describe :: (value: $T/Allowed) -> string {
    return "listed";
}

describe :: (value: $T/ .[float32, bool]) -> string {
    return "inline";
}

describe :: (value: $T) -> string {
    return "other";
}

main :: () {
    assert(describe(Pair.{ 1, 2 }) == "listed", "a constant naming a type list admits its members");
    assert(describe(true) == "inline", "an inline type list admits its members");
    assert(describe(7) == "other", "types outside a list fall through to the unrestricted overload");
    print("ok\n");
}
