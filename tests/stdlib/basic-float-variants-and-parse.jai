// `FormatFloat` prints variants of floats (an `isa` of a `distinct` float64), and
// `string_to_float64_new` returns success first (sjorsdonkers/toml-jai).
#import "Basic";

Meters :: #type,distinct float64;
Close_Meters :: #type,isa Meters;
Count :: #type,distinct s32;
Small_Count :: #type,isa Count;

main :: () {
    m := cast(Close_Meters) 2.5;
    text := tprint("%", FormatFloat.{ value = m, trailing_width = 1 });
    assert(text == "2.5", "%", text);
    n := cast(Small_Count) 42;
    text = tprint("%", FormatInt.{ value = n });
    assert(text == "42", "%", text);

    ok, value, rest := string_to_float64_new("1.5e2xyz");
    assert(ok && value == 150 && rest == "xyz");
    ok, value, rest = string_to_float64_new("nope");
    assert(!ok);
    print("ok\n");
}
