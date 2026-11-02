// test: compile_fail
#import "Basic";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message M { reserved \"a\"; int32 a=1; }");

main :: () {}
