// Positive current reviewed contract: first requested name is formed during #run.
using #import "Basic";
C :: #import "Compiler";
M :: #import "Metaprogram_Plugins";
S :: #import "String";

exercise :: () {
    workspace := C.compiler_create_workspace("Dynamic plugin regression");

    assert(workspace >= 0);
    defer C.compiler_destroy_workspace(workspace);
    name := S.join("Dynamic_", "Probe(VALUE=41)");

    defer free(name);
    request: M.Plugin_To_Create;
    request.name = name;
    array_add(*request.options, "actual option");
    defer array_free(request.options);
    requests: [1] M.Plugin_To_Create = .[request];
    result: [..] *C.Metaprogram_Plugin;

    defer array_free(result);
    success := M.init_plugins(requests, *result, workspace);

    assert(success && result.count == 1 && result[0] != null);
    assert(result[0].workspace == workspace);
    requests[0].name = "Dynamic_Probe(VALUE=42)";
    mismatch := M.init_plugins(requests, *result, workspace);

    assert(!mismatch && result.count == 1);
    print("ok\n");
}

#run,stallable exercise();

main :: () {}
