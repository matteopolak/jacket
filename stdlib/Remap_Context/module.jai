#module_parameters(VERBOSE := true);
Remap_Context :: Impl.Remap_Context;
remap_context :: Impl.remap_context;

#scope_file
Impl :: #import "internal/context/remap"(VERBOSE = VERBOSE);
