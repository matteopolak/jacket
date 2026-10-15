#import "Basic";
C :: #import "Context_Probe";
main :: () {
    assert(!context.independent_profiler_probe);
    C.set();
    assert(context.independent_profiler_probe);
    print("ok\n");
}
