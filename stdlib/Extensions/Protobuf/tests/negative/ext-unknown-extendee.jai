// test: compile_fail
#import "Basic";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto2'; extend Missing { optional int32 x = 100; }");

main :: () {}
