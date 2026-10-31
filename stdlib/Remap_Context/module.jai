// Turns a context built by another program into this program's `#Context`: `remap_context` returns
// it, `Remap_Context()` installs it for the rest of the scope. `VERBOSE := false` silences the
// warning for a context it cannot map.
#module_parameters(VERBOSE := true);
Remap_Context :: Impl.Remap_Context;
remap_context :: Impl.remap_context;

#scope_file
Impl :: #import "internal/context/remap"(VERBOSE = VERBOSE);
