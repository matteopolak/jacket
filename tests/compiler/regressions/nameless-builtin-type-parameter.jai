// A builtin type name between parameters is a nameless parameter of that type, not a name that
// shares the next parameter's type or default.
// rules: proc.15
#import "Basic";

join :: (separator: string, string, _reversed := false) -> string {
    return separator;
}

pair :: (n: s32, s32) -> s32 {
    return n + 6;
}

main :: () {
    assert(join("a", "b") == "a");
    assert(join("a", "b", true) == "a");
    assert(pair(1, 2) == 7);
    print("ok\n");
}
