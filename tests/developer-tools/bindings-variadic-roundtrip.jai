#import "Basic";
G :: #import "developer-tools-variadic-generated";
D :: #import "developer-tools-variadic-default";
S :: #import "developer-tools-variadic-signature";

main :: () {
    small: s8 = -2;
    unsigned: u16 = 60000;
    value: float32 = 1.25;
    pointed: s32 = 7;

    assert(G.original_variadic(5, small, unsigned, value, *pointed) == 60135);
    assert(D.original_variadic(5, small, unsigned, value, *pointed) == 60135);
    assert(G.original_variadic_numbers(0) == 0 && D.original_variadic_numbers(0) == 0);
    assert(G.original_variadic_numbers(2, cast(float32) 0.5, cast(float64) 1.25) == 1.75);
    assert(D.original_variadic_numbers(2, cast(float32) 0.5, cast(float64) 1.25) == 1.75);
    callback: S.Signature = G.original_get_variadic();

    assert(callback(5, small, unsigned, value, *pointed) == 60135);
    default_callback := D.original_get_variadic();

    assert(default_callback(5, small, unsigned, value, *pointed) == 60135);
    print("ok\n");
}
