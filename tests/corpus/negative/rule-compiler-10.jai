// rules: compiler.10 ws.5
// A metaprogram's compiler_report in the default ERROR mode fails the compile.
#import "Compiler";

#run compiler_report("rule-compiler-10: deliberate metaprogram error");

main :: () {}
