C :: #import "Compiler";

state: int;
complete: bool;
plugin: C.Metaprogram_Plugin;

get_plugin :: () -> *C.Metaprogram_Plugin {
    plugin.init = initialize;
    plugin.before_intercept = before;
    plugin.add_source = add;
    plugin.message = message;
    plugin.finish = finish;
    plugin.shutdown = shutdown;

    return *plugin;
}

require :: (value: bool) {
    if !value C.compiler_report("", 0, 0, "plugin lifecycle failure");
}

initialize :: (p: *C.Metaprogram_Plugin, options: [] string) -> bool {
    require(p == *plugin);
    require(state == 0);
    state = 1;
    require(options.count == 1);
    if options[0] == "refuse" return false;

    require(options[0] == "accept");

    return true;
}

before :: (p: *C.Metaprogram_Plugin, flags: *C.Intercept_Flags) {
    require(p == *plugin && flags != null);
    require(state == 1);
    state = 2;
}

add :: (p: *C.Metaprogram_Plugin) {
    require(state == 2);
    state = 3;
    C.add_build_string("generated :: 42;", p.workspace);
}

message :: (p: *C.Metaprogram_Plugin, m: *C.Message) {
    require(p == *plugin);
    require(state == 3);
    if m.kind == .COMPLETE complete = true;
}

finish :: (p: *C.Metaprogram_Plugin) {
    require(p == *plugin && state == 3 && complete);
    state = 4;
}

shutdown :: (p: *C.Metaprogram_Plugin) {
    require(p == *plugin && (state == 4 || state == 1));
    state = 5;
}
