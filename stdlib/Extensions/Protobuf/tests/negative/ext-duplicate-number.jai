// test: compile_fail
#import "Basic";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto2'; message B { extensions 100 to 110; } extend B { optional int32 x = 100; } extend B { optional int32 y = 100; }");

main :: () {}
