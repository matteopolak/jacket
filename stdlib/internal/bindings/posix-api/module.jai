// The POSIX declarations as the library itself sees them: a project that ships its own POSIX
// module never reaches the library internals through it.
#assert OS == .LINUX || OS == .MACOS || OS == .ANDROID || OS == .IOS, "These bindings require their native platform";

#load "../../../POSIX/declarations.jai";
#load "../../../POSIX/wrappers.jai";
