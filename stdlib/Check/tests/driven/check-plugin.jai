// Reviewed Check factory contract used by Default_Metaprogram.
#import "Basic";
#import "Compiler";
C :: #import "Check";
No_Bindings :: #import "Check"(CHECK_BINDINGS = false);

unused_init :: (plugin: *Metaprogram_Plugin, options: [] string) -> bool {
    return false;
}

main :: () {
    plugin := C.get_plugin();

    assert(plugin != null && plugin.message != null);
    assert(type_of(plugin) == *Metaprogram_Plugin);
    plugin.init = unused_init;
    again := C.get_plugin();

    assert(again == plugin && again.init == null);
    assert(again.before_intercept == null && again.add_source == null);
    assert(again.finish == null && again.shutdown == null && again.log_help == null && again.communication_proc == null);
    other := No_Bindings.get_plugin();

    assert(other != null && other != plugin && other.message != null);
    message: Message_Typechecked;
    message.kind = .COMPLETE;
    message.workspace = 0;
    plugin.workspace = 0;
    plugin.message(plugin, cast(*Message) *message);
    plugin.message(plugin, null);
    print("ok\n");
}
