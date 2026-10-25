#import "Extensions/Testing";

module_self_tests_pass :: () {
    string_tests();
} @test

wildcards_match_whole_strings :: () {
    expect(wildcard_match("abcdef", "a*[c-e]?f"));
    expect(wildcard_match("", "***"));
    expect(wildcard_match("a*b", "a[*]b"));
    expect(!wildcard_match("ab", "a?b"));
    expect(!wildcard_match("a", "[b-d]"));
    expect(wildcard_match("zzabac", "*ab?c"));
} @test

scan2_stores_each_conversion :: () {
    number: s64;
    flag: bool;
    text: string;
    real: float;

    expect(scan2("42 true \"words here\" 1.25%", "%d %b %s %f%%", *number, *flag, *text, *real));
    expect(number == 42 && flag && text == "words here" && real == 1.25);
    expect(!scan2("7 no", "%i %b", *number, *flag));
    expect_equal(number, 7);
    precise: float64;

    expect(scan2("1.23456789012345", "%f", *precise));
    expect_equal(precise, 1.23456789012345);
} @test

scan2_rejects_values_that_do_not_fit_or_have_no_target :: () {
    small: u8 = 11;

    expect(!scan2("256", "%d", *small) && small == 11);
    expect(scan2("17", "%i", *small) && small == 17);
    null_integer: *s64;

    expect(!scan2("1", "%d", null_integer));
    empty: Any;

    expect(!scan2("1", "%d", empty));
} @test

scan_returns_the_values_it_read :: () {
    ok, values := scan("%d %s", "-8 \"yes\"");

    expect(ok && values.count == 2);
    expect_equal((cast(*s64) values[0].value_pointer).*, -8);
    expect_equal((cast(*string) values[1].value_pointer).*, "yes");
    failed, partial := scan("%b !%d", "true ?5");

    expect(!failed && partial.count == 1);
    percent, unused := scan("%%", "%");

    expect(!percent && unused.count == 0);
} @test

#scope_file

#import "String";
