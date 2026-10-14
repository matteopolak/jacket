#import "Basic";
C :: #import "Compiler";
P :: #import "Program_Print";
F :: #import "File";

#run {
    w := C.compiler_create_workspace("print contract");
    options := C.get_build_options(w); options.output_type = .NO_OUTPUT;
    C.set_build_options(options, w);
    C.add_build_file("inspected.jai", w);
    plugin := P.get_plugin(); plugin.workspace = w;
    C.compiler_begin_intercept(w);
    while true {
        message := C.compiler_wait_for_message();
        assert(message != null);
        plugin.message(plugin, message);
        if message.kind == .COMPLETE {
            assert((cast(*C.Message_Complete) message).error_code == .NONE);
            break;
        }
    }
    plugin.finish(plugin);
    text, success := F.read_entire_file("pp_inspected.jai.txt");
    assert(success);
    expected :: "bloom:: 47;";
    found := false;
    for i: 0..text.count-expected.count {
        part := string.{expected.count, text.data+i};
        if part == expected found = true;
    }
    assert(found);
    assert(F.file_delete("pp_inspected.jai.txt"));
    C.compiler_end_intercept(w);
}

main :: () { print("ok\n"); }
