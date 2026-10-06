// Calls renamed stdlib procedures with named arguments, matching the parameter names of the Jai modules.
#import "Basic";
#import "Math";
#import "Crc";
#import "Unicode";
#import "xxHash";

main :: () {
    a := Quaternion.{ 0, 0, 0, 1 };
    b := Quaternion.{ 0, 0, 1, 0 };
    q := slerp(start = a, end = b, t = 0.5);
    assert(length(a = q) > 0.99 && length(a = q) < 1.01);
    assert(length(a = Vector2.{ 3, 4 }) == 5);
    assert(length_squared(a = Vector3.{ 1, 2, 2 }) == 9);
    v := abs(v = Vector2.{ -1, 2 });
    assert(v.x == 1);

    assert(floor(f = 1.5) == 1);
    assert(ceil(f = 1.5) == 2);
    assert(floor(x = 1.5 .(float64)) == 1);
    assert(sqrt(x = 4.0) == 2);
    assert(abs(pow(x = 2.0, power = 3.0) - 8) < 0.001);
    assert(abs(pow(x = 2.0 .(float64), y = 3.0 .(float64)) - 8) < 0.001);
    assert(abs(tan(theta = 0.0)) < 0.0001);
    assert(abs(sin(x = 0.0)) < 0.0001);

    arr: [..] int;
    array_add(*arr, item = 5);
    array_insert_at(*arr, item = 3, index = 0);
    assert(arr.count == 2 && arr[0] == 3 && arr[1] == 5);

    s := sprint(format_string = "n=%", 7);
    assert(s == "n=7");
    n := print(format_string = "");
    assert(n == 0);

    ok, value, rest := false, 0.0, "";
    value, ok, rest = string_to_float(arg = "2.5");
    assert(ok && value == 2.5);

    assert(crc64(s = "abc") == crc64("abc"));

    text := "hi";
    code, status := utf8_next_character(s = *text);
    assert(code == #char "h" && status == .CONVERSION_OK);
    assert(text == "i");

    state: XXH32_state_t;
    XXH32_reset(*state, 0);
    XXH32_update(*state, input = "abc".data, len = 3);
    assert(XXH32_digest(*state) != 0);

    seconds, success := to_float64_seconds(input = to_apollo(1.5));
    assert(success && seconds == 1.5);
}
