// A tool path recorded by a checkout's build that no longer exists (its toolchain was upgraded)
// is reported with the script that records it again, and is not swapped for another tool.
#import "Basic";
#import "String";
#import "File";
BG :: #import "Bindings_Generator";

errors: [..] string;

capture :: (text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR array_add(*errors, copy_string(text));
}

main :: () {
    record := tprint("%recorded-tool-path.txt", #filepath);
    gone := tprint("%no-such-folder/bin/clang", #filepath);

    assert(write_entire_file(record, gone));
    defer file_delete(record);

    push_context {
        context.logger = capture;
        path, recorded := BG.gp_configured_path(record, "tools/developer_checks/build_developer_bindings.py");

        assert(recorded && path == gone, "%", path);
    }

    assert(errors.count == 1, "%", errors);
    assert(contains(errors[0], "no longer exists"), "%", errors[0]);
    assert(contains(errors[0], "PYTHONPATH=tools python3 tools/developer_checks/build_developer_bindings.py"), "%", errors[0]);
    print("recorded tool path passed\n");
}
