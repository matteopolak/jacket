#import "Basic";
G :: #import "developer-tools-pointer-defaults-generated";

main :: () {
    assert(G.pointer_default() == 17);
    value: s32 = 31;
    assert(G.pointer_default(*value) == 31);
    print("ok\n");
}
