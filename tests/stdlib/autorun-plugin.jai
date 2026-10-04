// Autorun is a metaprogram plugin; check that it typechecks, wires callbacks and
// that a failed COMPLETE message disables running.
#import "Basic";
#import "Compiler";
Autorun :: #import "Autorun";

main :: () {
    plugin := Autorun.get_plugin();
    assert(plugin.message != null);
    assert(plugin.shutdown != null);

    complete: Message_Complete;
    complete.kind = .COMPLETE;
    complete.error_code = .COMPILATION_FAILED;
    plugin.message(plugin, *complete);

    // With errors seen, shutdown must not launch anything and must free the plugin.
    plugin.shutdown(plugin);
    print("ok\n");
}
