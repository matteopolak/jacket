// Reviewed 2026-10-15 parser state/borrowing/terminator/parameter contracts.
using #import "Basic";
M :: #import "Metaprogram_Plugins";
C :: #import "Compiler";

#run,stallable {
    empty: [] M.Plugin_To_Create;
    initialized: [..] *C.Metaprogram_Plugin;
    success := M.init_plugins(empty, *initialized, -1);

    assert(success && initialized.count == 0);
    array_free(initialized);
}

main :: () {
    assert(type_info(M.Plugin_To_Create).name == "Plugin_To_Create");
    plain: [3] string = .["source.jai", "-o", "other.jai"];
    for index: 0..plain.count {
        ok, plugins, remaining := M.parse_plugin_arguments(plain, index);
        assert(ok && plugins.count == 0);
        assert(remaining.count == plain.count && remaining.data == plain.data);
    }

    stopped: [3] string = .["-", "+ignored", "value"];
    ok, plugins, remaining := M.parse_plugin_arguments(stopped);

    assert(ok && plugins.count == 0 && remaining.data == stopped.data && remaining.count == stopped.count);
    args: [10] string = .["source.jai", "-o", "+First", "-arg", "", "+Second(VALUE=7)", "yes", "-", "+literal", "-target"];

    ok, plugins, remaining = M.parse_plugin_arguments(args);

    defer {
        for plugin: plugins array_free(plugin.options);
        array_free(plugins);
        free(remaining.data);
    }

    assert(ok && plugins.count == 2 && remaining.count == 4);
    assert(remaining[0] == "source.jai" && remaining[1] == "-o");
    assert(remaining[2] == "+literal" && remaining[3] == "-target");
    assert(plugins[0].name == "First" && plugins[0].name.data == args[2].data + 1);
    assert(plugins[0].options.count == 2 && plugins[0].options[0] == "-arg" && plugins[0].options[1] == "");
    assert(plugins[0].options[0].data == args[3].data);
    assert(plugins[1].name == "Second(VALUE=7)" && plugins[1].options.count == 1 && plugins[1].options[0] == "yes");
    bare: [2] string = .["+", "+X"];
    valid, bare_plugins, rest := M.parse_plugin_arguments(bare);

    assert(valid && bare_plugins.count == 2 && bare_plugins[0].name.count == 0 && bare_plugins[1].name == "X");
    assert(rest.count == 0);
    array_free(bare_plugins);
    free(rest.data);
    invalid, no_plugins, original := M.parse_plugin_arguments(plain, -1);

    assert(!invalid && no_plugins.count == 0 && original.data == plain.data);
    print("ok\n");
}
