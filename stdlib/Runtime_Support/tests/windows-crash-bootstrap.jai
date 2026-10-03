// Default configured initializer; explicit ON/OFF entry options have workspace tests.
#import "Basic";
R :: #import "Runtime_Support";

main :: () {
    ctx := R.__jai_runtime_init(0, null);
    assert(ctx != null);
    R.__jai_runtime_fini(ctx);
}
