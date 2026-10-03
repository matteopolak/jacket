#scope_module

initialize_plugins :: (w: Workspace, request: *Build_Request) -> bool {
    M :: #import "Metaprogram_Plugins";
    requested: [..] M.Plugin_To_Create;

    defer array_free(requested);

    for plugin: request.plugin_requests {
        entry: M.Plugin_To_Create;
        entry.name = plugin.name;
        entry.options = plugin.arguments;
        array_add(*requested, entry);
    }

    if request.check {
        entry: M.Plugin_To_Create;
        if request.check_bindings entry.name = "Check(CHECK_BINDINGS=true)";
        else entry.name = "Check(CHECK_BINDINGS=false)";
        array_add(*requested, entry);
    }

    return M.init_plugins(requested, *plugins, w);
}
