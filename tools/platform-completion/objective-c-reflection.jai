create_objc_class_member_stub :: (class_name: string, framework_name: string) -> string {
    Basic.assert(_sdk_objc_identifier(class_name), "Expected an Objective-C class identifier");
    Basic.assert(_sdk_objc_identifier(framework_name), "Expected a system framework identifier");
    native_class := objc_lookUpClass(class_name);
    Basic.assert(native_class != null, "Class must be loaded before introspection");
    builder: String_Builder;
    defer Basic.free_buffers(*builder);
    Basic.print_to_builder(*builder, "// Runtime API members for % from %. Instance size: % bytes.\n", class_name, framework_name, class_getInstanceSize(native_class));
    Basic.append(*builder, "// Paste inside a pointer-only SDK class record; private ivar layout is not reproduced.\n");
    for class_methods: 0..1 {
        owner := native_class;
        if class_methods == 1 owner = object_getClass(cast(id) native_class);
        count: u32;
        methods := class_copyMethodList(owner, *count);
        defer _sdk_objc_libc_free(methods);
        if methods == null continue;
        for index: 0..cast(s64) count-1 {
            method := methods[index];
            selector_name := _sdk_objc_c_string(sel_getName(method_getName(method)));
            encoding := _sdk_objc_c_string(method_getTypeEncoding(method));
            safe_selector := true;
            for byte: selector_name {
                if byte != 58 && byte != 95 && !(byte >= 65 && byte <= 90) && !(byte >= 97 && byte <= 122) && !(byte >= 48 && byte <= 57) safe_selector = false;
            }
            if !safe_selector continue;
            result_type: [4096] u8;
            method_getReturnType(method, *result_type[0], 4096);
            result, supported := _sdk_objc_jai_type(_sdk_objc_c_string(*result_type[0]));
            if !supported {
                Basic.print_to_builder(*builder, "// Requires a reviewed aggregate declaration: % [%]\n", selector_name, encoding);
                continue;
            }
            args: String_Builder;
            defer Basic.free_buffers(*args);
            argument_count := method_getNumberOfArguments(method);
            if argument_count < 2 continue;
            supported_arguments := true;
            argument_index: u32 = 2;
            while argument_index < argument_count {
                argument_type: [4096] u8;
                method_getArgumentType(method, argument_index, *argument_type[0], 4096);
                type_name, argument_supported := _sdk_objc_jai_type(_sdk_objc_c_string(*argument_type[0]));
                if !argument_supported || type_name == "void" { supported_arguments = false; break; }
                Basic.print_to_builder(*args, ", arg%: %", argument_index - 2, type_name);
                argument_index += 1;
            }
            if !supported_arguments {
                Basic.print_to_builder(*builder, "// Requires a reviewed argument declaration: % [%]\n", selector_name, encoding);
                continue;
            }
            arg_text := Basic.builder_to_string(*args);
            defer if arg_text.count > 0 Basic.free(arg_text);
            Basic.append(*builder, "objc_");
            for byte: selector_name {
                if byte == 58 Basic.append(*builder, cast(u8) 95);
                else if (byte >= 65 && byte <= 90) || (byte >= 97 && byte <= 122) || (byte >= 48 && byte <= 57) || byte == 95 Basic.append(*builder, byte);
                else Basic.append(*builder, cast(u8) 95);
            }
            if class_methods == 1 Basic.append(*builder, "_class");
            Basic.print_to_builder(*builder, " :: (self: *void%) -> % #no_context {\n", arg_text, result);
            Basic.print_to_builder(*builder, "    invoke: (sdk_receiver: *void, sdk_selector: SEL%) -> % #c_call;\n", arg_text, result);
            Basic.append(*builder, "    invoke = cast(type_of(invoke)) objc_msgSend;\n    ");
            if result != "void" Basic.append(*builder, "return ");
            if class_methods == 1 Basic.print_to_builder(*builder, "invoke(cast(*void) objc_getClass(cast(*u8) \"%\\0\"), sel_registerName(cast(*u8) \"%\\0\")", class_name, selector_name);
            else Basic.print_to_builder(*builder, "invoke(self, sel_registerName(cast(*u8) \"%\\0\")", selector_name);
            argument_index = 2;
            while argument_index < argument_count {
                Basic.print_to_builder(*builder, ", arg%", argument_index - 2);
                argument_index += 1;
            }
            Basic.append(*builder, ");\n}\n");
        }
    }
    return Basic.builder_to_string(*builder);
}
objc_create_class :: ($type: Type, $parent: Type) -> Class {
    info := type_info(type);
    parent_info := type_info(parent);
    Basic.assert(info.type == .STRUCT && parent_info.type == .STRUCT, "Objective-C classes require concrete records");
    record := cast(*Type_Info_Struct) info;
    native_parent := class(parent);
    Basic.assert(native_parent != null, "Native parent class is unavailable");
    Basic.assert(class_getInstanceSize(native_parent) == cast(u64) parent_info.runtime_size, "Jai base record must describe the full native parent instance prefix");
    Basic.assert(info.runtime_size >= parent_info.runtime_size, "Subclass storage is smaller than its parent");
    Basic.assert(_sdk_objc_identifier(record.name), "Class name must be a concrete Objective-C identifier");
    native_class := objc_allocateClassPair(native_parent, record.name, 0);
    Basic.assert(native_class != null, "Class allocation failed or name already registered");
    complete := false;
    defer if !complete objc_disposeClassPair(native_class);
    base_found := false;
    for member: record.members {
        flags := cast(u32) member.flags;
        if (flags & 2) != 0 continue;
        if (flags & 1) != 0 {
            if member.type.type != .PROCEDURE continue;
            offset := member.offset_into_constant_storage;
            Basic.assert(offset >= 0 && offset <= record.constant_storage.count - size_of(IMP), "Procedure constant storage is unavailable");
            function := Any.{type = member.type, value_pointer = record.constant_storage.data + offset};
            selector_builder: String_Builder;
            defer Basic.free_buffers(*selector_builder);
            leading_underscore := true;
            for character: member.name {
                if character != 95 leading_underscore = false;
                if character == 95 && !leading_underscore Basic.append(*selector_builder, cast(u8) 58);
                else Basic.append(*selector_builder, character);
            }
            selector_name := Basic.builder_to_string(*selector_builder);
            defer Basic.free(selector_name);
            for note: member.notes {
                if note.count > 14 && string.{14, note.data} == "objc_selector:" {
                    Basic.free(selector_name);
                    selector_name = Basic.copy_string(string.{note.count - 14, note.data + 14});
                }
            }
            class_method := false;
            for note: member.notes if note == "objc_class_method" class_method = true;
            if class_method objc_add_class_method(native_class, function, selector_name);
            else objc_add_instance_method(native_class, function, selector_name);
            continue;
        }
        if member.offset_in_bytes == 0 && member.type == parent_info {
            Basic.assert((flags & 16) != 0, "Native parent prefix must use #as");
            base_found = true;
            continue;
        }
        Basic.assert(member.offset_in_bytes >= parent_info.runtime_size, "Subclass field overlaps its native parent");
        Basic.assert(_sdk_objc_identifier(member.name), "Invalid native ivar name");
        alignment := _sdk_objc_alignment(member.type);
        alignment_log2: u8 = 0;
        while (cast(s64) 1 << alignment_log2) < alignment alignment_log2 += 1;
        encoded := objc_type_encode(member.type);
        defer Basic.free(encoded);
        name_bytes := Basic.to_c_string(member.name);
        defer Basic.free(name_bytes);
        encoding_bytes := Basic.to_c_string(encoded);
        defer Basic.free(encoding_bytes);
        Basic.assert(name_bytes != null && encoding_bytes != null, "Ivar metadata allocation failed");
        Basic.assert(class_addIvar(native_class, name_bytes, cast(u64) member.type.runtime_size, alignment_log2, encoding_bytes) != 0, "Native ivar registration failed");
        ivar := class_getInstanceVariable(native_class, name_bytes);
        Basic.assert(ivar != null && ivar_getOffset(ivar) == member.offset_in_bytes, "Jai and Objective-C ivar offsets disagree");
    }
    Basic.assert(base_found, "Subclass must embed its full #as parent record at offset zero");
    native_size := class_getInstanceSize(native_class);
    Basic.assert(native_size <= cast(u64) info.runtime_size, "Native class is larger than the Jai record");
    if native_size < cast(u64) info.runtime_size {
        // The C runtime can omit a record's final alignment padding. Represent
        // that trailing storage as a real private ivar before registration.
        padding := info.runtime_size - cast(s64) native_size;
        padding_name := cast(*u8) "__jai_sdk_tail_padding\0";
        Basic.assert(class_getInstanceVariable(native_class, padding_name) == null, "Reserved tail-padding ivar name is already in use");
        padding_encoding: String_Builder;
        defer Basic.free_buffers(*padding_encoding);
        Basic.print_to_builder(*padding_encoding, "[%C]", padding);
        encoding := Basic.builder_to_string(*padding_encoding);
        defer Basic.free(encoding);
        encoded := Basic.to_c_string(encoding);
        defer Basic.free(encoded);
        Basic.assert(encoded != null && class_addIvar(native_class, padding_name, cast(u64) padding, 0, encoded) != 0, "Native trailing storage registration failed");
    }
    Basic.assert(class_getInstanceSize(native_class) == cast(u64) info.runtime_size, "Jai and native class sizes disagree");
    complete = true;
    // The caller completes any additional methods before objc_finalize_class.
    return native_class;
}
objc_add_instance_method :: (my_class: Class, proc: Any, selector_name: string) {
    Basic.assert(my_class != null && proc.type != null && proc.type.type == .PROCEDURE && proc.value_pointer != null, "Expected a native class and procedure value");
    ty := cast(*Type_Info_Procedure) proc.type;
    type_check_objc_method_type(ty);
    colons: s64 = 0;
    for byte: selector_name { Basic.assert(byte != 0, "Selector contains an embedded zero"); if byte == 58 colons += 1; }
    Basic.assert(selector_name.count > 0 && colons == ty.argument_types.count - 2, "Selector arity differs from the C-call method");
    encoding := objc_func_encode(ty);
    defer Basic.free(encoding);
    bytes := Basic.to_c_string(encoding);
    defer Basic.free(bytes);
    Basic.assert(bytes != null, "Method encoding allocation failed");
    implementation := (cast(*IMP) proc.value_pointer).*;
    Basic.assert(implementation != null, "Null native method implementation");
    native_selector := cast(SEL) selector(selector_name);
    Basic.assert(native_selector != null && class_addMethod(my_class, native_selector, implementation, bytes) != 0, "Method registration failed or selector already exists");
}
objc_add_class_method :: (my_class: Class, proc: Any, selector_name: string) {
    Basic.assert(my_class != null, "Expected a native class");
    metaclass := object_getClass(cast(id) my_class);
    Basic.assert(metaclass != null && class_isMetaClass(metaclass) != 0, "Native metaclass is unavailable");
    objc_add_instance_method(metaclass, proc, selector_name);
}
objc_type_encode :: (builder: *String_Builder, info: *Type_Info, pointer_depth := 0) {
    Basic.assert(builder != null && info != null && pointer_depth >= 0 && pointer_depth < 64, "Invalid or recursively unbounded native type");
    if info.type == .INTEGER {
        integer := cast(*Type_Info_Integer) info;
        if info.runtime_size == 1 { if integer.signed Basic.append(builder, "c"); else Basic.append(builder, "C"); }
        else if info.runtime_size == 2 { if integer.signed Basic.append(builder, "s"); else Basic.append(builder, "S"); }
        else if info.runtime_size == 4 { if integer.signed Basic.append(builder, "i"); else Basic.append(builder, "I"); }
        else if info.runtime_size == 8 { if integer.signed Basic.append(builder, "q"); else Basic.append(builder, "Q"); }
        else Basic.assert(false, "Unsupported native integer width");
    } else if info.type == .FLOAT {
        if info.runtime_size == 4 Basic.append(builder, "f");
        else if info.runtime_size == 8 Basic.append(builder, "d");
        else Basic.assert(false, "Unsupported native floating width");
    } else if info.type == .BOOL Basic.append(builder, "B");
    else if info.type == .VOID Basic.append(builder, "v");
    else if info.type == .ENUM objc_type_encode(builder, cast(*Type_Info) (cast(*Type_Info_Enum) info).internal_type, pointer_depth);
    else if info.type == .VARIANT objc_type_encode(builder, (cast(*Type_Info_Variant) info).variant_of, pointer_depth);
    else if info.type == .PROCEDURE {
        procedure := cast(*Type_Info_Procedure) info;
        Basic.assert((cast(u32) procedure.procedure_flags & 32) != 0, "Native function pointers require #c_call");
        Basic.append(builder, "^?");
    } else if info.type == .POINTER {
        pointed := (cast(*Type_Info_Pointer) info).pointer_to;
        if info == type_info(Class) Basic.append(builder, "#");
        else if info == type_info(SEL) Basic.append(builder, ":");
        else if pointed.type == .STRUCT && has_isa_member(cast(*Type_Info_Struct) pointed) Basic.append(builder, "@");
        else {
            Basic.append(builder, "^");
            objc_type_encode(builder, pointed, pointer_depth + 1);
        }
    } else if info.type == .STRING Basic.append(builder, "{JaiString=q^C}");
    else if info.type == .ARRAY {
        array := cast(*Type_Info_Array) info;
        Basic.assert(array.array_type == .FIXED && array.array_count >= 0, "Native arrays require a fixed element count");
        Basic.print_to_builder(builder, "[%", array.array_count);
        objc_type_encode(builder, array.element_type, pointer_depth);
        Basic.append(builder, "]");
    } else if info.type == .STRUCT {
        record := cast(*Type_Info_Struct) info;
        is_union := (cast(u32) record.textual_flags & 2) != 0;
        if is_union Basic.append(builder, "("); else Basic.append(builder, "{");
        // Generic Jai names contain punctuation; anonymous native tags are valid.
        if _sdk_objc_identifier(record.name) Basic.append(builder, record.name); else Basic.append(builder, "?");
        if pointer_depth <= 1 {
            Basic.append(builder, "=");
            for member: record.members {
                if (cast(u32) member.flags & 3) != 0 continue;
                objc_type_encode(builder, member.type, pointer_depth);
            }
        }
        if is_union Basic.append(builder, ")"); else Basic.append(builder, "}");
    } else Basic.assert(false, "Type has no Objective-C native encoding");
}
objc_type_encode :: (info: *Type_Info) -> string {
    builder: String_Builder;
    defer Basic.free_buffers(*builder);
    objc_type_encode(*builder, info);
    return Basic.builder_to_string(*builder);
}
objc_func_encode :: (ty: *Type_Info_Procedure) -> string {
    type_check_objc_method_type(ty);
    builder: String_Builder;
    defer Basic.free_buffers(*builder);
    if ty.return_types.count == 0 Basic.append(*builder, "v");
    else objc_type_encode(*builder, ty.return_types[0]);
    // Runtime method encodings accept the compact form without stack offsets.
    Basic.append(*builder, "@:");
    index: s64 = 2;
    while index < ty.argument_types.count {
        objc_type_encode(*builder, ty.argument_types[index]);
        index += 1;
    }
    return Basic.builder_to_string(*builder);
}
type_check_objc_method_type :: (ty: *Type_Info_Procedure) {
    Basic.assert(ty != null && ty.type == .PROCEDURE, "Expected a procedure descriptor");
    Basic.assert((cast(u32) ty.procedure_flags & 32) != 0, "Objective-C methods require #c_call");
    Basic.assert((cast(u32) ty.procedure_flags & 6) == 0, "Objective-C methods cannot be compile-time-only or polymorphic");
    Basic.assert(ty.argument_types.count >= 2 && ty.return_types.count <= 1, "Methods require receiver/selector arguments and at most one native return");
    Basic.assert(ty.argument_types[0].type == .POINTER && ty.argument_types[1].type == .POINTER, "Receiver and selector must be pointers");
    builder: String_Builder;
    defer Basic.free_buffers(*builder);
    for argument: ty.argument_types {
        Basic.assert(argument.type != .VOID && argument.type != .ARRAY, "Native method parameters cannot be void or fixed arrays by value");
        objc_type_encode(*builder, argument);
    }
    for result: ty.return_types {
        Basic.assert(result.type != .ARRAY, "Native method cannot return a fixed array");
        objc_type_encode(*builder, result);
    }
}
// PRIVATE HELPERS
_sdk_objc_libc :: #system_library "libc";
_sdk_objc_libc_free :: (pointer: *void) #foreign _sdk_objc_libc "free";
_sdk_objc_c_string :: (pointer: *u8) -> string #no_context {
    if pointer == null return "";
    count: s64 = 0;
    while pointer[count] != 0 count += 1;
    return string.{count, pointer};
}
_sdk_objc_identifier :: (name: string) -> bool #no_context {
    if name.count == 0 return false;
    for byte, index: name {
        if byte == 95 || (byte >= 65 && byte <= 90) || (byte >= 97 && byte <= 122) continue;
        if index > 0 && byte >= 48 && byte <= 57 continue;
        return false;
    }
    return true;
}
_sdk_objc_alignment :: (info: *Type_Info) -> s64 {
    Basic.assert(info != null && info.runtime_size >= 0, "Invalid native layout descriptor");
    if info.type == .POINTER || info.type == .PROCEDURE || info.type == .STRING return 8;
    if info.type == .BOOL return 1;
    if info.type == .INTEGER || info.type == .FLOAT {
        Basic.assert(info.runtime_size == 1 || info.runtime_size == 2 || info.runtime_size == 4 || info.runtime_size == 8, "Unreviewed native scalar alignment");
        return info.runtime_size;
    }
    if info.type == .ENUM return _sdk_objc_alignment(cast(*Type_Info) (cast(*Type_Info_Enum) info).internal_type);
    if info.type == .VARIANT return _sdk_objc_alignment((cast(*Type_Info_Variant) info).variant_of);
    if info.type == .ARRAY return _sdk_objc_alignment((cast(*Type_Info_Array) info).element_type);
    Basic.assert(info.type == .STRUCT, "Unreviewed native layout alignment");
    record := cast(*Type_Info_Struct) info;
    if (cast(u32) record.textual_flags & 4) != 0 return 1;
    maximum: s64 = 1;
    for member: record.members {
        if (cast(u32) member.flags & 3) != 0 continue;
        alignment := _sdk_objc_alignment(member.type);
        if alignment > maximum maximum = alignment;
    }
    return maximum;
}
_sdk_objc_jai_type :: (encoding: string) -> string, bool #no_context {
    if encoding.count == 0 return "", false;
    index: s64 = 0;
    while index < encoding.count {
        byte := encoding[index];
        if byte == 114 || byte == 110 || byte == 78 || byte == 111 || byte == 79 || byte == 82 || byte == 86 index += 1;
        else break;
    }
    if index == encoding.count return "", false;
    code := encoding[index];
    if code == 99 return "s8", true;
    if code == 67 return "u8", true;
    if code == 115 return "s16", true;
    if code == 83 return "u16", true;
    if code == 105 || code == 108 return "s32", true;
    if code == 73 || code == 76 return "u32", true;
    if code == 113 return "s64", true;
    if code == 81 return "u64", true;
    if code == 102 return "float", true;
    if code == 100 return "float64", true;
    if code == 66 return "bool", true;
    if code == 118 return "void", true;
    if code == 42 return "*u8", true;
    if code == 64 || code == 35 || code == 58 || code == 94 return "*void", true;
    return "", false;
}
