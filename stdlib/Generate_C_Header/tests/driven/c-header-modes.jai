#import "Basic";
#import "Compiler";
#import "File";
#import "Hash_Table";
#import "String";
H :: #import "Generate_C_Header";

SmallMode :: enum u8 {
    Red;
    Blue;
}

First :: struct {
    Item :: struct {
        x: s32;
    }
}

Second :: struct {
    Item :: struct {
        x: s64;
    }
}

// Test-only descriptors following the reviewed support-record layouts.
Newstring :: struct {
    count: s64;
    data: *u8;
}

Array_View_64 :: struct {
    count: s64;
    data: *u8;
}

Resizable_Array :: struct {
    count: s64;
    data: *void;
    allocated: s64;
    allocator: Allocator;
}

Any_Struct :: struct {
    type: *Type_Info;
    value_pointer: *void;
}

Support_User :: struct {
    text: string;
    view: [] u32;
    dynamic: [..] u8;
    boxed: Any;
    type: Type;
    callback: (value: s32) -> s32 #c_call;
}

main :: () {
    info: H.Header_Info;

    table_add(*info.exported_types, type_info(SmallMode), "SmallMode");
    normal := tprint("%../../../../target/developer-tools-enum-normal.h", #filepath);

    assert(H.generate_header(*info, normal));
    normal_text, normal_ok := read_entire_file(normal);

    defer free(normal_text);
    assert(normal_ok && contains(normal_text, "typedef enum SmallMode {"));
    assert(!contains(normal_text, "typedef enum SmallMode :"));
    info.enum_internal_types = true;
    info.prefix_enum_values = true;
    info.jai_type_prefix = "Jai";
    fixed := tprint("%../../../../target/developer-tools-enum-fixed.h", #filepath);

    assert(H.generate_header(*info, fixed));
    fixed_text, fixed_ok := read_entire_file(fixed);

    defer free(fixed_text);
    assert(fixed_ok && contains(fixed_text, "typedef enum JaiSmallMode : u8"));
    assert(contains(fixed_text, "JSM_Red = 0"));
    support_struct: Code_Struct;
    support_struct.defined_type = cast(*Type_Info_Struct) type_info(Newstring);
    structs: [1] Typechecked(Code_Struct) = .[.{ expression = *support_struct }];
    message: Message_Typechecked;
    message.kind = .TYPECHECKED;
    message.structs = structs;
    H.handle_message(*info, *message);
    assert(info.type_newstring == type_info(Newstring));
    info.type_array_view_64 = type_info(Array_View_64);
    info.type_resizable_array = type_info(Resizable_Array);
    info.type_any = type_info(Any_Struct);
    info.type_type_info = type_info(Type_Info);
    table_add(*info.exported_types, type_info(Support_User), "Support_User");
    table_add(*info.exported_types, type_info(First.Item), "Collision");
    table_add(*info.exported_types, type_info(Second.Item), "Collision");
    procedure: Code_Declaration;
    procedure.name = "support_sum";
    procedure.type = type_info(#type (value: *Support_User) -> s64 #c_call);
    array_add(*info.exported_procedures, *procedure);
    omitted: Code_Declaration;
    omitted.name = "non_c_call_must_be_omitted";
    omitted.type = type_info(#type () -> s64);
    array_add(*info.exported_procedures, *omitted);
    support := tprint("%../../../../target/developer-tools-support.h", #filepath);

    assert(H.generate_header(*info, support));
    text, ok := read_entire_file(support);

    defer free(text);
    assert(ok && contains(text, "void *callback;"));
    assert(!contains(text, "non_c_call_must_be_omitted"));
    assert(contains(text, "JaiCollision;") && contains(text, "JaiCollision1;"));
    print("ok\n");
}
