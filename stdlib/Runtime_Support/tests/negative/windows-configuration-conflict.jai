// Check the enabled Windows startup path without requiring a native cross-linker.
#import "Basic";
R :: #import "Runtime_Support"(DEFINE_SYSTEM_ENTRY_POINT = false, DEFINE_INITIALIZATION = true, ENABLE_BACKTRACE_ON_CRASH = true);

main :: () {
    ctx := R.__jai_runtime_init(0, null);
    assert(ctx != null);
    R.__jai_runtime_fini(ctx);
}
