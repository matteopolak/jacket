// Canonical runner facade; integration owns its single module load.
#load "../internal/developer-tools/generator-run.jai";

run_generator :: (generator: *Generator, options: Generate_Bindings_Options) -> bool {
    return gr_run(generator, options);
}

handle_toplevel_cursors :: (cur: CXCursor, parent: CXCursor, client_data: CXClientData) -> CXChildVisitResult #c_call {
    // There is no Jai Context on entry to a native callback. Without a supplied
    // Context, even a logger call would be invalid.
    if !client_data return .Break;

    push_context, defer_pop;
    context = (cast(*#Context) client_data).*;

    return gr_toplevel(cur, parent);
}

#scope_file
#import "Basic";
