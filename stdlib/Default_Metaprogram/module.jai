plugins: [..] *C.Metaprogram_Plugin;
verbose := false;

HELP_STRING :: #string DONE
Default build: source.jai [options] [+Plugin plugin-options] [- target-arguments]
  -add CODE / -run CODE          add declarations or compile-time work
  -exe NAME / -output_path DIR   select the executable destination
  -import_dir DIR                prepend a module search directory
  -o / -od / -very_debug         select optimization and debugging
  -llvm / -x64                   select a compiler backend
  -no_inline -no_dce -no_split    adjust optimization passes
  -output_ir                     retain LLVM IR before and after optimization
  -quiet -no_color -msvc_format -natvis -verbose
  -debug_for -debugger -no_backtrace_on_crash
  -context_size BYTES            set the maximum context allocation
  -no_check -check_bindings -no_check_bindings
  -help / -? / -version          display information
DONE

is_absolute_path :: (path: string) -> bool {
    if path.count == 0 return false;
    return path[0] == 47 || path[0] == 92 || (path.count >= 3 && path[1] == 58);
}

build :: () {
    execute_default_build();
}

message_loop :: (w: Workspace) {
    failed := false;

    while true {
        message := C.compiler_wait_for_message();
        if !message {
            C.compiler_end_intercept(w);
            shutdown_plugins();
            C.compiler_report("Target workspace ended without a completion message.");
            return;
        }

        for p: plugins if p.message p.message(p, message);

        if message.workspace == w && message.kind == .COMPLETE {
            complete := cast(*C.Message_Complete) message;
            failed = complete.error_code != .NONE;
            break;
        }
    }

    C.compiler_end_intercept(w);
    for p: plugins if p.finish p.finish(p);

    shutdown_plugins();
    if failed C.compiler_set_workspace_status(.FAILED, w);
}

#scope_module
using #import "Basic";
C :: #import "Compiler";
S :: #import "String";
F :: #import "File";
#load "../internal/build/default-options.jai";
#load "../internal/build/default-workspace.jai";
#load "../internal/build/default-plugins.jai";
