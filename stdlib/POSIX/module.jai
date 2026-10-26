#assert OS == .LINUX || OS == .MACOS || OS == .ANDROID || OS == .IOS, "POSIX requires its documented native platform";

// `internal/bindings/posix-api` loads the declarations; the library's own code imports it too,
// so a project that ships its own `POSIX` module never reaches the library internals.
using Declarations :: #import "internal/bindings/posix-api";
