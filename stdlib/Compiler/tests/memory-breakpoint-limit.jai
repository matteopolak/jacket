// test: should_fail("compiler memory breakpoints are unavailable")
C :: #import "Compiler";

#run C.compiler_set_memory_breakpoint(null);
main :: () {}
