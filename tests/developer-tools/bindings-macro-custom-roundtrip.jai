#import "Basic";
G :: #import "developer-tools-macros-custom-generated";

main :: () {
    assert(G.RENAMED_INTEGER == 17 && G.ORIGINAL_NEGATIVE == -37);
    assert(G.ORIGINAL_UNSIGNED == 0xffffffffffffffff && type_of(G.ORIGINAL_UNSIGNED) == u64);
    assert(G.ORIGINAL_REAL == 18.5 && type_of(G.ORIGINAL_REAL) == float32);
    assert(G.ORIGINAL_EXPRESSION == 55);
    assert(G.macro_native_sum() == G.RENAMED_INTEGER + G.ORIGINAL_EXPRESSION);
    print("ok\n");
}
