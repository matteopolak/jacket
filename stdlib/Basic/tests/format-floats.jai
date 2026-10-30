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
    // Exact ties round to even at every precision; carries move the point or the exponent.
    tiny_bits: u64 = 1;
    tiny := (cast(*float64) *tiny_bits).*;

    assert(sprint("%", formatFloat(cast(float64) 0.125, trailing_width = 2, zero_removal = .NO)) == "0.12");
    assert(sprint("%", formatFloat(cast(float64) 0.375, trailing_width = 2, zero_removal = .NO)) == "0.38");
    assert(sprint("%", formatFloat(cast(float64) 2.5, trailing_width = 0)) == "2");
    assert(sprint("%", formatFloat(cast(float64) 3.5, trailing_width = 0)) == "4");
    assert(sprint("%", formatFloat(cast(float64) 1.0005, trailing_width = 3, zero_removal = .NO)) == "1.000");
    assert(sprint("%", formatFloat(cast(float64) 0.0005, trailing_width = 3, zero_removal = .NO)) == "0.001");
    assert(sprint("%", formatFloat(cast(float64) 999.9995, trailing_width = 3)) == "1000    ");
    assert(sprint("%", formatFloat(cast(float64) 9.9999999, trailing_width = 6, zero_removal = .NO)) == "10.000000");
    assert(sprint("%", formatFloat(cast(float64) 9.95, trailing_width = 1, mode = .SCIENTIFIC)) == "9.9e+00");
    assert(sprint("%", formatFloat(cast(float64) 9.96, trailing_width = 1, mode = .SCIENTIFIC)) == "1e+01  ");
    assert(sprint("%", formatFloat(cast(float64) 0.00001, mode = .SHORTEST)) == "1e-05");
    assert(sprint("%", formatFloat(tiny, mode = .SHORTEST)) == "5e-324");
    assert(sprint("%", formatFloat(tiny, trailing_width = 3, zero_removal = .NO)) == "0.000");
    assert(sprint("%", formatFloat(tiny, trailing_width = 16, mode = .SCIENTIFIC)) == "4.9406564584124654e-324");
    assert(sprint("%", formatFloat(cast(float64) 1.7976931348623157e308, mode = .SHORTEST)) == "1.7976931348623157e+308");
    assert(sprint("%", formatFloat(cast(float64) 1.0e22)) == "10000000000000000000000");
    assert(sprint("%", formatFloat(cast(float64) 1.0e23)) == "99999999999999991600000");
    assert(sprint("%", formatFloat(cast(float64) 9223372036854775808.0)) == "9223372036854775810");
    assert(sprint("%", formatFloat(cast(float64) 123456.789, trailing_width = 25, zero_removal = .NO)) == "123456.7890000000043073669075966");
    assert(sprint("%", formatFloat(cast(float64) 123456.789, trailing_width = 30, zero_removal = .NO)) == "123456.789000000004307366907596588135");
    assert(sprint("%", formatFloat(cast(float64) 0.1, trailing_width = 18, mode = .SCIENTIFIC)) == "1.000000000000000056e-01");
    assert(sprint("%", formatFloat(cast(float64) 0.1, trailing_width = 20, mode = .SCIENTIFIC)) == "1.00000000000000005551e-01");
    assert(sprint("%", formatFloat(cast(float32) 0.1, trailing_width = 10, zero_removal = .NO)) == "0.1000000015");
    assert(sprint("%", formatFloat(cast(float64) 1.0e15 + 0.3, mode = .SHORTEST)) == "1000000000000000.2");
    assert(sprint("%", formatFloat(cast(float64) 18446744073709551616.0, trailing_width = 2)) == "18446744073709551616   ");
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
