// Target source regression; requires a real loaded context when executed.
G :: #import "GL";

main :: () {
    G.gl_enable_debug_output();
    G.gl_enable_debug_output(break_on_error = true);
}
