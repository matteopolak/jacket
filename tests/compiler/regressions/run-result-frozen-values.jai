// A constant initialised by `#run` is frozen into the program: floats of both widths, a
// resizable array, an `Any` and a pointer to type information all survive. `#insert` also works
// in expression position, with a string or with `Code`.
#import "Basic";

Settings :: struct {
    scale: float32;
    ratio: float64;
    values: [..] int;
    boxed: Any;
    info: *Type_Info;
}

make_settings :: () -> Settings {
    s: Settings;
    s.scale = 1.5;
    s.ratio = 0.25;
    array_add(*s.values, 3);
    array_add(*s.values, 4);
    s.boxed = 42;
    s.info = type_info(int);

    return s;
}

SCALE :: #run cast(float32) 2.5;
RATIO :: #run cast(float64) 0.125;
SETTINGS :: #run make_settings();

expr_code :: ($c: Code) -> int #expand {
    return #insert c;
}

main :: () {
    assert(SCALE == 2.5);
    assert(RATIO == 0.125);
    assert(SETTINGS.scale == 1.5);
    assert(SETTINGS.ratio == 0.25);
    assert(SETTINGS.values.count == 2);
    assert(SETTINGS.values[0] + SETTINGS.values[1] == 7);
    assert(SETTINGS.boxed.type == type_info(int));
    assert(<<cast(*int) SETTINGS.boxed.value_pointer == 42);
    assert(SETTINGS.info.type == .INTEGER);

    n := 5;

    assert((#insert "n + 1;") == 6);
    assert(expr_code(n * 2) == 10);
    print("ok\n");
}
