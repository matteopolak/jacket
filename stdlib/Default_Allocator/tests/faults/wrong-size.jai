// test: only(.X64) args("wrong-size") should_fail("invalid pointer or source extent")
// Resizes a block while claiming it was larger than it is. Only the heap backend (x64) records
// block sizes; the C-backed fallback cannot check the claim.
#load "../probes/heaps/main.jai";
