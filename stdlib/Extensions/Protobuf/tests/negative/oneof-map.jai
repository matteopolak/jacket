// test: compile_fail
#import "Basic";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message M { oneof a { map<string,int32> b=1; } }");

main :: () {}
