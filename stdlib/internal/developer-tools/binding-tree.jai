// Calls `visitor` for `decl` and every declaration below it, depth first, passing each its parent and `data`.
//
// A declaration is visited once, even when it is reachable twice. Returning `.RECURSE` visits its children,
// which are the members of a namespace, struct, enum or bitfield group, the arguments of a function, and for
// a struct also its metadata, parents, template parameters and virtual functions; returning `.STOP` skips
// them but does not end the walk. `parent` and `depth` describe where `decl` sits, and `depth` counts up from
// the given value. Asserts that `visitor` is not null, and that the walk does not go 256 levels deep.
visit_declarations :: (visitor: Declaration_Visitor_With_Data, data: *void, decl: *Declaration, parent: *Declaration = null, depth := 0) {
    assert(visitor != null && depth >= 0, "invalid declaration visitor");
    seen: [..] *Declaration;
    defer array_free(seen);
    walk_declarations(visitor, data, decl, parent, depth, *seen);
}

// Does the same as the overload that takes `data`, for a visitor that needs none.
visit_declarations :: (visitor: Declaration_Visitor, decl: *Declaration, parent: *Declaration = null, depth := 0) {
    closure: Plain_Visitor = .{ visitor = visitor };
    visit_declarations(call_plain_visitor, *closure, decl, parent, depth);
}

// Returns the enum called `enum_name` that is visible from `decl`, or null.
//
// Matches the original or the output name. Searches the scope of `decl` (`decl` itself when it is a namespace
// or struct, otherwise its parent), then each enclosing scope, looking at the scope itself and at its direct
// members, and finally the global scope of the active generator. A cyclic parent chain logs an error and
// gives null.
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
    if gm_generator_problem().count return null;

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

// Returns a new type that refers to `_enum`, with the size, alignment and number flags of its underlying
// type.
//
// The type is allocated with `New`. Asserts that `_enum` is not null.
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

// Sets the type of `decl` to a new type that refers to `_enum`. Asserts that `decl` is not null.
change_type_to_enum :: (decl: *Declaration, _enum: *Enum) {
    assert(decl != null);
    decl.type = create_type_of_enum(_enum);
}

// Sets the type of `decl` to the enum called `enum_name` found by [find_parent_enum].
//
// Returns false and changes nothing when there is no such enum.
change_type_to_enum :: (decl: *Declaration, enum_name: string) -> bool {
    _enum := find_parent_enum(decl, enum_name);
    if !_enum return false;
    change_type_to_enum(decl, _enum);
    return true;
}

// Sets the return type of `function_type` to `new_return_type`. `name` is only used in the assertion message.
//
// Asserts that neither the function type nor the new type is null.
change_return_type :: (name: string, function_type: *Function_Type, new_return_type: *CType) {
    assert(function_type != null && new_return_type != null, "invalid return type change for %", name);
    function_type.return_type = new_return_type;
}

// Sets the return type of the function that `decl` declares or points to, to `new_return_type`.
//
// Asserts that `decl` is not null, and that it has a function type, looking through typedefs and pointers.
change_return_type :: (decl: *Declaration, new_return_type: *CType) {
    assert(decl != null);
    change_return_type(decl.name, declaration_function_type(decl), new_return_type);
}

// Sets the return type of `function_type` to a type that refers to `_enum`.
//
// `name` is only used in the assertion message. Asserts that `function_type` is not null.
change_return_type_to_enum :: (name: string, function_type: *Function_Type, _enum: *Enum) {
    assert(function_type != null, "no function type for %", name);
    change_return_type(name, function_type, create_type_of_enum(_enum));
}

// Sets the return type of the function that `decl` declares or points to, to a type that refers to `_enum`.
//
// Asserts that `decl` is not null.
change_return_type_to_enum :: (decl: *Declaration, _enum: *Enum) {
    assert(decl != null);
    change_return_type_to_enum(decl.name, declaration_function_type(decl), _enum);
}

// Sets the return type of the function that `decl` declares or points to, to the enum called `enum_name`
// found by [find_parent_enum].
//
// Returns false and changes nothing when there is no such enum or `decl` has no function type.
change_return_type_to_enum :: (decl: *Declaration, enum_name: string) -> bool {
    _enum := find_parent_enum(decl, enum_name);
    if !_enum || !declaration_function_type(decl) return false;
    change_return_type_to_enum(decl, _enum);
    return true;
}

// Sets the return type of `function_type` to the enum called `enum_name`.
//
// The enum is searched for from each argument of the function. Returns false and changes nothing when none is
// found.
change_return_type_to_enum :: (name: string, function_type: *Function_Type, enum_name: string) -> bool {
    _enum := find_function_enum(function_type, enum_name);
    if !_enum return false;
    change_return_type_to_enum(name, function_type, _enum);
    return true;
}

// Sets the type of the argument `argument_name` of `function_type` to a type that refers to `_enum`.
//
// The argument matches by original or output name. Returns false when `function_type` or `_enum` is null,
// which also logs an error naming `name`, or when there is no such argument.
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

// Sets the type of the argument `argument_name` of the function that `decl` declares or points to, to a type
// that refers to `_enum`.
//
// Returns false when `decl` is null, has no function type or has no such argument.
change_argument_type_to_enum :: (decl: *Declaration, argument_name: string, _enum: *Enum) -> bool {
    if !decl return false;
    return change_argument_type_to_enum(decl.name, declaration_function_type(decl), argument_name, _enum);
}

// Sets the type of the argument `argument_name` of the function that `decl` declares or points to, to the
// enum called `enum_name` found by [find_parent_enum].
//
// Returns false and changes nothing when there is no such enum or argument.
change_argument_type_to_enum :: (decl: *Declaration, argument_name: string, enum_name: string) -> bool {
    _enum := find_parent_enum(decl, enum_name);
    if !_enum return false;
    return change_argument_type_to_enum(decl, argument_name, _enum);
}

// Sets the type of the argument `argument_name` of `function_type` to the enum called `enum_name`.
//
// The enum is searched for from each argument of the function. Returns false and changes nothing when none is
// found or the function has no such argument.
change_argument_type_to_enum :: (name: string, function_type: *Function_Type, argument_name: string, enum_name: string) -> bool {
    _enum := find_function_enum(function_type, enum_name);
    if !_enum return false;
    return change_argument_type_to_enum(name, function_type, argument_name, _enum);
}

// Returns a new pointer type to `pointee`, with the size and alignment of a pointer on the host.
//
// The type is allocated with `New`. Asserts that `pointee` is not null.
create_pointer_to :: (pointee: *CType) -> *CType {
    assert(pointee != null);
    result := New(CType);
    result.pointer_to = pointee;
    result.size = size_of(*void);
    result.alignment = align_of(*void);

    return result;
}

// Returns a new integer type of `size` bytes, signed or unsigned.
//
// The type is allocated with `New` and its alignment equals its size. Asserts when `size` is not 1, 2, 4 or
// 8.
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
