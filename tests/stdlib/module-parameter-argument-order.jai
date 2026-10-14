// rules: modparam.3
// Module-parameter argument lists preserve written order in Jai. Equal values
// supplied in a different order create distinct instances.
#import "Basic";

A :: #import "Rules_Params"(LOUD = true, COUNT = 2);
B :: #import "Rules_Params"(COUNT = 2, LOUD = true);

main :: () {
    assert(A.bump() == 1, "first instance starts independently");
    assert(B.bump() == 1, "reordered arguments create a separate instance");
    assert(A.bump() == 2 && B.bump() == 2, "each instance retains its own globals");
    print("ok\n");
}
