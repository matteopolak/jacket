using #import "Basic";
C :: #import "Compiler";
M :: #import "Metaprogram_Plugins";

#run,stallable {
    requests: [1] M.Plugin_To_Create;
    requests[0].name = "Deliberately_Unavailable_Plugin_Module";
    initialized: [..] *C.Metaprogram_Plugin;
    M.init_plugins(requests, *initialized, -1);
}

main :: () {}
