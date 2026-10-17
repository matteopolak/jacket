// Real missing-tool failure of the private callback; not Windows public workflow proof.
#load "../../internal/build/linux-link.jai";

run_missing_linker :: (output_directory: string) {
    assert(output_directory.count > 0);
    w := C.compiler_create_workspace("linux custom linker failure");

    assert(w >= 0);
    options := C.get_build_options(w);
    options.os_target = .LINUX;
    options.cpu_target = .X64;
    options.use_custom_link_command = true;
    options.output_path = output_directory;
    options.output_executable_name = "missing-linker-target";
    // Current compiler has no X64 backend. The message callback is tested separately.
    options.backend = .LLVM;
    C.set_build_options(options, w);
    C.set_build_options_dc(.{ do_output = false, write_added_strings = false });
    C.compiler_begin_intercept(w);
    p := get_plugin();

    assert(p != null);
    p.workspace = w;
    defer shutdown(p);
    C.add_build_string("main :: () {}", w);
    seen_link := false;

    while true {
        event := C.compiler_wait_for_message();

        assert(event != null);
        if event.workspace != w continue;

        if event.kind == .PHASE {
            phase := cast(*C.Message_Phase) event;
            if phase.phase == .READY_FOR_CUSTOM_LINK_COMMAND seen_link = true;
            message(p, event);
        } else if event.kind == .COMPLETE {
            complete := cast(*C.Message_Complete) event;
            assert(seen_link);
            assert(complete.error_code == .COMPILATION_FAILED);
            break;
        }
    }

    C.compiler_end_intercept(w);
    write_string("missing linker failure observed\n");
}
