#assert OS == .LINUX || OS == .MACOS || OS == .ANDROID || OS == .IOS, "POSIX requires its documented native platform";

// The declarations and wrappers are loaded by `internal/bindings/posix-api`, the one copy the
// library's own code imports too, so a project that ships its own `POSIX` module replaces this
// module without reaching the library internals.
using Declarations :: #import "internal/bindings/posix-api";
