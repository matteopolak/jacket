#module_parameters(VALUE := 41);

get_plugin :: () -> *C.Metaprogram_Plugin {
    instance.init = initialize;
    return *instance;
}

#scope_file
C :: #import "Compiler";
using #import "Basic";

instance: C.Metaprogram_Plugin;
initializations: int;

initialize :: (plugin: *C.Metaprogram_Plugin, options: [] string) -> bool {
    assert(plugin.workspace >= 0);
    assert(initializations == 0);
    expected := sprint("registration option %", VALUE);

    defer free(expected);
    assert(options.count == 1 && options[0] == expected);
    initializations += 1;

    return true;
}
