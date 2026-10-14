#import "Basic";

main :: () {
    assert(sprint("%", cast(float64) 1.25) == "1.25");
    assert(sprint("%", formatFloat(cast(float64) 1.23456789)) == "1.234568");
    assert(sprint("%", formatFloat(cast(float64) 1.25, trailing_width = 4)) == "1.25  ");
    assert(sprint("%", formatFloat(cast(float64) 1, trailing_width = 3)) == "1    ");
    assert(sprint("%", formatFloat(cast(float64) 1, trailing_width = 3, zero_removal = .ONE_ZERO_AFTER_DECIMAL)) == "1.0  ");
    assert(sprint("%", formatFloat(cast(float64) 1, zero_removal = .NO)) == "1.000000");
    assert(sprint("%", formatFloat(cast(float64) 1.25, trailing_width = 4, width = 10)) == "    1.25  ");
    assert(sprint("%", formatFloat(cast(float64) 12.5, mode = .SCIENTIFIC)) == "1.25e+01");
    customized := formatFloat(cast(float64) 12345.5);
    customized.thousand_separators = true;
    customized.thousand_separator_character = 95;
    customized.decimal_separator_character = 44;
    customized.positive_number_prefix = .PLUS;
    assert(sprint("%", customized) == "+12_345,5");
    assert(sprint("%", formatFloat(cast(float64) 0.1, mode = .SHORTEST)) == "0.1");
    assert(sprint("%", formatFloat(cast(float32) 0.1, mode = .SHORTEST)) == "0.1");
    assert(sprint("%", formatFloat(cast(float64) 1.23456789012345, mode = .SHORTEST)) == "1.23456789012345");
    power_bits: u64 = 0x0060000000000000; // 2^-1017 has an asymmetric rounding interval.
    power := (cast(*float64) *power_bits).*;

    assert(sprint("%", formatFloat(power, mode = .SHORTEST)) == "7.120236347223045e-307");
    single_power_bits: u32 = 0x0f800000; // 2^-96.
    single_power := (cast(*float32) *single_power_bits).*;

    assert(sprint("%", formatFloat(single_power, mode = .SHORTEST)) == "1.2621775e-29");
    vectors: [8] u64 = .[1, 0x000fffffffffffff, 0x0010000000000000, 0x3fefffffffffffff,
                          0x3ff0000000000001, 0x7fefffffffffffff, 0x8000000000000001, 0xffefffffffffffff];
    for bits: vectors {
        value := (cast(*float64) *bits).*;
        text := sprint("%", formatFloat(value, mode = .SHORTEST));

        defer free(text);
        parsed, success, remainder := string_to_float64(text);

        assert(success && remainder.count == 0);
        assert((cast(*u64) *parsed).* == bits);
    }

    negative_zero_bits: u64 = 0x8000000000000000;
    negative_zero := (cast(*float64) *negative_zero_bits).*;

    assert(sprint("%", negative_zero) == "-0");
    infinity_bits: u64 = 0x7ff0000000000000;
    infinity := (cast(*float64) *infinity_bits).*;

    assert(sprint("%", infinity) == "Inf");
    nan_bits: u64 = 0x7ff8000000000000;
    nan := (cast(*float64) *nan_bits).*;

    assert(sprint("%", nan) == "NaN");
    builder: String_Builder;

    defer free_buffers(*builder);
    assert(stb_print_float(*builder, formatFloat(cast(float64) 1.25)) == 4);
    print("ok\n");
}
