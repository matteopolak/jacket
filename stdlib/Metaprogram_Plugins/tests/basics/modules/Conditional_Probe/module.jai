get_plugin :: () -> *C.Metaprogram_Plugin {
    instance.init = initialize;
    return *instance;
}

#scope_file
C :: #import "Compiler";
using #import "Basic";

instance: C.Metaprogram_Plugin;

initialize :: (plugin: *C.Metaprogram_Plugin, options: [] string) -> bool {
    assert(plugin.workspace >= 0);
    assert(options.count == 1);
    print("conditional %\n", options[0]);
    return options[0] == "accept";
}
