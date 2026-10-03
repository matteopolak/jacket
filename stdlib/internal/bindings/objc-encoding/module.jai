// Reviewed public type encodings from canonical reflection.
encode :: (builder: *Basic.String_Builder, info: *Type_Info, class_info: *Type_Info,
           id_info: *Type_Info, selector_info: *Type_Info, pointer_depth: s64 = 0) {
    require(builder != null && info != null, "invalid Objective-C encoding input");

    if info == class_info {
        Basic.append(builder, "#");
    } else if info == id_info {
        Basic.append(builder, "@");
    } else if info == selector_info {
        // Selector transparently aliases *void; both have this encoding.
        Basic.append(builder, ":");
    } else if info.type == .INTEGER {
        integer := cast(*Type_Info_Integer) info;
        signed_codes := "cslq";
        unsigned_codes := "CSLQ";
        index: s64;

        if info.runtime_size == 1 index = 0;
        else if info.runtime_size == 2 index = 1;
        else if info.runtime_size == 4 index = 2;
        else if info.runtime_size == 8 index = 3;
        else require(false, "unsupported Objective-C integer width");

        Basic.append(builder, ifx integer.signed then signed_codes[index] else unsigned_codes[index]);
    } else if info.type == .FLOAT {
        require(info.runtime_size == 4 || info.runtime_size == 8, "unsupported Objective-C floating width");
        Basic.append(builder, ifx info.runtime_size == 8 then "d" else "f");
    } else if info.type == .BOOL {
        Basic.append(builder, "B");
    } else if info.type == .VOID {
        Basic.append(builder, "v");
    } else if info.type == .ENUM {
        enumeration := cast(*Type_Info_Enum) info;
        encode(builder, enumeration.internal_type, class_info, id_info, selector_info, pointer_depth);
    } else if info.type == .PROCEDURE {
        Basic.append(builder, "?");
    } else if info.type == .STRING {
        Basic.append(builder, "{string=q^c}");
    } else if info.type == .POINTER {
        pointer := cast(*Type_Info_Pointer) info;
        if pointer.pointer_to.type == .INTEGER && pointer.pointer_to.runtime_size == 1 {
            Basic.append(builder, "*");
        } else {
            Basic.append(builder, "^");
            encode(builder, pointer.pointer_to, class_info, id_info, selector_info, pointer_depth + 1);
        }
    } else if info.type == .ARRAY {
        array := cast(*Type_Info_Array) info;
        if array.array_type == .VIEW {
            Basic.append(builder, "{_static=q^c}");
        } else if array.array_type == .RESIZABLE {
            Basic.append(builder, "{_dynamic=qq^v?^v}");
        } else {
            require(array.array_type == .FIXED, "unsupported Objective-C array category");
            Basic.append(builder, "[");
            require(Basic.print(builder, "%", array.array_count), "Objective-C encoding allocation failed");
            encode(builder, array.element_type, class_info, id_info, selector_info);
            Basic.append(builder, "]");
        }
    } else if info.type == .STRUCT {
        structure := cast(*Type_Info_Struct) info;
        if has_isa_member(structure, class_info) {
            Basic.append(builder, "@");
        } else {
            Basic.append(builder, "{");
            Basic.append(builder, structure.name);

            if pointer_depth < 2 {
                Basic.append(builder, "=");

                for member: structure.members {
                    if (member.flags & (.CONSTANT | .IMPORTED | .USING)) != 0 continue;
                    encode(builder, member.type, class_info, id_info, selector_info, pointer_depth);
                }
            }

            Basic.append(builder, "}");
        }
    } else {
        require(false, "reflected type has no supported Objective-C encoding");
    }

    require(!builder.failed, "Objective-C encoding allocation failed");
}

has_isa_member :: (structure: *Type_Info_Struct, class_info: *Type_Info) -> bool {
    require(structure != null, "missing Objective-C reflected record");

    for member: structure.members {
        if member.name != "isa" continue;
        require(member.type == class_info, "Objective-C isa member must have exact Class type");
        return true;
    }

    return false;
}

alignment :: (info: *Type_Info) -> s64 {
    require(info != null, "missing Objective-C reflected type");
    if info.type == .STRUCT return (cast(*Type_Info_Struct) info).alignment;

    if info.type == .ARRAY {
        array := cast(*Type_Info_Array) info;
        if array.array_type == .FIXED return alignment(array.element_type);
        return size_of(*void);
    }

    if info.type == .STRING return size_of(*void);
    if info.type == .ENUM return alignment((cast(*Type_Info_Enum) info).internal_type);

    require(info.type == .INTEGER || info.type == .FLOAT || info.type == .BOOL ||
            info.type == .POINTER || info.type == .PROCEDURE, "unsupported Objective-C alignment");
    require(info.runtime_size == 1 || info.runtime_size == 2 || info.runtime_size == 4 || info.runtime_size == 8,
            "unsupported Objective-C scalar layout");

    return info.runtime_size;
}

validate_method :: (procedure: *Type_Info_Procedure, class_info: *Type_Info,
                    id_info: *Type_Info, selector_info: *Type_Info) {
    require(procedure != null && (procedure.procedure_flags & .IS_C_CALL) != 0,
            "Objective-C method implementation requires C calling convention");
    require(procedure.argument_types.count >= 2 && procedure.return_types.count <= 1,
            "Objective-C method requires receiver/selector and at most one return");
    receiver := procedure.argument_types[0];
    eligible := receiver == id_info;

    if receiver.type == .POINTER {
        eligible = eligible || (cast(*Type_Info_Pointer) receiver).pointer_to.type == .STRUCT;
    } else if receiver.type == .STRUCT {
        eligible = has_isa_member(cast(*Type_Info_Struct) receiver, class_info);
    }

    require(eligible, "Objective-C method receiver has no supported object representation");
    require(procedure.argument_types[1] == selector_info, "Objective-C method requires exact Selector argument");
}

require :: (condition: bool, message: string) {
    if condition return;
    Basic.assert(false, message);
    // A returning assertion handler cannot authorize an unsafe native call.
    abort();
}

#scope_file
Basic :: #import "Basic";

libc :: #library,system "c";
abort :: () #foreign libc;
