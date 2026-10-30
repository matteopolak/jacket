// test: skip_on(.WASM, mode = .NATIVE, reason = "a program that works only at compile time has no wasm output to run")
// A program with no `main` that does its work at compile time: what the compiler prints is its
// standard output.
#import "Basic";

#run print("computed at compile time\n");
