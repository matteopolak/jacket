// test: compile_fail
#import "Basic";
#load "fault-setup.jai";

#assert invalid_index(0x7fffffffffffffff, true, true);
main :: () {}
