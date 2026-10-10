// rules: compiler.29
// At compile time `get_type_table()` lists every type declared so far, used or not, with
// descriptors that can be walked; in the running program it keeps listing what the program reaches.
#import "Basic";
#import "Compiler";

Used :: struct {
    a: int;
    b: string;
}

Unused :: struct {
    c: float;
    d: [3] u8;
}

Unused_Enum :: enum {
    FIRST;
    SECOND;
}

#run {
    table := get_type_table();
    assert(table.count > 0);
    found_unused, found_enum := false, false;
    for table {
        if it.type == .STRUCT {
            info := cast(*Type_Info_Struct) it;
            if info.name == "Unused" {
                found_unused = true;
                assert(info.members.count == 2 && info.members[0].name == "c" && info.members[1].name == "d");
            }
        }
        if it.type == .ENUM {
            info := cast(*Type_Info_Enum) it;
            if info.name == "Unused_Enum" {
                found_enum = true;
                assert(info.names.count == 2 && info.names[1] == "SECOND");
            }
        }
    }
    assert(found_unused && found_enum);

    // Another workspace's table, between its compiler messages.
    w := compiler_create_workspace("other");
    other_options := get_build_options(w);
    other_options.output_type = .NO_OUTPUT;
    set_build_options(other_options, w);
    compiler_begin_intercept(w);
    add_build_string("Other_Struct :: struct { x: int; } main :: () {}", w);
    seen := false;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE {
            for get_type_table(w) {
                if it.type == .STRUCT && (cast(*Type_Info_Struct) it).name == "Other_Struct" seen = true;
            }
            break;
        }
    }
    compiler_end_intercept(w);
    assert(seen);
}

main :: () {
    x: Used;
    print("%\n", x);
    // The running program's table has what the program reaches, not every declaration.
    for get_type_table() {
        if it.type == .STRUCT {
            assert((cast(*Type_Info_Struct) it).name != "Unused_Never_Reached");
        }
    }
    print("ok\n");
}
