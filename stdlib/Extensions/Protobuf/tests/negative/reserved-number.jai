// test: compile_fail
#import "Basic";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message M { reserved 1 to 3; int32 a=2; }");

main :: () {}
