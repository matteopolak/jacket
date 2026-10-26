// The Windows API declarations as the library itself sees them: a project that ships its own
// Windows module never reaches the library internals through it.
#assert OS == .WINDOWS, "These bindings require their native platform";

#load "../../../Windows/declarations.jai";
#load "../../../Windows/wrappers.jai";
