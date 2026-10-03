verify_elsewhere_binding :: (header: *Code_Procedure_Header, os: Operating_System_Tag, cpu: CPU_Tag) -> bool {
    if !header return binding_problem(null, "null procedure header");
    if !(header.procedure_flags & .ELSEWHERE) || header.name == header.foreign_function_name return true;

    windows := os == .WINDOWS;
    supported := windows || os == .LINUX || os == .MACOS || os == .IOS || os == .ANDROID || (os == .NN_SWITCH2 && cpu == .ARM64);

    if !supported || (cpu != .X64 && cpu != .ARM64) {
        if !binding_target_reported {
            log("Check: mangled binding verification is unavailable for target %/%\n", os, cpu);
            binding_target_reported = true;
        }

        return true;
    }

    info := binding_base_type(header.type);
    if !info || info.type != .PROCEDURE return binding_problem(header, "missing procedure type metadata");

    procedure := cast(*Type_Info_Procedure) info;
    receiver: s64;

    if header.procedure_flags & .CPP_METHOD receiver = 1;
    if receiver > procedure.argument_types.count return binding_problem(header, "C++ method has no receiver argument");

    char_signed := !(cpu == .ARM64 && (os == .LINUX || os == .ANDROID || os == .NN_SWITCH2));
    signature := binding_parse(header.foreign_function_name.data, header.foreign_function_name.count, cast(s32) windows, ifx windows then 4 else 8, cast(s32) char_signed);

    if !signature return binding_problem(header, "native signature parser returned no result");

    #if DEBUG_BINDINGS {
        binding_pool_total += signature.used_bytes;
        binding_pool_peak = max(binding_pool_peak, signature.used_bytes);
    }

    if signature.status != 0 return binding_problem(header, tprint("could not parse foreign signature (%)", to_string(signature.error)));
    if signature.argument_count != procedure.argument_types.count - receiver return binding_problem(header, "foreign argument count does not match Jai declaration");

    for index: 0..signature.argument_count - 1 {
        if !binding_same_type(signature.arguments[index], procedure.argument_types[index + receiver])
            return binding_problem(header, tprint("foreign argument % type does not match Jai declaration", index + 1));
    }

    if signature.result && !binding_same_returns(signature.result, procedure.return_types)
        return binding_problem(header, "encoded foreign return type does not match Jai declaration");

    return true;
}

print_binding_check_memory_stats :: () {
    log("Check: mangled-type pool peak=% bytes cumulative=% bytes\n", binding_pool_peak, binding_pool_total);
}

finish_error_checking :: () {
    #if DEBUG_BINDINGS print_binding_check_memory_stats();
}

#scope_file
#import "Basic";
#import "Compiler";

Binding_Native_Type :: struct {
    kind: s32;
    bytes: s32;
    child: *Binding_Native_Type;
    name: *u8;
    extent: s64;
    arguments: **Binding_Native_Type;
    argument_count: s64;
}

Binding_Signature :: struct {
    result: *Binding_Native_Type;
    arguments: **Binding_Native_Type;
    argument_count: s64;
    used_bytes: s64;
    status: s32;
    error: *u8;
}

#assert size_of(Binding_Native_Type) == 48;
#assert size_of(Binding_Signature) == 48;

binding_native :: #foreign_library "check-native/developer_tools_check";
binding_parse :: (name: *u8, count: s64, windows: s32, long_bytes: s32, char_signed: s32) -> *Binding_Signature #foreign binding_native "developer_check_parse";
binding_pool_peak: s64;
binding_pool_total: s64;
binding_target_reported: bool;

binding_base_type :: (type: *Type_Info) -> *Type_Info {
    for depth: 0..63 {
        if !type || type.type != .VARIANT return type;
        type = (cast(*Type_Info_Variant) type).variant_of;
    }

    return null;
}

binding_same_returns :: (native: *Binding_Native_Type, returns: [] *Type_Info, depth := 0) -> bool {
    if returns.count == 0 return native && native.kind == 0;
    return returns.count == 1 && binding_same_type(native, returns[0], depth + 1);
}

binding_same_type :: (native: *Binding_Native_Type, type: *Type_Info, depth := 0) -> bool {
    if !native || depth >= 64 return false;

    type = binding_base_type(type);
    if !type return false;
    if native.kind == 0 return type.type == .VOID;
    if native.kind == 1 return type.type == .BOOL;

    if native.kind == 2 || native.kind == 3 {
        if type.type != .INTEGER || type.runtime_size != native.bytes return false;
        return (cast(*Type_Info_Integer) type).signed == (native.kind == 2);
    }

    if native.kind == 4 return type.type == .FLOAT && type.runtime_size == native.bytes;

    if native.kind == 5 {
        // Jai procedure values already represent native function pointers.
        if type.type == .PROCEDURE && native.child && native.child.kind == 8 return binding_same_type(native.child, type, depth + 1);
        if type.type != .POINTER return false;
        return binding_same_type(native.child, (cast(*Type_Info_Pointer) type).pointer_to, depth + 1);
    }

    if native.kind == 6 {
        if !native.name return false;
        if type.type == .STRUCT return (cast(*Type_Info_Struct) type).name == to_string(native.name);
        if type.type == .ENUM return (cast(*Type_Info_Enum) type).name == to_string(native.name);
        return false;
    }

    if native.kind == 7 {
        if type.type != .ARRAY return false;
        array := cast(*Type_Info_Array) type;
        return array.array_type == .FIXED && array.array_count == native.extent && binding_same_type(native.child, array.element_type, depth + 1);
    }

    if native.kind == 8 {
        if type.type != .PROCEDURE return false;

        procedure := cast(*Type_Info_Procedure) type;
        if !(procedure.procedure_flags & .IS_C_CALL) || procedure.argument_types.count != native.argument_count return false;
        if !binding_same_returns(native.child, procedure.return_types, depth + 1) return false;
        for argument, index: procedure.argument_types if !binding_same_type(native.arguments[index], argument, depth + 1) return false;

        return true;
    }

    return false;
}

binding_problem :: (header: *Code_Procedure_Header, message: string) -> bool {
    location: Source_Code_Location;

    if header {
        location.line_number = header.l0;
        location.character_number = header.c0;
        if header.enclosing_load location.fully_pathed_filename = header.enclosing_load.fully_pathed_filename;
    }

    compiler_report(tprint("Check: %", message), loc = location, mode = .ERROR_CONTINUABLE);

    return false;
}
