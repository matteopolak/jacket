visit_declarations :: (visitor: Declaration_Visitor_With_Data, data: *void, decl: *Declaration, parent: *Declaration = null, depth := 0) {
    assert(visitor != null && depth >= 0, "invalid declaration visitor");
    seen: [..] *Declaration;
    defer array_free(seen);
    walk_declarations(visitor, data, decl, parent, depth, *seen);
}

visit_declarations :: (visitor: Declaration_Visitor, decl: *Declaration, parent: *Declaration = null, depth := 0) {
    closure: Plain_Visitor = .{ visitor = visitor };
    visit_declarations(call_plain_visitor, *closure, decl, parent, depth);
}

find_parent_enum :: (decl: *Declaration, enum_name: string) -> *Enum {
    if !decl || !enum_name return null;

    scope := decl.parent;
    if decl.kind == .NAMESPACE || decl.kind == .STRUCT scope = decl;

    seen: [..] *Declaration;

    defer array_free(seen);

    while scope {
        if array_find(seen, scope) {
            log_error("Bindings_Generator: cyclic declaration parent chain\n");
            return null;
        }

        array_add(*seen, scope);
        if scope.kind == .ENUM && (scope.name == enum_name || scope.output_name == enum_name) return cast(*Enum) scope;

        members := declaration_scope_members(scope);
        for candidate: members if candidate && candidate.kind == .ENUM && (candidate.name == enum_name || candidate.output_name == enum_name) return cast(*Enum) candidate;

        scope = scope.parent;
    }

    // Native translation-unit parents have no Declaration: their real member
    // list is the active Generator's Block. Do not fabricate a parent record.
    if !context.generator return null;

    session := context._bindings_generator_session;
    if session && (!session.live || session.generator != context.generator || session.target_os != TARGET_OS) return null;

    generator := context.generator;
    if !gm_dependency_view_valid(generator.global_scope.members) {
        log_error("Bindings_Generator: malformed global enum lookup storage\n");
        return null;
    }

    for candidate: generator.global_scope.members {
        if candidate && candidate.kind == .ENUM && (candidate.name == enum_name || candidate.output_name == enum_name) return cast(*Enum) candidate;
    }

    return null;
}

create_type_of_enum :: (_enum: *Enum) -> *CType {
    assert(_enum != null, "enum type requires an enum declaration");
    result := New(CType);
    result.type_of_enum = _enum;

    if _enum.type {
        result.size = _enum.type.size;
        result.alignment = _enum.type.alignment;
        result.number_flags = _enum.type.number_flags;
    }

    return result;
}

change_type_to_enum :: (decl: *Declaration, _enum: *Enum) {
    assert(decl != null);
    decl.type = create_type_of_enum(_enum);
}

change_type_to_enum :: (decl: *Declaration, enum_name: string) -> bool {
    _enum := find_parent_enum(decl, enum_name);
    if !_enum return false;
    change_type_to_enum(decl, _enum);
    return true;
}

change_return_type :: (name: string, function_type: *Function_Type, new_return_type: *CType) {
    assert(function_type != null && new_return_type != null, "invalid return type change for %", name);
    function_type.return_type = new_return_type;
}

change_return_type :: (decl: *Declaration, new_return_type: *CType) {
    assert(decl != null);
    change_return_type(decl.name, declaration_function_type(decl), new_return_type);
}

change_return_type_to_enum :: (name: string, function_type: *Function_Type, _enum: *Enum) {
    assert(function_type != null, "no function type for %", name);
    change_return_type(name, function_type, create_type_of_enum(_enum));
}

change_return_type_to_enum :: (decl: *Declaration, _enum: *Enum) {
    assert(decl != null);
    change_return_type_to_enum(decl.name, declaration_function_type(decl), _enum);
}

change_return_type_to_enum :: (decl: *Declaration, enum_name: string) -> bool {
    _enum := find_parent_enum(decl, enum_name);
    if !_enum || !declaration_function_type(decl) return false;
    change_return_type_to_enum(decl, _enum);
    return true;
}

change_return_type_to_enum :: (name: string, function_type: *Function_Type, enum_name: string) -> bool {
    _enum := find_function_enum(function_type, enum_name);
    if !_enum return false;
    change_return_type_to_enum(name, function_type, _enum);
    return true;
}

