// An installed diagnostic assertion must not swallow a genuine failure.
#import "Basic";
D :: #import "Debug";

main :: () {
    D.init();
    D.set_report_mode(true);
    assert(false, "original failing assertion");
}
