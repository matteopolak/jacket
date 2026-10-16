RPM :: #import "rpmalloc";

libc :: #library,system "c";
abort :: () #foreign libc;

main :: () {
    if RPM.initialize() != 0 abort();
    if RPM.config().page_size <= 0 || RPM.config().span_size < 4096 abort();
    RPM.finalize();
}
