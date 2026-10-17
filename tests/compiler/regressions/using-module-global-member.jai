// `using Module.global;` at file scope brings the members of another module's global variable
// into scope (Simp does this with `GL.gl_procs`); the global was read as a constant before.
// rules: scope.9
#import "Basic";
Table :: #import "Using_Global";

using Table.procs;

main :: () {
    Table.load_procs();
    assert(answer() == 42);
}
