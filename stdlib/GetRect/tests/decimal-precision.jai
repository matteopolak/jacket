#import "Basic";
G :: #import "GetRect";

Bits :: union {
    f: float64;
    u: u64;
}

main :: () {
    theme: G.Slider_Theme;
    state: G.Slider_State;
    ok, value := G.slider_parse_text_float_default("1.000000000000000166533453693773481063544750213623046875", *theme, *state);
    result: Bits;
    result.f = value;
    assert(ok && result.u == 0x3ff0000000000001, "UI parser must round rather than truncate significant digits");
    ok, value = G.slider_parse_text_float_default("4.9406564584124654417656879286822137236505980e-324", *theme, *state);
    result.f = value;
    assert(ok && result.u == 1, "UI parser must preserve a minimum subnormal");
    ok, value = G.slider_parse_text_float_default("-0", *theme, *state);
    result.f = value;
    assert(ok && result.u == 0x8000000000000000);
    ok, value = G.slider_parse_text_float_default(" 1.25 \t", *theme, *state);
    assert(ok && value == 1.25);
    ok, value = G.slider_parse_text_float_default("1.25 junk", *theme, *state);
    assert(!ok);
    theme.decimals = 20;
    formatted := G.slider_format_text_float_default("", "", 1.25, *theme, *state, .DISPLAY);

    defer free(formatted);
    assert(formatted == "1.25000000000000000000", "requested decimal precision must not be capped at twelve");
    print("canonical decimal precision ok\n");
}
