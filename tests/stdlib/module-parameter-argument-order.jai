// rules: modparam.3
// Equal module-parameter values share an instance even when the named arguments are
// written in a different order.
#import "Basic";

A :: #import "Rules_Params"(LOUD = true, COUNT = 2);
B :: #import "Rules_Params"(COUNT = 2, LOUD = true);

main :: () {
    A.bump();
    assert(B.bump() == 2, "same values in another order must share the instance");
    print("ok\n");
}
