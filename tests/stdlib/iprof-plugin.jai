// Iprof imported as a metaprogram plugin (the default IMPORT_MODE): typechecks the compile-time half and
// exercises the command-line option handling. Instrumentation itself needs a live compiler.
#import "Basic";
#import "Compiler";
Iprof :: #import "Iprof";

main :: () {
    plugin := Iprof.get_plugin();
    assert(plugin.message != null);
    assert(plugin.add_source != null);
    assert(plugin.handle_one_option != null);
    assert(plugin.log_help != null);

    options: [] string = string.["-csv", "out.csv", "-min_size", "12", "-modules"];
    cursor := 0;
    while cursor < options.count {
        next := plugin.handle_one_option(plugin, options, cursor);
        assert(next > cursor);
        cursor = next;
    }

    flags: Intercept_Flags;
    plugin.before_intercept(plugin, *flags);
    plugin.shutdown(plugin);
    print("ok\n");
}
