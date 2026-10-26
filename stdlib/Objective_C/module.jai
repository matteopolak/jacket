#assert OS == .MACOS || OS == .IOS, "Objective_C requires its documented native platform";

// The declarations and wrappers are loaded by `internal/bindings/objc-api`, the one copy the
// library's own code imports too, so a project that ships its own `Objective_C` module replaces
// this module without reaching the library internals.
using Declarations :: #import "internal/bindings/objc-api";
