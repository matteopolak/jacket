Canonical_Builder :: #import "internal/basic-builder-types"(ENABLE_ASSERT = true);

String_Builder :: Canonical_Builder.String_Builder;

#load "print-types.jai";

#add_context print_style: Print_Style;

#add_context inside_memory_debugger: bool;

is_active :: inline () -> bool #no_debug {
    return context.inside_memory_debugger;
}
