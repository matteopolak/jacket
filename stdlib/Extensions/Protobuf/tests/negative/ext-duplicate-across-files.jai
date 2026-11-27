// test: compile_fail("clash.proto:3:15: error[metaprogram-report]: duplicate extension number")
#import "Basic";
Proto :: #import "Extensions/Protobuf";
Api :: #import "Extensions/Protobuf/Schema"(PATH = #run Proto.beside("../fixtures/extensions/clash.proto"));

main :: () {}
