// Actual plugin callback must diagnose an intercepted small-integer addition.
#import "Compiler";
C :: #import "Check";

#run {
    workspace := compiler_create_workspace("Original Check callback rejection");
    options := get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, workspace);
    plugin := C.get_plugin();
    plugin.workspace = workspace;
    compiler_begin_intercept(workspace);
    add_build_string("bad :: (a: u8, b: u8) -> u8 { return a + b; }\nmain :: () { value := bad(1, 2); }\n", workspace);

    while true {
        message := compiler_wait_for_message();
        if !message break;

        if message.workspace == workspace {
            plugin.message(plugin, message);
            if message.kind == .COMPLETE break;
        }
    }

    compiler_end_intercept(workspace);
}
