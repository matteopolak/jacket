// rules: modparam.1 modparam.2 modparam.3 modparam.4 modparam.6 modparam.7 modparam.8 modparam.9 modparam.10 modparam.11 modparam.12
// Module parameters, through tests/stdlib/modules/Rules_Params.
#import "Basic";

Color :: enum {
    RED;
    GREEN;
}

Pair :: struct {
    a: s32;
    b: s32;
}

One :: #import "Rules_Params";
Two :: #import "Rules_Params"(LOUD = true, COUNT = 2);
Two_Again :: #import "Rules_Params"(LOUD = true, COUNT = 2);
Traced :: #import "Rules_Params"()(TRACE = true);
Fancy :: #import "Rules_Params"(STYLE = .FANCY, TAG = Color.GREEN);
Paired :: #import "Rules_Params"(TAG = Pair.{ 3, 4 });
Ignored :: #import "Rules_Params"(NO_SUCH_PARAMETER = 5);
Wide :: #import "Rules_Params"(WIDTH = 2);
Narrow :: #import "Rules_Params"(WIDTH = cast(u8) 9);

as_u16 :: (x: u16) -> u16 {
    return x;
}

main :: () {
    // modparam.1: arguments are given in the declaration's two-list shape.
    assert(Two.greeting("bob") == "HELLO bob;HELLO bob;", "configured greeting");
    assert(One.greeting("bob") == "hello bob;", "default greeting");

    // modparam.2: different first-list values make different instances.
    assert(One.bump() == 1 && Two.bump() == 1 && One.bump() == 2, "separate instances");
    // modparam.3: equal values share one instance.
    assert(Two_Again.bump() == 2, "equal arguments share an instance");

    // modparam.4: program parameters configure the one shared instance; a plain import sees
    // the configured value.
    assert(One.trace_enabled() && Traced.trace_enabled(), "program parameter is shared");
    assert(Two.trace_enabled() && Fancy.trace_enabled(), "program parameter reaches every instance");
    assert(Traced.bump() == 3, "program-parameter import joins the default instance");

    // modparam.6: a scalar argument takes the parameter's written type.
    assert(Two.count_type() == s64 && Two.COUNT == 2, "typed parameter");

    // modparam.7: enum and aggregate arguments keep its type; `.Member` takes the parameter's type.
    assert(Fancy.style_is_fancy() && !One.style_is_fancy(), "`.Member` argument");
    assert(Fancy.tag_type() == Color && Fancy.TAG == .GREEN, "enum argument keeps its type");
    assert(Paired.tag_type() == Pair && Paired.TAG.b == 4, "aggregate argument keeps its type");

    // modparam.8: a parameter without a written type given an untyped number stays untyped.
    assert(as_u16(Wide.WIDTH) == 2, "untyped argument converts to u16");
    small: u8 = Wide.WIDTH;
    half: float32 = Wide.WIDTH;
    assert(small == 2 && half == 2, "untyped argument converts to u8 and float32");

    // modparam.9: a typed argument keeps its type.
    assert(type_of(Narrow.WIDTH) == u8 && Narrow.WIDTH == 9, "typed argument");

    // modparam.10: parameters are constants in the module's #load-ed files too.
    assert(Wide.width() == 2 && Narrow.width() == 9, "parameter read from a loaded file");

    // modparam.11: an unknown argument name is ignored.
    assert(Ignored.greeting("x") == "hello x;", "unknown argument ignored");

    // modparam.12: names declared in the trailing block may be used by defaults.
    assert(One.width() == 40 && One.WIDTH == 40, "default from the trailing block");
    print("ok\n");
}
