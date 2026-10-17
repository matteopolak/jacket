#module_parameters(VALUE := 41);

get_plugin :: () -> *C.Metaprogram_Plugin {
    instance.init = initialize;
    return *instance;
}

#scope_file
C :: #import "Compiler";
using #import "Basic";

instance: C.Metaprogram_Plugin;

initialize :: (plugin: *C.Metaprogram_Plugin, options: [] string) -> bool {
    assert(plugin.workspace >= 0 && VALUE == 41);
    assert(options.count == 1 && options[0] == "actual option");
    print("initialized %\n", VALUE);
    return true;
}
