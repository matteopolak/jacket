#import "Basic";
G :: #import "developer-tools-macro-enums-generated";

#assert size_of(G.Options) == 4;
#assert size_of(G.WideOptions) == 8;

main :: () {
    assert(G.Options.INTEGER == G.Options.ORIGINAL_INTEGER);
    assert(G.Options.NEGATIVE == G.Options.ORIGINAL_NEGATIVE);
    assert(G.macro_native_mode(.INTEGER) == 19);
    assert(G.macro_native_mode(.NEGATIVE) == -35);
    assert(G.macro_native_mode(.EXPRESSION) == 57);
    assert(cast(u64) G.WideOptions.UNSIGNED == 0xffffffffffffffff);
    assert(G.WideOptions.UNSIGNED == G.WideOptions.ORIGINAL_UNSIGNED);
    assert(G.macro_native_unsigned(.UNSIGNED) == 0xfffffffffffffffe);
    print("ok\n");
}
