// test: skip_on(.WASM)
// A program with no `main` that does its work at compile time: what the compiler prints is its
// standard output.
#import "Basic";

#run print("computed at compile time\n");
