// Reviewed aggregate init status, no-init options and actual factory entity identity.
using #import "Basic";
C :: #import "Compiler";
M :: #import "Metaprogram_Plugins";

#run,stallable {
    workspace := C.compiler_create_workspace("Aggregate plugin test");

    assert(workspace >= 0);
    defer C.compiler_destroy_workspace(workspace);
    requests: [3] M.Plugin_To_Create;
    requests[0].name = "Dynamic_Probe(VALUE=41)";
    requests[1].name = "Conditional_Probe";
    requests[2].name = "Passive_Probe";
    array_add(*requests[0].options, "actual option");
    array_add(*requests[1].options, "reject");
    array_add(*requests[2].options, "not accepted without initializer");
    defer for request: requests array_free(request.options);
    initialized: [..] *C.Metaprogram_Plugin;

    defer array_free(initialized);
    first := M.init_plugins(requests, *initialized, workspace);

    assert(!first && initialized.count == 3);
    for plugin: initialized assert(plugin != null && plugin.workspace == workspace);

    assert(initialized[2].init == null);
    requests[1].options[0] = "accept";
    array_reset(*requests[2].options);
    second := M.init_plugins(requests, *initialized, workspace);

    assert(second && initialized.count == 6);
    for i: 0..2 assert(initialized[i] == initialized[i + 3]);

    print("ok\n");
}

main :: () {}
