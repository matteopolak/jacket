// A body that fails must be reported, not retried forever: each retry re-creates its nested
// procedures, and lowering those is not progress.
#import "Basic";
main :: () {
    helper :: (n: int) { print("%\n", n); }
    helper(1);
    print("%\n", undefined_name_here);
}
