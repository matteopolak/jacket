#import "Basic";
#import "Compiler";
#import "File";
#import "Hash_Table";
R :: #import "Rewrite";

#run {
    path := tprint("%../../target/developer-tools-rewrite-program.jai", #filepath);
    workspace := compiler_create_workspace("Rewrite independent fixture");
    options := get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, workspace);
    plugin: R.My_Plugin;

    assert(R.init(*plugin, .[]));
    defer plugin.shutdown(*plugin);
    plugin.workspace = workspace;
    compiler_begin_intercept(workspace);
    add_build_file(path, workspace);

    while true {
        message := compiler_wait_for_message();
        if !message break;

        R.gather(*plugin, message);

        if message.kind == .COMPLETE {
            assert((cast(*Message_Complete) message).error_code == .NONE);
            break;
        }
    }

    compiler_end_intercept(workspace);
    count: s64;
    for entry: plugin.files_to_rewrite.entries if entry.hash >= FIRST_VALID_HASH count += entry.value.single_rewrites.count;

    assert(count == 1);
    plugin.write_mode = true;
    R.finish(*plugin);
    data, ok := read_entire_file(path);

    defer free(data);
    assert(ok);
    assert(contains(data, "result := (pointer).*;"));
}

main :: () {}

#scope_file
#import "String";
