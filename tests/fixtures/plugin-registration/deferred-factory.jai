using #import "Basic";
C :: #import "Compiler";
S :: #import "String";

#placeholder Factory_Body;
body_scope :: #code Factory_Body;
Loader :: #type (workspace: Workspace, options: [] string, results: *[..] *C.Metaprogram_Plugin) -> bool;

load :: (workspace: Workspace, options: [] string, results: *[..] *C.Metaprogram_Plugin) -> bool {
    #insert Factory_Body;
}

exercise :: () {
    callback: Loader = load;
    workspace := C.compiler_create_workspace("Dynamic factory target");

    defer C.compiler_destroy_workspace(workspace);
    name := S.join("Registration_", "Probe");

    defer free(name);
    parameter := 40 + 1;
    body := sprint("P :: #import \"%\"(VALUE=%); plugin := P.get_plugin(); assert(plugin != null && plugin.init != null); plugin.workspace = workspace; array_add(results, plugin); return plugin.init(plugin, options);", name, parameter);

    defer free(body);
    source := sprint("Factory_Body :: #string BODY\n%\nBODY\n", body);

    defer free(source);
    C.add_build_string(source, -1, body_scope);
    options: [1] string = .["registration option 41"];
    results: [..] *C.Metaprogram_Plugin;

    defer array_free(results);
    assert(callback(workspace, options, *results));
    assert(results.count == 1 && results[0] != null && results[0].workspace == workspace);
    print("dynamic factory initialized before return\n");
}

#run,stallable exercise();

main :: () {}
