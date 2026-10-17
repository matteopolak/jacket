#import "Basic";
#import "Extensions/Long_Double";

Distinct_Wide :: #type,distinct Long_Double;
Wrapped_Wide :: #type,isa Distinct_Wide;

Holder :: struct {
    value: Long_Double;
}

check_conversion :: (value: Long_Double) {
    original: [16] u8;
    payload := cast(*u8) *value;

    for 0..size_of(Long_Double) - 1 original[it] = payload[it];

    converted := cast(float64) value;

    assert(tprint("%", value) == tprint("%", converted));
    assert(tprint("%", formatFloat(value, mode = .SHORTEST)) == tprint("%", formatFloat(converted, mode = .SHORTEST)));
    assert(tprint("%", formatFloat(value, trailing_width = 3, zero_removal = .NO)) == tprint("%", formatFloat(converted, trailing_width = 3, zero_removal = .NO)));
    assert(tprint("%", formatFloat(value, mode = .SCIENTIFIC)) == tprint("%", formatFloat(converted, mode = .SCIENTIFIC)));
    for 0..size_of(Long_Double) - 1 assert(original[it] == payload[it]);
}

main :: () {
    value: Long_Double = 1.5;
    borrowed: Any = value;
    pointer_before := borrowed.value_pointer;

    assert(tprint("%", value) == "1.5");
    assert(tprint("%", borrowed) == "1.5");
    assert(tprint("%", formatFloat(value, trailing_width = 2, zero_removal = .NO)) == "1.50");
    holder := Holder.{ value };
    values: [2] Long_Double = .[value, 2.5];
    distinct_value: Distinct_Wide = 1.5;
    wrapped: Wrapped_Wide = 1.5;

    assert(tprint("%", holder) == "{1.5}");
    assert(tprint("%", values) == "[1.5, 2.5]");
    assert(tprint("%", distinct_value) == "1.5");
    assert(tprint("%", wrapped) == "1.5");
    assert(tprint("%", formatFloat(wrapped, trailing_width = 2, zero_removal = .NO)) == "1.50");
    assert(value == 1.5 && borrowed.value_pointer == pointer_before && pointer_before == *value);
    value = 2.5;
    assert(tprint("%", borrowed) == "2.5");
    check_conversion(value);
    zero: Long_Double;

    check_conversion(-zero);
    check_conversion(zero / zero);
    check_conversion(value / zero);
    check_conversion(-value / zero);
    big: Long_Double = 1;
    tiny: Long_Double = 1;

    for 1..1100 {
        big *= 2;
        tiny /= 2;
    }

    check_conversion(big);
    check_conversion(-big);
    check_conversion(tiny);
    // An exact binary64 midpoint and a wide value just above it test rounding.
    step: Long_Double = 1;
    for 1..53 step /= 2;

    check_conversion(1 + step);
    check_conversion(1 + step + step / 128);

    #if LONG_DOUBLE_IS_WIDE {
        assert(tprint("%", formatFloat(1 + step, mode = .SHORTEST)) == "1");
        assert(tprint("%", formatFloat(1 + step + step / 128, mode = .SHORTEST)) == "1.0000000000000002");
    }

    assert(tprint("%", cast(float32) 1.25) == "1.25");
    assert(tprint("%", formatFloat(cast(float32) 0.1, mode = .SHORTEST)) == "0.1");
    assert(tprint("%", formatFloat(cast(float64) 0.1, mode = .SHORTEST)) == "0.1");
    print("ok\n");
}
