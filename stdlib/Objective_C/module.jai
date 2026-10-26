#assert OS == .MACOS || OS == .IOS, "Objective_C requires its documented native platform";

// `internal/bindings/objc-api` loads the declarations; the library's own code imports it too,
// so a project that ships its own `Objective_C` module never reaches the library internals.
using Declarations :: #import "internal/bindings/objc-api";
