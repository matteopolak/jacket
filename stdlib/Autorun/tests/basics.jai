#import "Basic";
A :: #import "Autorun";
C :: #import "Compiler";
F :: #import "File";
S :: #import "String";
P :: #import "POSIX";
U :: #import "File_Utilities";

errors: s64;
information: s64;

capture :: (text: string, data: *void, info: Log_Info) {
    assert(text.count > 0);
    if (info.common_flags & .ERROR) != 0 errors += 1;
    else information += 1;
}

CHILD :: #string JAI
#import "Basic";
F :: #import "File";
main :: () {
    args := get_command_line_arguments();
    defer array_free(args);
    assert(args.count == 3);
    assert(F.write_entire_file(args[1],args[2]));
}
JAI

#run {
    output := sprint("%/../../../target/portable-tests/", S.path_strip_filename(#file));

    defer free(output);
    workspace := C.compiler_create_workspace("autorun contract");
    options := C.get_build_options(workspace);
    options.output_path = output;
    executable_name := sprint("autorun-child-%", P.getpid());

    defer free(executable_name);
    options.output_executable_name = executable_name;
    C.set_build_options(options, workspace);
    C.compiler_begin_intercept(workspace);
    C.add_build_string(CHILD, workspace);
    marker := sprint("%autorun-marker-%", output, P.getpid());

    defer free(marker);
    plugin := A.get_plugin();

    assert(plugin == A.get_plugin());
    base := cast(*C.Metaprogram_Plugin) plugin;
    base.workspace = workspace;
    arguments := string.[marker, "forwarded bytes"];

    assert(base.init(base, arguments));

    while true {
        message := C.compiler_wait_for_message();
        if message.kind == .COMPLETE {
            executable := sprint("%%", output, executable_name);
            assert(U.file_exists(executable));
            free(executable);
        }

        base.message(base, message);

        if message.kind == .COMPLETE {
            assert((cast(*C.Message_Complete) message).error_code == .NONE);
            break;
        }
    }

    C.compiler_end_intercept(workspace);
    base.shutdown(base);
    content, loaded := F.read_entire_file(marker);

    assert(loaded && content == "forwarded bytes");
    free(content);
    assert(F.file_delete(marker));
    A.run_build_result_of_workspace(workspace, arguments);
    content, loaded = F.read_entire_file(marker);
    assert(loaded && content == "forwarded bytes");
    free(content);
    assert(F.file_delete(marker));
    other := C.compiler_create_workspace("autorun non-executable");
    other_options := C.get_build_options(other);
    other_options.output_type = .NO_OUTPUT;
    C.set_build_options(other_options, other);
    scoped := context;
    scoped.logger = capture;
    push_context scoped A.run_build_result_of_workspace(other);
    assert(information == 1 && errors == 0);
}

main :: () {
    scoped := context;
    scoped.logger = capture;
    push_context scoped {
        A.run_native("/does-not-exist-portable-autorun", .[]);
        A.run_native("/bin/sh", string.["-c", "exit 7"]);
    }

    assert(errors == 2);
    print("ok\n");
}
