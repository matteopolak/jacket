// test: compile_fail
#import "Basic";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "edition='2023'; message M { int32 x=1 [features.field_presence=BAD]; }");

main :: () {}
