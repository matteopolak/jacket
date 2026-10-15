#import "Basic";
#import "Compiler";
C :: #import "Check";

#run {
    workspace := compiler_create_workspace("Original Check factory callback workflow");
    options := get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, workspace);
    plugin := C.get_plugin();
    plugin.workspace = workspace;
    compiler_begin_intercept(workspace);
    add_build_string("promoted :: (a: u8, b: u8) -> u32 { return cast(u32) a + b; }\nmain :: () {}\n", workspace);
    messages: s64;

    while true {
        message := compiler_wait_for_message();
        if !message break;

        if message.workspace == workspace {
            plugin.message(plugin, message);
            if message.kind == .TYPECHECKED messages += 1;
            if message.kind == .COMPLETE break;
        }
    }

    compiler_end_intercept(workspace);
    assert(messages > 0);
}

main :: () {
    print("ok\n");
}
