#assert OS == .WINDOWS, "Windows requires its documented native platform";

// The declarations and wrappers are loaded by `internal/bindings/windows-api`, the one copy the
// library's own code imports too, so a project that ships its own `Windows` module replaces
// this module without reaching the library internals.
using Declarations :: #import "internal/bindings/windows-api";
