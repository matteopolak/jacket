A :: #import "Objective_C/AppKit";
GL :: #import "GL";

// Type-check the actual producer return against the public consumer.
identity :: () -> *A.NSOpenGLContext {
    return GL.nsgl_create_context(3, 2);
}

main :: () {}
