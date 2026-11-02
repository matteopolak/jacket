// test: compile_fail
#import "Basic";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto2'; message B { extensions 100 to 110; } extend B { map<string, int32> m = 100; }");

main :: () {}
