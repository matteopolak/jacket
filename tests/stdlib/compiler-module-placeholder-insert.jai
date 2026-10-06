// A metaprogram fills a module's `#placeholder`s with `add_build_string(s, w, import)`
// (Vk-Engine's entity code). An `#insert` in the module that builds a type from them runs
// once they exist, an unused struct of the module that names that type checks, and the
// program reaches the generated names through its import.
#import "Basic";
#import "Compiler";

TARGET :: #string END
#import "Basic";
#import "Generated_Kinds";
main :: () {
    for Kind_Labels if it.count print("%\n", it);
    k := Kind.STONE;
    print("% %\n", k, label_of(k));
}
END

GENERATED :: #string END
Kind_Names :: string.["", "STONE", "TREE"];
Kind_Labels :: string.["", "Stone", "Tree"];
END

#run {
    options := get_build_options();
    child: Build_Options;
    copy_commonly_propagated_fields(options, *child);
    paths: [..] string;
    array_add(*paths, "modules");
    for options.import_path array_add(*paths, it);
    child.import_path = paths;
    child.output_type = .NO_OUTPUT;
    w := compiler_create_workspace("generated kinds");
    set_build_options(child, w);
    compiler_begin_intercept(w);
    add_build_string(TARGET, w);
    target: *Message_Import;
    generated := false;
    failed := false;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .IMPORT {
            import := cast(*Message_Import) message;
            if import.module_name == "Generated_Kinds" target = import;
        }
        if message.kind == .PHASE && !generated && target {
            if (cast(*Message_Phase) message).phase == .TYPECHECKED_ALL_WE_CAN {
                generated = true;
                add_build_string(GENERATED, w, target);
            }
        }
        if message.kind == .COMPLETE {
            failed = (cast(*Message_Complete) message).error_code != .NONE;
            break;
        }
    }
    compiler_end_intercept(w);
    assert(generated, "the module was never imported");
    assert(!failed, "the generated module did not compile");
    set_build_options_dc(.{ do_output = false });
};
