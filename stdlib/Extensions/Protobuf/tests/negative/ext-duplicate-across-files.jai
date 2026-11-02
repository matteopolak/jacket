// test: compile_fail
#import "Basic";
Proto :: #import "Extensions/Protobuf";
Api :: #import "Extensions/Protobuf/Schema"(PATH = #run Proto.beside("../fixtures/extensions/clash.proto"));

main :: () {}
