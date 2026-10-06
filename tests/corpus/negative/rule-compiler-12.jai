// rules: compiler.12
// Marking a healthy workspace FAILED makes the compile fail.
#import "Compiler";

#run compiler_set_workspace_status(.FAILED);

main :: () {}
