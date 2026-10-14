#import "Basic";
C :: #import "Runtime_Support_Crash_Handler";
P :: #import "POSIX";

main :: () {
    C.init();
    C.init();
    assert(P.raise(P.SIGSEGV) == 0);
    assert(false, "fatal signal unexpectedly returned");
}
