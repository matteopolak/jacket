// Reviewed baked aliases must retain the canonical Basic formatter and defaults.
using #import "Basic";
A :: #import "Android/Toolchain";

main :: () {
    for value: 0..255 {
        plain := sprint("%", A.fh(value));
        expected := sprint("%", formatInt(value, base = 16));

        assert(plain == expected);
        free(plain);
        free(expected);
        two := sprint("%", A.fh2(value));

        expected = sprint("%", formatInt(value, minimum_digits = 2, base = 16));
        assert(two == expected);
        free(two);
        free(expected);
        four := sprint("%", A.fh4(value));

        expected = sprint("%", formatInt(value, minimum_digits = 4, base = 16));
        assert(four == expected);
        free(four);
        free(expected);
        eight := sprint("%", A.fh8(value));

        expected = sprint("%", formatInt(value, minimum_digits = 8, base = 16));
        assert(eight == expected);
        free(eight);
        free(expected);
    }

    wrapped := A.fh8(cast(u64) 0xffffffffffffffff);
    ordinary := formatInt(cast(u64) 0xffffffffffffffff, minimum_digits = 8, base = 16);

    assert(type_of(wrapped) == type_of(ordinary));
    assert(type_info(type_of(wrapped)).name == "FormatInt");
    assert(wrapped.base == 16 && wrapped.minimum_digits == 8);
    named := sprint("%", A.fh(31, minimum_digits = 7, padding = #char "X"));
    expected := sprint("%", formatInt(31, base = 16, minimum_digits = 7, padding = #char "X"));

    assert(named == expected);
    free(named);
    free(expected);
    grouped := sprint("%", A.fh4(0x123456, digits_per_comma = 2, comma_string = "_"));

    expected = sprint("%", formatInt(0x123456, minimum_digits = 4, base = 16, digits_per_comma = 2, comma_string = "_"));
    assert(grouped == expected);
    free(grouped);
    free(expected);
    print("ok\n");
}
