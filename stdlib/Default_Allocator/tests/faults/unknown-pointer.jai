// test: only(.X64) args("unknown-pointer") should_fail("requires a live owned pointer")
// Asks for the usable size of a pointer the allocator never handed out. `usable_size` exists only
// in the heap backend (x64).
#load "../probes/heaps/main.jai";
