// The Icon metaprogram plugin takes "-icon <file>" and ignores messages until an icon is set.
#import "Basic";
Icon :: #import "Icon";

#run {
    plugin := Icon.get_plugin();
    assert(plugin.handle_one_option != null && plugin.message != null && plugin.shutdown != null);
    options := string.["-other", "-icon", "app.ico", "-icon"];
    assert(plugin.handle_one_option(plugin, options, 0) == 0, "not ours");
    assert(plugin.handle_one_option(plugin, options, 1) == 3, "flag and its argument");
    assert(plugin.handle_one_option(plugin, options, 3) == 4, "missing argument still advances");
    assert(plugin.handle_one_option(plugin, options, 9) == 9, "out of range");
    plugin.message(plugin, null);
    plugin.shutdown(plugin);
}

main :: () {}
