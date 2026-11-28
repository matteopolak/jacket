// Canonical runner facade; integration owns its single module load.
#load "../internal/developer-tools/generator-run.jai";

// Parses the headers in `options.source_files` into `generator` and prepares its declarations for output.
//
// Loads the libraries, runs libclang over the headers, converts the top-level declarations, names enum
// values, evaluates macros and applies the visitors from the options. Returns false, with an error logged,
// when a library cannot be loaded, the headers cannot be parsed or the declaration graph is invalid.
// `generator` must be initialized, and the options should be normalized (see [normalize_options]);
// [generate_bindings] does all of this.
run_generator :: (generator: *Generator, options: Generate_Bindings_Options) -> bool {
    return gr_run(generator, options);
}

// The libclang visitor that converts each top-level cursor of the translation unit into a declaration.
//
// `client_data` must point to the `#Context` to run in. A null pointer stops the traversal.
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
