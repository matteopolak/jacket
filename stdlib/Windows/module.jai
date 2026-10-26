#assert OS == .WINDOWS, "Windows requires its documented native platform";

// `internal/bindings/windows-api` loads the declarations; the library's own code imports it too,
// so a project that ships its own `Windows` module never reaches the library internals.
using Declarations :: #import "internal/bindings/windows-api";
