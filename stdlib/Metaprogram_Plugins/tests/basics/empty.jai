// Empty first-call definition completes the import, without manufacturing entries.
using #import "Basic";
C :: #import "Compiler";
M :: #import "Metaprogram_Plugins";

#run,stallable {
    requests: [] M.Plugin_To_Create;
    existing: C.Metaprogram_Plugin;
    initialized: [..] *C.Metaprogram_Plugin;

    array_add(*initialized, *existing);
    defer array_free(initialized);
    success := M.init_plugins(requests, *initialized, -1);

    assert(success && initialized.count == 1 && initialized[0] == *existing);
    changed: [1] M.Plugin_To_Create;
    changed[0].name = "Not_Imported_After_Empty_Definition";
    mismatch := M.init_plugins(changed, *initialized, -1);

    assert(!mismatch && initialized.count == 1);
    print("ok\n");
}

main :: () {}
