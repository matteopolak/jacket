// Known typed registration, generated imports, no undeclared continuation.
using #import "Basic";
C :: #import "Compiler";
S :: #import "String";

#load "registration.jai";

finish_setup :: () {
    assert(registration_ready() && registration_count == 2);
    defer end_registration();
    workspace := C.compiler_create_workspace("Registered factory target");

    defer C.compiler_destroy_workspace(workspace);
    results: [..] *C.Metaprogram_Plugin;

    defer array_free(results);

    for factory_slots {
        plugin := it();

        assert(plugin != null && plugin.init != null);
        plugin.workspace = workspace;
        option := sprint("registration option %", 41 + it_index);
        options: [1] string = .[option];

        assert(plugin.init(plugin, options));
        free(option);
        assert(plugin.workspace == workspace);
        array_add(*results, plugin);
    }

    assert(results.count == 2 && results[0] != results[1]);
    print("registered and initialized\n");
}

request_source :: () {
    name := S.join("Registration_", "Probe");

    defer free(name);
    epoch := begin_registration(2);
    source := sprint("Imported_A :: #import \"%\"(VALUE=41); Imported_B :: #import \"%\"(VALUE=42); #run { assert(register_factory(%, 1, Imported_B.get_plugin)); assert(register_factory(%, 0, Imported_A.get_plugin)); finish_setup(); }", name, name, epoch, epoch);

    defer free(source);
    C.add_build_string(source, -1, #code register_factory);
}

#run,stallable request_source();

main :: () {}
