#import "Basic";
M :: #import "Math";
H :: #import "Float16";
G :: #import "GetRect";
S :: #import "Sloppy_Math";

B64 :: union {
    f: float64;
    u: u64;
}

main :: () {
    assert(S.min_of_type(s64) == M.S64_MIN && S.max_of_type(s64) == M.S64_MAX);
    assert(S.min_of_type(s8) == -128 && S.max_of_type(s8) == 127 && S.max_of_type(u8) == 255);
    zero: B64;
    zero.u = 0x8000_0000_0000_0000;
    assert(M.signbit(zero.f) == 1);
    assert(M.signbit(M.sqrt(zero.f)) == 1);
    assert(M.atan2(zero.f, cast(float64) -1) == -M.PI64);
    m, e := M.frexp(M.FLOAT64_MIN);

    assert(m == 0.5 && e == -1021);
    tiny: B64;
    tiny.u = 1;
    m, e = M.frexp(tiny.f);
    assert(m == 0.5 && e == -1073);
    assert(M.ldexp(m, e) == tiny.f);
    assert(M.ldexp(cast(float64) 1, -1074) == tiny.f);
    assert(M.ldexp(cast(float64) 1, -1075) == 0);
    assert(M.ldexp(cast(float64) 3, -1075) == 2 * tiny.f);
    assert(M.isinf(M.ldexp(cast(float64) 1, 1024)));
    assert(M.isnan(M.sqrt(cast(float64) -1)) && M.isnan(M.log(cast(float64) -1)));
    assert(M.log(cast(float64) 0) == -M.FLOAT64_INFINITY);
    assert(M.pow(cast(float64) -2, cast(float64) 3) == -8);
    assert(M.pow(cast(float64) -2, cast(float64) 2) == 4);
    assert(M.pow(M.FLOAT64_NAN, cast(float64) 0) == 1);
    assert(M.isnan(M.pow(cast(float64) -2, cast(float64) 0.5)));
    v := M.xyz(1e-30, 0, 0);
    n := M.length(v);

    assert(n > 0);
    v = M.normalize(v);
    assert(v.x == 1);
    huge := M.xyz(1e30, 1e30, 0);

    assert(M.is_finite(M.length(huge)));
    nan, inf := M.is_nan_is_inf(M.FLOAT32_NAN);

    assert(nan && !inf);
    nan, inf = M.is_nan_is_inf(M.FLOAT64_INFINITY);
    assert(!nan && inf);
    assert(cast(u16) H.float32_to_float16(M.FLOAT32_INFINITY) == 0x7c00);
    assert((cast(u16) H.float32_to_float16(-M.FLOAT32_NAN) & 0x8000) != 0);
    b: String_Builder;
    v2 := M.xy(1, 2);
    any: Any = v2;

    assert(M.math_struct_printer(*b, any, null));
    assert(builder_string_length(*b) > 0);
    free_buffers(*b);
    theme: G.Slider_Theme;
    state: G.Slider_State;
    text := G.slider_format_text_float_default("", "", 1e100, *theme, *state, .DISPLAY);
    ok, value := G.slider_parse_text_float_default(text, *theme, *state);

    assert(ok && M.abs((value - cast(float64) 1e100) / cast(float64) 1e100) < cast(float64) 1e-12);
    free(text);
    print("numeric edges ok\n");
}
