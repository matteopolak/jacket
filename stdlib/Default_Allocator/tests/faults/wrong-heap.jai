// test: only(.X64) args("wrong-heap") should_fail("invalid pointer or source extent")
// Frees a block through a heap that does not own it. Only the heap backend (x64) knows which
// heap a block belongs to; the C-backed fallback has no heaps.
#load "../probes/heaps/main.jai";
