#import "Basic";
// test: compile_fail
#load "fault-setup.jai";

#assert invalid_index(0, false, false);
main :: () {}
