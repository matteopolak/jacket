// A named import whose module argument is computed by `#run` code that needs Context
// (`tprint` allocates through it). Building Context loads every named import, and this one
// must be skipped while its own arguments are evaluated rather than reported as circular.
#import "Basic";

Settings :: #import "Run_Argument_Context"(DIR = #run tprint("%/%", "root", "proto"));

main :: () {
    assert(Settings.dir() == "root/proto");
    context.run_argument_extra = 3;
    assert(context.run_argument_extra == 3);
    print("ok\n");
}
