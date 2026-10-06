// rules: compiler.1 compiler.2 compiler.3 compiler.4 compiler.5 compiler.6 compiler.7 compiler.8 compiler.9 compiler.11 compiler.13 compiler.14 compiler.18 compiler.19
// Claims of docs/metaprogramming/compiler-module.md, checked from compile-time code.
#import "Basic";
#import "Compiler";
#import "String";

SMALL :: #string END
answer :: () -> int { return 42; }
main :: () { answer(); }
END

check_options :: () -> bool {
    // compiler.1: the program's own options start from the target constants.
    own := get_build_options();
    assert(own.os_target == OS && own.cpu_target == CPU, "compiler.1: os/cpu target defaults");

    // compiler.2: compile_time_command_line holds the metaprogram arguments.
    args := get_toplevel_command_line();
    assert(own.compile_time_command_line.count == args.count, "compiler.2: command line count");
    for args assert(own.compile_time_command_line[it_index] == it, "compiler.2: command line entry");

    // compiler.3: a new workspace starts from the struct defaults.
    w := compiler_create_workspace("options probe");
    fresh := get_build_options(w);
    defaults: Build_Options;
    assert(fresh.output_type == defaults.output_type, "compiler.3: output_type default");
    assert(fresh.os_target == OS && fresh.cpu_target == CPU, "compiler.3: target default");
    assert(fresh.temporary_storage_size == defaults.temporary_storage_size, "compiler.3: temporary storage default");

    // compiler.7: the default import_path is empty.
    assert(fresh.import_path.count == 0, "compiler.7: default import_path is empty");

    // compiler.4: get_build_options returns what set_build_options stored.
    fresh.output_type = .NO_OUTPUT;
    fresh.output_executable_name = "never_written";
    fresh.temporary_storage_size = 12345;
    set_build_options(fresh, w);
    back := get_build_options(w);
    assert(back.output_type == .NO_OUTPUT && back.output_executable_name == "never_written", "compiler.4: stored copy");
    assert(back.temporary_storage_size == 12345, "compiler.4: stored size");

    // compiler.5: a relative import path is anchored at the caller's directory.
    paths: [..] string;
    array_add(*paths, "mods_dir");
    back.import_path = paths;
    set_build_options(back, w);
    anchored := get_build_options(w).import_path;
    assert(anchored.count == 1, "compiler.5: one import path");
    assert(anchored[0] == tprint("%mods_dir", path_strip_filename(#file)), "compiler.5: anchored at this file's directory");

    // compiler.6: set_build_options_dc updates the stored copy.
    set_build_options_dc(.{ do_output = false, output_path = "elsewhere" }, w);
    after := get_build_options(w);
    assert(after.output_type == .NO_OUTPUT && after.output_path == "elsewhere", "compiler.6: dc stored");

    // compiler.18: get_name answers the name given at creation.
    assert(get_name(w) == "options probe", "compiler.18: workspace name");

    // compiler.14: the version numbers come from the first digit run of the version text.
    info: Version_Info;
    text := compiler_get_version_info(*info);
    assert(text.count > 0, "compiler.14: version text");
    start := 0;
    while start < text.count && (text[start] < #char "0" || text[start] > #char "9") start += 1;
    numbers: [3] s32;
    slot := 0;
    i := start;
    while i < text.count && slot < 3 {
        c := text[i];
        if c >= #char "0" && c <= #char "9" {
            numbers[slot] = numbers[slot] * 10 + cast(s32) (c - #char "0");
        } else if c == #char "." {
            slot += 1;
        } else {
            break;
        }
        i += 1;
    }
    assert(info.major == numbers[0] && info.minor == numbers[1] && info.micro == numbers[2], "compiler.14: parsed numbers");
    return true;
}

check_messages :: () -> bool {
    w := compiler_create_workspace("messages probe");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(SMALL, w);
    while true {
        message := compiler_wait_for_message();
        // compiler.9: every message names the workspace it came from.
        assert(message.workspace == w, "compiler.9: message workspace");
        if message.kind == .COMPLETE break;
    }
    // compiler.8: once everything finished, waiting again keeps returning COMPLETE.
    again := compiler_wait_for_message();
    assert(again.kind == .COMPLETE, "compiler.8: COMPLETE repeats");
    assert((cast(*Message_Complete) again).error_code == .NONE, "compiler.8: with the last error code");
    compiler_end_intercept(w);
    return true;
}

#run check_options();
#run check_messages();

// compiler.11: a warning is reported without failing the compile.
#run compiler_report("rules-compiler-module: this warning is expected", mode = .WARNING);

// compiler.13: marking a workspace OK is accepted and changes nothing.
#run compiler_set_workspace_status(.OK);

main :: () {


    // compiler.19: get_type_table reads the runtime info's type table.
    table := get_type_table();
    assert(table.count > 0, "compiler.19: type table is filled");
    info := get_runtime_info();
    assert(info.type_table.data == table.data && info.type_table.count == table.count, "compiler.19: same table");
    print("ok\n");
}
