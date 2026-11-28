RPM :: #import "rpmalloc";

libc :: #library,system "c";

#import "internal/libc";

abort :: c_abort;

main :: () {
    if RPM.initialize() != 0 abort();
    if RPM.config().page_size <= 0 || RPM.config().span_size < 4096 abort();
    RPM.finalize();
}
