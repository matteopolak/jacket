// Example_Plugin: drive every callback by hand (jaic cannot load plugins) and check the
// option parser's cursor handling.
#import "Basic";
#import "Compiler";
Example :: #import "Example_Plugin";

main :: () {
    plugin := Example.get_plugin();
    assert(plugin.before_intercept != null && plugin.add_source != null && plugin.message != null);
    assert(plugin.finish != null && plugin.shutdown != null && plugin.handle_one_option != null && plugin.log_help != null);

    flags: Intercept_Flags;
    plugin.before_intercept(plugin, *flags);
    plugin.add_source(plugin);

    checked: Message_Typechecked;
    checked.kind = .TYPECHECKED;
    plugin.message(plugin, *checked);

    phase: Message_Phase;
    phase.kind = .PHASE;
    phase.phase = .PRE_WRITE_EXECUTABLE;
    phase.executable_name = "demo";
    plugin.message(plugin, *phase);

    complete: Message_Complete;
    complete.kind = .COMPLETE;
    plugin.message(plugin, *complete);

    options := string.["-x", "-option", "value", "-option"];
    assert(plugin.handle_one_option(plugin, options, 0) == 0);  // not ours
    assert(plugin.handle_one_option(plugin, options, 1) == 3);  // flag and its argument
    assert(plugin.handle_one_option(plugin, options, 3) == 4);  // missing argument
    assert(plugin.handle_one_option(plugin, options, 9) == 9);  // out of range

    plugin.log_help(plugin);
    plugin.finish(plugin);
    plugin.shutdown(plugin);
    print("ok\n");
}
