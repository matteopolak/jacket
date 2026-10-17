// Only public canonical Basic is reexported. No library implementation is copied.
#scope_export
using Configured_Basic :: #import "Basic"()(ENABLE_ASSERT = false);

#assert !#exists(String_Builder.Buffer.ensured_count);
