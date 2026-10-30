// test: compile_fail
#import "Basic";
#load "fault-setup.jai";

#assert invalid_index(0, false, false);
main :: () {}
