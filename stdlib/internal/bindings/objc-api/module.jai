// The Objective-C runtime declarations as the library itself sees them: a project that ships its
// own Objective_C module never reaches the library internals through it.
#assert OS == .MACOS || OS == .IOS, "These bindings require their native platform";

#load "../../../Objective_C/declarations.jai";
#load "../../../Objective_C/wrappers.jai";
