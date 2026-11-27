// test: only(.X64) args("wrong-size") should_fail("invalid pointer or source extent")
// Resizes a block while claiming it was larger than it is. The C-backed fallback checks the same
// claim; `../stack-addresses.jai` covers it on every target.
#load "../probes/heaps/main.jai";
