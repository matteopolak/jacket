// test: compile_fail
#import "Basic";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto2'; message B { extensions 19000 to 19999; } extend B { optional int32 x = 19500; }");

main :: () {}
