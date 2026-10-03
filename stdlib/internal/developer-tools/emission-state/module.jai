// Single private traversal protocol across parameterized generator instances.
// No canonical model records or public compatibility state live here.
Phase :: enum {
    DECLARATION_BODY;
    JAI_TYPE;
    CPP_TYPE;
    CPP_EXPRESSION;
    CPP_MATCH;
    CPP_ARRAY_QUALIFIERS;
    FUNCTION_TYPE;
    TARGET_EXPRESSION;
    EXPRESSION;
    LITERAL;
    STRUCT_LITERAL;
    TEMPLATE_ARGUMENTS;
    ARRAY_PARAMETER;
}

enter :: (phase: Phase, node: *void) -> bool {
    if !node return false;

    path := context._bindings_emission_traversal;
    if !path {
        path = New(Path);
        path.allocator = context.allocator;
        context._bindings_emission_traversal = path;
    }

    for item: path.active if item.phase == phase && item.node == node {
        return false;
    }

    array_add(*path.active, Entry.{ phase, node });

    return true;
}

leave :: () {
    path := context._bindings_emission_traversal;

    assert(path != null && path.active.count > 0);
    path.active.count -= 1;
    if path.active.count return;

    push_allocator(path.allocator);
    array_free(path.active);
    context._bindings_emission_traversal = null;
    free(path);
}

#scope_file
#import "Basic";

Entry :: struct {
    phase: Phase;
    node: *void;
}

Path :: struct {
    allocator: Allocator;
    active: [..] Entry;
}

#add_context _bindings_emission_traversal: *Path;
