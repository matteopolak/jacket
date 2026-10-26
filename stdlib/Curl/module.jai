// The declarations are loaded by `internal/bindings/curl-api`, the one copy the library's own
// code imports too, so a project that ships its own `Curl` module replaces this module without
// reaching the library internals.
using Declarations :: #import "internal/bindings/curl-api";
