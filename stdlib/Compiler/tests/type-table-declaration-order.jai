#import "Basic";
#import "Compiler";

// Opening a workspace builds descriptors for the Compiler module's records. They enter the type
// table in declaration order, whichever thread parsed a spelling first.

drive :: () {
    workspace := compiler_create_workspace("order");
    options := get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, workspace);

    compiler_begin_intercept(workspace);
    add_build_string("main :: () {}", workspace);

    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE break;
    }

    compiler_end_intercept(workspace);
}

#run drive();

index_of :: (name: string) -> int {
    for get_type_table() {
        if it.type == .STRUCT && (cast(*Type_Info_Struct) it).name == name return it_index;
    }

    return -1;
}

main :: () {
    // Declared in this order in the Compiler module's records.
    names :: string.["Message", "Message_File", "Message_Import", "Message_Phase", "Message_Typechecked", "Code_Ident", "Code_Literal"];

    previous := -1;

    for names {
        at := index_of(it);
        assert(at > previous, "% is out of declaration order", it);
        previous = at;
    }

    print("ok\n");
}
