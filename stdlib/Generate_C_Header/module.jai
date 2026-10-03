Header_Info :: struct {
    prefix_enum_values: bool;
    enum_internal_types: bool;
    jai_type_prefix: string;
    exported_procedures: [..] *Code_Declaration;
    exported_types: Table(*Type_Info, string);
    name_types: Table(string, *Type_Info);
    ordered_types: [..] *Type_Info;
    type_resizable_array: *Type_Info;
    type_array_view_8: *Type_Info;
    type_array_view_16: *Type_Info;
    type_array_view_32: *Type_Info;
    type_array_view_64: *Type_Info;
    type_type_info: *Type_Info;
    type_newstring: *Type_Info;
    type_any: *Type_Info;
}

#load "../internal/developer-tools/c-header.jai";

#scope_file
#import "Basic";
#import "Compiler";
#import "Hash_Table";
#import "File";
