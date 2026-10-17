#import "Basic";
G :: #import "developer-tools-macros-generated";

#assert G.ORIGINAL_LONG_BYTES == (ifx OS == .WINDOWS then 4 else 8);

main :: () {
    assert(G.ORIGINAL_INTEGER == 17 && G.ORIGINAL_NEGATIVE == -37);
    assert(G.ORIGINAL_UNSIGNED == 0xffffffffffffffff);
    assert(type_of(G.ORIGINAL_UNSIGNED) == u64);
    assert(G.ORIGINAL_REAL == 18.5 && type_of(G.ORIGINAL_REAL) == float32);
    assert(G.ORIGINAL_EXPRESSION == 55);
    assert(G.ORIGINAL_LONG_BYTES == 8);
    assert(G.macro_native_sum() == G.ORIGINAL_INTEGER + G.ORIGINAL_EXPRESSION);
    print("ok\n");
}
