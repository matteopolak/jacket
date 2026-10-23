// A build program that turns its output off may reach `init_plugins` on a branch that does not
// run: the initialization it would generate is then never defined, and that is not an error.
#import "Basic";
#import "Compiler";
#import "Metaprogram_Plugins";

#run build();

build :: () {
    set_build_options_dc(.{ do_output = false });

    arguments := get_build_options().compile_time_command_line;

    if arguments.count == 0 return;

    plugins: [..] *Metaprogram_Plugin;
    requests: [..] Plugin_To_Create;

    init_plugins(requests, *plugins, 0);
}
