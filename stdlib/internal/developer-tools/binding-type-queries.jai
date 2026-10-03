size_from_number_flags :: (flags: Number_Flags) -> u32 {
    if flags & ._8BIT return 1;
    if flags & ._16BIT return 2;
    if flags & ._32BIT return 4;
    if flags & ._64BIT return 8;

    return 0;
}

type_has_name :: (type: *CType) -> bool {
    if !type return false;
    return !type.type_of_struct || type.type_of_struct.name.count > 0;
}

get_type_name :: (type: *CType, output_name := true, strip_pointers := false) -> string {
    seen: [..] *CType;

    defer array_free(seen);

    while type {
        if array_find(seen, type) {
            log_error("Bindings_Generator: cyclic pointer name query\n");
            return "";
        }

        array_add(*seen, type);

        if strip_pointers && type.pointer_to {
            type = type.pointer_to;
            continue;
        }

        declaration := query_named_declaration(type);
        if !declaration return "";

        return ifx output_name then declaration.output_name else declaration.name;
    }

    return "";
}

try_find_struct_in_type :: (type: *CType) -> *Struct {
    seen: [..] *CType;

    defer array_free(seen);

    while type {
        if array_find(seen, type) {
            log_error("Bindings_Generator: cyclic struct type query\n");
            return null;
        }

        array_add(*seen, type);
        if type.type_of_struct return type.type_of_struct;
        if type.pointer_to type = type.pointer_to;
        else if type.array_element_type type = type.array_element_type;
        else return null;
    }

    return null;
}

#scope_module
#import "Basic";

query_named_declaration :: (type: *CType) -> *Declaration {
    if type.type_of_typedef return cast(*Declaration) type.type_of_typedef;
    if type.type_of_enum return cast(*Declaration) type.type_of_enum;
    if type.type_of_struct return cast(*Declaration) type.type_of_struct;
    return type.type_of_declaration;
}
