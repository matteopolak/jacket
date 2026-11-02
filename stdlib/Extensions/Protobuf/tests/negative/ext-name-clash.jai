// test: compile_fail
#import "Basic";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto2'; message B { extensions 100 to 110; optional int32 x = 1; extend B { optional int32 x = 100; } }");

main :: () {}
