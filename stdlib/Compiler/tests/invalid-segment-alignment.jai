// test: should_fail("alignment must be a positive power of two")
C :: #import "Compiler";

#run C.add_data_segment("invalid", alignment = 3);
main :: () {}
