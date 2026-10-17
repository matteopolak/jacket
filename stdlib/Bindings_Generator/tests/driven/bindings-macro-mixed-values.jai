#import "Basic";
G :: #import "developer-tools-macro-mixed-generated";

// This exact mixed pair is a generation/value control, not a native ABI claim.
#assert G.MixedValues.INTEGER == 17;
#assert cast(u64) G.MixedValues.UNSIGNED == 0xffffffffffffffff;

main :: () {
    assert(G.MixedValues.INTEGER == G.MixedValues.ORIGINAL_INTEGER);
    assert(G.MixedValues.UNSIGNED == G.MixedValues.ORIGINAL_UNSIGNED);
    print("ok\n");
}
