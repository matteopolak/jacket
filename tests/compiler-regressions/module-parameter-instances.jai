// A module imported with different parameters is a separate instance: a plain import after a
// configured one does not join it.
// rules: modparam.13
#import "Basic";

Loud :: #import "Instance_Flag"(LOUD = true);
Quiet :: #import "Instance_Flag";

main :: () {
    assert(Loud.is_loud());
    assert(!Quiet.is_loud());
    print("ok\n");
}
