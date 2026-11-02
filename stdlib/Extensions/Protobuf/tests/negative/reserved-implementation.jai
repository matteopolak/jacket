// test: compile_fail
#import "Basic";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message M { int32 a=19000; }");

main :: () {}
