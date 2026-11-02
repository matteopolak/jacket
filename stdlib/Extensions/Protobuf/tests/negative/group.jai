// test: compile_fail
#import "Basic";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message M { group G=1 { int32 a=2; } }");

main :: () {}
