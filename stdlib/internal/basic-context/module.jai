Canonical_Builder :: #import "internal/basic-builder-types"(ENABLE_ASSERT = true);

String_Builder :: Canonical_Builder.String_Builder;

#load "print-types.jai";

#add_context print_style: Print_Style;

#add_context inside_memory_debugger: bool;

is_active :: inline () -> bool #no_debug {
    return context.inside_memory_debugger;
}

// Set by the memory debugger when it starts tracking; `allocator_released_everything` calls it. It lives here, in a
// module no import parameter changes, so an allocator module can reach the debugger whatever `Basic` it imports.
release_hook: #type (owner: Allocator);

// Tells the memory debugger, if one is tracking, that `owner` gave back every range it handed out.
allocator_released_everything :: (owner: Allocator) {
    if release_hook && !is_active() release_hook(owner);
}
