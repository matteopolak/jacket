GL :: #import "GL";
WT :: #import "Window_Type";

// Compile-only dependency reproducer. Never run with an invalid native window.
main :: () {
    GL.gl_create_context(WT.INVALID_WINDOW);
}
