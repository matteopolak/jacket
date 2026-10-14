#import "Basic";
#import "String";

main :: () {
    string_tests();
    assert(wildcard_match("abcdef", "a*[c-e]?f"));
    assert(wildcard_match("", "***"));
    assert(wildcard_match("a*b", "a[*]b"));
    assert(!wildcard_match("ab", "a?b"));
    assert(!wildcard_match("a", "[b-d]"));
    assert(wildcard_match("zzabac", "*ab?c"));
    number: s64;
    flag: bool;
    text: string;
    real: float;

    assert(scan2("42 true \"words here\" 1.25%", "%d %b %s %f%%", *number, *flag, *text, *real));
    assert(number == 42 && flag && text == "words here" && real == 1.25);
    assert(!scan2("7 no", "%i %b", *number, *flag));
    assert(number == 7);
    precise: float64;

    assert(scan2("1.23456789012345", "%f", *precise));
    assert(precise == 1.23456789012345);
    small: u8 = 11;

    assert(!scan2("256", "%d", *small) && small == 11);
    assert(scan2("17", "%i", *small) && small == 17);
    null_integer: *s64;

    assert(!scan2("1", "%d", null_integer));
    empty: Any;

    assert(!scan2("1", "%d", empty));
    ok, values := scan("%d %s", "-8 \"yes\"");

    assert(ok && values.count == 2);
    assert((cast(*s64) values[0].value_pointer).* == -8);
    assert((cast(*string) values[1].value_pointer).* == "yes");
    failed, partial := scan("%b !%d", "true ?5");

    assert(!failed && partial.count == 1);
    percent, unused := scan("%%", "%");

    assert(!percent && unused.count == 0);
    print("ok\n");
}
