// An installed diagnostic assertion must not swallow a genuine failure.
#import "Basic";
D :: #import "Debug";

main :: () {
    D.init();
    assert(false, "original failing assertion");
}
