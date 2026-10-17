#import "Compiler";
P :: #import "Performance_Report";

#run {
    plugin := cast(*Metaprogram_Plugin) P.get_plugin();
    plugin.finish(plugin);
    plugin.shutdown(plugin);
}

main :: () {}
