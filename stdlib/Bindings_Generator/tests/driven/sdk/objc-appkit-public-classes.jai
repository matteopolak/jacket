// Independently authored reduction of two hash-verified application consumers.
// Reviewed AppKit class identities; no SDK allocation is needed for this reduction.
AppKit :: #import "Objective_C/AppKit";
Same :: #import "Objective_C/AppKit";

application :: (value: *AppKit.NSApplication) -> *Same.NSApplication {
    return value;
}

context :: (value: *AppKit.NSOpenGLContext) -> *Same.NSOpenGLContext {
    return value;
}

main :: () {}
