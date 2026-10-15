// Reviewed generic callback: the default heap cannot be destroyed.
RPM :: #import "rpmalloc";

main :: () {
    RPM.rpmalloc_allocator_proc(.DESTROY_HEAP, 0, 0, null, null);
}
