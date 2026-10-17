// Reviewed generic callback: invalid mode reports through active assertions.
RPM :: #import "rpmalloc";

main :: () {
    invalid: s32 = 999;
    RPM.rpmalloc_allocator_proc(cast(Allocator_Mode) invalid, 0, 0, null, null);
}
