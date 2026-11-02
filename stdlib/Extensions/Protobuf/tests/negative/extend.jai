// test: compile_fail
#import "Basic";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; extend M { int32 a=1; }");

main :: () {}
