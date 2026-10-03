// Imports are supplied once; the typed indirect call waits for its declared body.
init_plugins :: (to_create: [] Plugin_To_Create, results: *[..] *C.Metaprogram_Plugin, w: Workspace) -> (success: bool) {
    if !initialization_requested {
        body := create_initializer_body(to_create);

        defer free(body);
        literal := quote_source_string(body);

        defer free(literal);
        declaration := Strings.join("#scope_module\nGenerated_Initialization :: ", literal, ";\n");

        defer free(declaration);
        initialization_requested = true;
        C.add_build_string(declaration, -1, INITIALIZATION_SCOPE);
    }

    invoke: Initialization = initialize_requested;

    return invoke(to_create, results, w);
}

#scope_module
using #import "Basic";
C :: #import "Compiler";
Strings :: #import "String";

Initialization :: #type (requests: [] Plugin_To_Create, results: *[..] *C.Metaprogram_Plugin, w: Workspace) -> bool;
#placeholder Generated_Initialization;
INITIALIZATION_SCOPE :: #code Generated_Initialization;
initialization_requested: bool;

initialize_requested :: (requests: [] Plugin_To_Create, results: *[..] *C.Metaprogram_Plugin, w: Workspace) -> bool {
    #insert Generated_Initialization;
}

create_initializer_body :: (requests: [] Plugin_To_Create) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);

    for request, index: requests {
        module_name := request.name;
        suffix: string;
        split := Strings.find_index_from_left(module_name, cast(u8) 40);

        if split >= 0 {
            suffix = Strings.slice(module_name, split, module_name.count);
            module_name = Strings.slice(module_name, 0, split);
        }

        quoted := quote_source_string(module_name);

        defer free(quoted);
        print_to_builder(*builder, "Requested_% :: #import %%;\n", index, quoted, suffix);
    }

    print_to_builder(*builder, "if requests.count != % return false;\n", requests.count);

    for request, index: requests {
        quoted := quote_source_string(request.name);
        defer free(quoted);
        print_to_builder(*builder, "if requests[%].name != % return false;\n", index, quoted);
    }

    append(*builder, "success := true;\n");

    for index: 0..requests.count - 1 {
        print_to_builder(*builder, "{ plugin := Requested_%.get_plugin();\n", index);
        append(*builder, "if !plugin { success = false; } else {\n");
        append(*builder, "plugin.workspace = w; array_add(results, plugin);\n");
        print_to_builder(*builder, "if plugin.init { if !plugin.init(plugin, requests[%].options) success = false; }\n", index);
        print_to_builder(*builder, "else if requests[%].options.count != 0 success = false; }}\n", index);
    }

    append(*builder, "return success;\n");

    return builder_to_string(*builder);
}

quote_source_string :: (value: string) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, "\"");

    for byte: value {
        if byte == 34 || byte == 92 append(*builder, "\\");
        if byte == 10 append(*builder, "\\n");
        else if byte == 13 append(*builder, "\\r");
        else append(*builder, byte);
    }

    append(*builder, "\"");

    return builder_to_string(*builder);
}
