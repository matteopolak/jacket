// Runtime operations use native opaque class handles, never Jai record storage.
create :: (info: *Type_Info, parent_info: *Type_Info, parent: *void, class_info: *Type_Info, id_info: *Type_Info, selector_info: *Type_Info) -> *void {
    Encoding.require(info != null && parent_info != null && info.type == .STRUCT && parent_info.type == .STRUCT,
                     "Objective-C class creation requires reflected record types");
    record := cast(*Type_Info_Struct) info;

    Encoding.require(record.members.count > 0, "Objective-C class record requires a first member");

    if !parent {
        Basic.log_error("Objective_C parent class % is unavailable", (cast(*Type_Info_Struct) parent_info).name);
        return null;
    }

    if instance_size(parent) != cast(u64) parent_info.runtime_size {
        Basic.log("Objective_C parent layout size differs for %", (cast(*Type_Info_Struct) parent_info).name, flags = .WARNING);
    }

    name := Native.terminated_copy(record.name);

    defer Native.release(name);
    result := allocate_class(parent, name, 0);
    if !result {
        Basic.log_error("Objective_C could not allocate class %", record.name);
        return null;
    }

    complete := false;

    defer if !complete dispose_class(result);

    for member, index: record.members {
        // The first reflected member is reserved for isa/the parent subobject.
        if index == 0 continue;
        if (member.flags & (.CONSTANT | .IMPORTED)) != 0 continue;
        if member.type.runtime_size == 0 continue;

        ivar_name := Native.terminated_copy(member.name);

        defer Native.release(ivar_name);
        builder: Basic.String_Builder;

        defer Basic.free_buffers(*builder);
        Encoding.encode(*builder, member.type, class_info, id_info, selector_info);
        encoded := Basic.builder_to_string(*builder);

        defer free_string(encoded);
        types := Native.terminated_copy(encoded);

        defer Native.release(types);
        align := Encoding.alignment(member.type);
        shift: u8;

        while align > 1 {
            align /= 2;
            shift += 1;
        }

        if add_ivar(result, ivar_name, cast(u64) member.type.runtime_size, shift, types) == cast(NativeBool) 0 {
            Basic.log_error("Objective_C could not add ivar % to %", member.name, record.name);
            return null;
        }
    }

    if instance_size(result) != cast(u64) info.runtime_size {
        Basic.log("Objective_C new class layout size differs for %", record.name, flags = .WARNING);
    }

    complete = true;

    return result;
}

install :: (target: *void, proc: Any, selector_name: string, class_info: *Type_Info, id_info: *Type_Info, selector_info: *Type_Info, is_class_method: bool) {
    Encoding.require(target != null && proc.type != null && proc.type.type == .PROCEDURE && proc.value_pointer != null,
                     "Objective-C method installation requires a class and procedure value");
    procedure := cast(*Type_Info_Procedure) proc.type;

    Encoding.validate_method(procedure, class_info, id_info, selector_info);
    encoding := method_encoding(procedure, class_info, id_info, selector_info);

    defer free_string(encoding);
    name := Native.terminated_copy(selector_name);

    defer Native.release(name);
    types := Native.terminated_copy(encoding);

    defer Native.release(types);
    actual := ifx is_class_method then metaclass(target) else target;
    implementation := (cast(**void) proc.value_pointer).*;

    Encoding.require(actual != null && implementation != null && add_method(actual, selector(name), implementation, types) != cast(NativeBool) 0,
                     "Objective-C runtime rejected method installation");
}

method_encoding :: (procedure: *Type_Info_Procedure, class_info: *Type_Info,
                    id_info: *Type_Info, selector_info: *Type_Info) -> string {
    Encoding.require(procedure != null && procedure.return_types.count <= 1,
                     "Objective-C function encoding requires at most one return");
    builder: Basic.String_Builder;

    defer Basic.free_buffers(*builder);
    if procedure.return_types.count == 0 Basic.append(*builder, "v");
    else Encoding.encode(*builder, procedure.return_types[0], class_info, id_info, selector_info);

    for argument: procedure.argument_types Encoding.encode(*builder, argument, class_info, id_info, selector_info);

    Encoding.require(!builder.failed, "Objective-C method encoding allocation failed");

    return Basic.builder_to_string(*builder);
}

free_string :: (value: string) {
    if value.data context.allocator.proc(.FREE, 0, value.count, value.data, context.allocator.data);
}

#scope_file
Basic :: #import "Basic";
Encoding :: #import "internal/bindings/objc-encoding";
Native :: #import "internal/bindings/native";

objc :: #library,system "objc";

#if CPU == .ARM64 || OS == .IOS {
    NativeBool :: bool;
} else {
    NativeBool :: s8;
}

allocate_class :: (parent: *void, name: *u8, extra: u64) -> *void #foreign objc "objc_allocateClassPair";
dispose_class :: (class: *void) #foreign objc "objc_disposeClassPair";
instance_size :: (class: *void) -> u64 #foreign objc "class_getInstanceSize";
add_ivar :: (class: *void, name: *u8, size: u64, alignment: u8, types: *u8) -> NativeBool #foreign objc "class_addIvar";
selector :: (name: *u8) -> *void #foreign objc "sel_registerName";
metaclass :: (object: *void) -> *void #foreign objc "object_getClass";
add_method :: (class: *void, selector: *void, implementation: *void, types: *u8) -> NativeBool #foreign objc "class_addMethod";
