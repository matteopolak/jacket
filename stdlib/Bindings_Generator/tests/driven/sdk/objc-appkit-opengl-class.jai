// Independent consumer reduction of the now-reviewed AppKit class identity.
AppKit :: #import "Objective_C/AppKit";
Same :: #import "Objective_C/AppKit";

context :: (value: *AppKit.NSOpenGLContext) -> *Same.NSOpenGLContext {
    return value;
}

main :: () {}
