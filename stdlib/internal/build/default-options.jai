#scope_module

Plugin_Request :: struct {
    name: string;
    arguments: [..] string;
}

Build_Request :: struct {
    options: C.Build_Options;
    sources: [..] string;
    snippets: [..] string;
    import_paths: [..] string;
    plugin_requests: [..] Plugin_Request;
    target_arguments: [..] string;
    help: bool;
    version: bool;
    check := true;
    check_bindings := true;
    explicit_exe: bool;
    explicit_path: bool;
}

option_error :: (message: string) {
    print("Default build: %\n", message);
    exit(1);
}

parse_build_arguments :: (arguments: [] string, options: C.Build_Options, apply_actions := true) -> Build_Request {
    request: Build_Request;
    request.options = options;
    index: s64;
    while index < arguments.count {
        word := arguments[index];

        index += 1;

        if word == "-" {
            while index < arguments.count {
                array_add(*request.target_arguments, arguments[index]);
                index += 1;
            }

            break;
        }

        if word.count > 0 && word[0] == 43 {
            if word.count == 1 option_error("A plugin name must follow '+'.");

            plugin: Plugin_Request;
            plugin.name = S.slice(word, 1, word.count);

            while index < arguments.count {
                next := arguments[index];
                if next == "-" || (next.count > 0 && next[0] == 43) break;
                array_add(*plugin.arguments, next);
                index += 1;
            }

            array_add(*request.plugin_requests, plugin);
            continue;
        }

        value: string;
        takes_value := word == "-add" || word == "-run" || word == "-exe" || word == "-output_path" || word == "-import_dir" || word == "-context_size";

        if takes_value {
            if index >= arguments.count option_error(sprint("Missing value after %.", word));
            value = arguments[index];
            index += 1;
        }

        if word == {
            case "-add"; array_add(*request.snippets, S.join(value, ";\n"));
            case "-run"; array_add(*request.snippets, S.join("#run ", value, ";\n"));
            case "-exe";
                request.options.output_executable_name = value;
                request.explicit_exe = true;
            case "-output_path";
                request.options.output_path = value;
                request.explicit_path = true;
            case "-import_dir"; array_add(*request.import_paths, value);
            case "-help"; #through;
            case "-?"; request.help = true;
            case "-version"; request.version = true;
            case "-verbose"; if apply_actions verbose = true;
            case "-no_check"; request.check = false;
            case "-check_bindings"; request.check_bindings = true;
            case "-no_check_bindings"; request.check_bindings = false;
            case "-release";
                if apply_actions print("-release is deprecated; use -o.\n");

                #through;
            case "-o"; #through;
            case "-optimized";
                C.set_optimization(*request.options, .VERY_OPTIMIZED);
                request.options.stack_trace = false;
            case "-release_debug";
                if apply_actions print("-release_debug is deprecated; use -od.\n");

                #through;
            case "-od"; #through;
            case "-optimized_debug";
                C.set_optimization(*request.options, .OPTIMIZED);
                request.options.stack_trace = true;
            case "-very_debug"; C.set_optimization(*request.options, .VERY_DEBUG);
            case "-llvm"; request.options.backend = .LLVM;
            case "-x64"; request.options.backend = .X64;
            case "-no_inline"; request.options.llvm_options.disable_inlining = true;
            case "-no_dce"; request.options.dead_code_elimination = .NONE;
            case "-no_split"; request.options.llvm_options.enable_split_modules = false;
            case "-output_ir";
                request.options.llvm_options.output_llvm_ir = true;
                request.options.llvm_options.output_llvm_ir_before_optimizations = true;
            case "-quiet"; request.options.text_output_flags = 0;
            case "-no_color"; request.options.use_ansi_color = false;
            case "-msvc_format"; request.options.use_visual_studio_message_format = true;
            case "-natvis"; request.options.use_natvis_compatible_types = true;
            case "-debug_for"; request.options.debug_for_expansions = true;
            case "-debugger"; request.options.interactive_bytecode_debugger = true;
            case "-no_backtrace_on_crash"; request.options.backtrace_on_crash = .OFF;
            case "-context_size";
                number, success, tail := to_integer(value);
                if success && tail.count == 0 && number >= size_of(Context_Base) && number <= 0x40000 request.options.context_size_max = cast(s32) number;
                else if apply_actions print("Ignoring invalid context size '%'.\n", value);
            case;
                if word.count > 0 && word[0] == 45 option_error(sprint("Unknown option '%'.", word));

                array_add(*request.sources, word);
        }
    }

    return request;
}

free_build_request :: (request: *Build_Request) {
    for code: request.snippets free(code);
    for plugin: request.plugin_requests array_free(plugin.arguments);

    array_free(request.sources);
    array_free(request.snippets);
    array_free(request.import_paths);
    array_free(request.plugin_requests);
    array_free(request.target_arguments);
}