change_argument_type_to_enum :: (name: string, function_type: *Function_Type, argument_name: string, _enum: *Enum) -> bool {
    if !function_type || !_enum {
        log_error("Bindings_Generator: no function or enum type for %\n", name);
        return false;
    }

    for argument: function_type.arguments if argument && (argument.name == argument_name || argument.output_name == argument_name) {
        change_type_to_enum(argument, _enum);
        return true;
    }

    return false;
}

change_argument_type_to_enum :: (decl: *Declaration, argument_name: string, _enum: *Enum) -> bool {
    if !decl return false;
    return change_argument_type_to_enum(decl.name, declaration_function_type(decl), argument_name, _enum);
}

change_argument_type_to_enum :: (decl: *Declaration, argument_name: string, enum_name: string) -> bool {
    _enum := find_parent_enum(decl, enum_name);
    if !_enum return false;
    return change_argument_type_to_enum(decl, argument_name, _enum);
}

change_argument_type_to_enum :: (name: string, function_type: *Function_Type, argument_name: string, enum_name: string) -> bool {
    _enum := find_function_enum(function_type, enum_name);
    if !_enum return false;
    return change_argument_type_to_enum(name, function_type, argument_name, _enum);
}

create_pointer_to :: (pointee: *CType) -> *CType {
    assert(pointee != null);
    result := New(CType);
    result.pointer_to = pointee;
    result.size = size_of(*void);
    result.alignment = align_of(*void);

    return result;
}

get_int_type :: (size: int, signed: bool) -> *CType {
    assert(size == 1 || size == 2 || size == 4 || size == 8, "unsupported integer byte width %", size);
    result := New(CType);
    result.size = size;
    result.alignment = size;
    if size == 1 result.number_flags = ._8BIT;
    if size == 2 result.number_flags = ._16BIT;
    if size == 4 result.number_flags = ._32BIT;
    if size == 8 result.number_flags = ._64BIT;
    if signed result.number_flags |= .SIGNED;

    return result;
}

#scope_file
#import "Basic";

Plain_Visitor :: struct {
    visitor: Declaration_Visitor;
}

call_plain_visitor :: (decl: *Declaration, parent: *Declaration, data: *void) -> Declaration_Visit_Result {
    closure := cast(*Plain_Visitor) data;
    assert(closure.visitor != null);
    return closure.visitor(decl, parent);
}

declaration_scope_members :: (decl: *Declaration) -> [] *Declaration {
    if decl.kind == .NAMESPACE return (cast(*Namespace) decl).declarations;
    if decl.kind == .STRUCT return (cast(*Struct) decl).declarations;
    if decl.kind == .ENUM return (cast(*Enum) decl).enumerates;
    if decl.kind == .BITFIELD return (cast(*Bitfield) decl).members;

    if decl.kind == .FUNCTION {
        type := declaration_function_type(decl);
        if type return type.arguments;
    }

    return .{};
}

walk_declarations :: (visitor: Declaration_Visitor_With_Data, data: *void, decl: *Declaration, parent: *Declaration, depth: s64, seen: *[..] *Declaration) {
    if !decl || array_find(seen.*, decl) return;

    assert(depth < 256, "declaration traversal exceeds supported depth");
    array_add(seen, decl);
    if visitor(decl, parent, data) == .STOP return;

    children: [..] *Declaration;

    defer array_free(children);
    for child: declaration_scope_members(decl) array_add(*children, child);

    if decl.kind == .STRUCT {
        st := cast(*Struct) decl;
        for child: st.metadata array_add(*children, child);
        for child: st.parents array_add(*children, child);
        for child: st.template_type_params array_add(*children, child);
        for child: st.virtual_functions array_add(*children, cast(*Declaration) child);
    }

    for child: children walk_declarations(visitor, data, child, decl, depth + 1, seen);
}

declaration_function_type :: (decl: *Declaration) -> *Function_Type {
    if !decl return null;

    type := decl.type;
    seen: [..] *CType;

    defer array_free(seen);

    while type {
        if array_find(seen, type) return null;

        array_add(*seen, type);
        if type.type_of_function return type.type_of_function;
        if type.type_of_typedef type = type.type_of_typedef.type;
        else if type.pointer_to type = type.pointer_to;
        else break;
    }

    return null;
}

find_function_enum :: (function_type: *Function_Type, enum_name: string) -> *Enum {
    if !function_type return null;

    for argument: function_type.arguments {
        candidate := find_parent_enum(argument, enum_name);
        if candidate return candidate;
    }

    return null;
}
