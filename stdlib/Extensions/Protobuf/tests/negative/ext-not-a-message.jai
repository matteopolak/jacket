// test: compile_fail
#import "Basic";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto2'; enum E { A = 0; } extend E { optional int32 x = 100; }");

main :: () {}
