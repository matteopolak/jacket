// `internal/bindings/curl-api` loads the declarations; the library's own code imports it too,
// so a project that ships its own `Curl` module never reaches the library internals.
using Declarations :: #import "internal/bindings/curl-api";
