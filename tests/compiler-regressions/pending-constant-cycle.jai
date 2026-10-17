// A module-level `#if` whose condition names an exported constant is retried, not dropped, when
// the lookup that expands it is itself part of resolving that constant.
#import "Basic";
#import "Pending_Constant_Cycle";

main :: () {
    assert(use_lock() == 1);
    print("ok\n");
}
