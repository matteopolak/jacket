// Marks C++ functions as omitted from the output, and passes other declarations to the visitors in
// `user_data`.
//
// A function whose foreign name differs from its name is given `.OMIT_FROM_OUTPUT` and the visit stops there.
// Any other declaration is offered to the `visitor` and `visitor_with_data` of the
// [Cpp_Functions_Visitor_Data] that `user_data` points to, and the result is `.STOP` when either returns it
// and `.RECURSE` otherwise. Asserts that `user_data` is not null.
omit_cpp_functions_visitor :: (decl: *Declaration, parent_decl: *Declaration, user_data: *void) -> Declaration_Visit_Result {
    if decl && decl.kind == .FUNCTION && decl.foreign_name != decl.name {
        decl.decl_flags |= .OMIT_FROM_OUTPUT;
        return .STOP;
    }

    assert(user_data != null, "Bindings_Generator: missing omission visitor closure");
    closure := cast(*Cpp_Functions_Visitor_Data) user_data;
    if closure.visitor && closure.visitor(decl, parent_decl) == .STOP return .STOP;
    if closure.visitor_with_data && closure.visitor_with_data(decl, parent_decl, closure.visitor_data) == .STOP return .STOP;

    return .RECURSE;
}

// Makes the output name of `decl` differ from every name in `existing_declarations`, and adds `decl` to them.
//
// Starts by escaping reserved names, and by appending `_` to a function named like a struct in the global
// scope. On a clash the name of an operator is converted with [replace_operator_name]; with
// `try_parent_prefix`, names of the parents are put in front one after another; then `_1`, `_2` and so on is
// appended until the name is free. Two functions with the same name and different signatures do not clash
// when `allow_overloads` is set. With `insert` false the list is not changed. A declaration named `_` is left
// alone and not added.
avoid_name_clash :: (existing_declarations: *[..] *Declaration, decl: *Declaration, allow_overloads := true, insert := true, try_parent_prefix := false) {
    assert(decl != null);
    original := decl.output_name;
    normalized := avoid_reserved_names(original);

    if normalized.data != original.data {
        static_result := binding_compact_name_is(original, "operator!") || binding_compact_name_is(original, "operator=");

        if normalized != original {
            session := context._bindings_generator_session;
            if session decl.output_name = GM_Sessions.copy_text(session, normalized);
            else decl.output_name = copy_string(normalized);
        }

        if !static_result free(normalized);
    }

    if decl.output_name == "_" return;

    generator := gp_current_generator();
    if decl.kind == .FUNCTION && generator {
        for member: generator.global_scope.members {
            if member && member.kind == .STRUCT && member.output_name == decl.output_name {
                repaired := sprint("%_", decl.output_name);
                binding_store_repaired_name(decl, repaired);
                break;
            }
        }
    }

    if existing_declarations && binding_name_collides(existing_declarations.*, decl, decl.output_name, allow_overloads) {
        if begins_with(decl.output_name, "operator") {
            repaired := replace_operator_name(decl.output_name);
            binding_store_repaired_name(decl, repaired);
        }

        base := decl.output_name;

        if try_parent_prefix {
            seen: [..] *Declaration;

            defer array_free(seen);
            parent := decl.parent;
            while parent {
                if array_find(seen, parent) {
                    log_error("Bindings_Generator: cyclic name-repair parent chain\n");
                    if context._bindings_generator_session context._bindings_generator_session.failed = true;
                    return;
                }

                array_add(*seen, parent);

                if parent.output_name {
                    repaired := sprint("%_%", parent.output_name, base);
                    binding_store_repaired_name(decl, repaired);
                    base = decl.output_name;
                    if !binding_name_collides(existing_declarations.*, decl, base, allow_overloads) break;
                }

                parent = parent.parent;
            }
        }

        if binding_name_collides(existing_declarations.*, decl, decl.output_name, allow_overloads) {
            for suffix: 1..existing_declarations.count + 1 {
                candidate := sprint("%_%", base, suffix);
                if !binding_name_collides(existing_declarations.*, decl, candidate, allow_overloads) {
                    binding_store_repaired_name(decl, candidate);
                    break;
                }

                free(candidate);
            }
        }
    }

    if existing_declarations && insert && !array_find(existing_declarations.*, decl) array_add(existing_declarations, decl);
}

// Appends the `#foreign` (for a function) or `#elsewhere` directive that binds `decl` to its library.
//
// Writes the library's identifier, then `foreign_name` in quotes when it differs from `decl_name`. A
// declaration without a library logs an error, writes `__UnknownLib` and marks the session failed.
append_elsewhere :: (builder: *String_Builder, decl: *Declaration, decl_name: string, foreign_name: string) {
    assert(builder != null && decl != null);
    append(builder, ifx decl.kind == .FUNCTION then "#foreign " else "#elsewhere ");

    if decl.library append(builder, decl.library.identifier);
    else {
        log_error("Bindings_Generator: declaration % has no resolved foreign library\n", decl_name);
        append(builder, "__UnknownLib");
        if context._bindings_generator_session context._bindings_generator_session.failed = true;
    }

    if foreign_name && foreign_name != decl_name {
        append(builder, " ");
        emit_model_quoted(builder, foreign_name);
    }
}

// Returns `name` without the first of the options' `strip_prefixes` that it starts with.
//
// Underscores that follow the prefix are dropped while an identifier still follows them, and a `_` is put in
// front when the result would not start like an identifier. Returns `name` unchanged when no prefix matches;
// the result may point into `name`, or be a new string in the `_` case. Asserts that options are active.
stripped_name :: (name: string) -> string {
    options := gp_current_options();

    assert(options != null, "Bindings_Generator: name stripping requires active options");
    if !name return name;

    for prefix: options.strip_prefixes {
        if !begins_with(name, prefix) continue;

        result := name;
        result.data += prefix.count;
        result.count -= prefix.count;
        while result.count > 1 && result.data[0] == #char "_" && is_valid_identifier_start(result.data[1]) {
            result.data += 1;
            result.count -= 1;
        }

        if result.count && !is_valid_identifier_start(result.data[0]) return sprint("_%", result);

        return result;
    }

    return name;
}

// Returns whether the typedef called `name` is replaced by its target in the output.
//
// True for names in `TYPEDEFS_TO_UNWRAP`, names starting with a prefix in `TYPEDEF_PREFIXES_TO_UNWRAP` or in
// the options' `typedef_prefixes_to_unwrap`, and names that start with `__` and end with `_t`. Asserts that
// options are active.
should_unwrap_typedef :: (name: string) -> bool {
    for candidate: TYPEDEFS_TO_UNWRAP if candidate == name return true;
    for prefix: TYPEDEF_PREFIXES_TO_UNWRAP if begins_with(name, prefix) return true;
    if begins_with(name, "__") && ends_with(name, "_t") return true;

    options := gp_current_options();

    assert(options != null, "Bindings_Generator: typedef policy requires active options");
    for prefix: options.typedef_prefixes_to_unwrap if begins_with(name, prefix) return true;

    return false;
}

// Returns whether the typedef at the cursor `cur` is replaced by its target in the output, by its spelling.
//
// See the overload that takes a name.
should_unwrap_typedef :: (cur: CXCursor) -> bool {
    spelling := clang_getCursorSpelling(cur);
    defer clang_disposeString(spelling);
    return should_unwrap_typedef(to_string(clang_getCString(spelling)));
}

// Returns whether `decl` is left out of the output.
//
// True when it has `.OMIT_FROM_OUTPUT`, when it is a macro whose value was discarded, or when `parent_decl`
// or one of its ancestors is omitted. A cyclic parent chain logs an error and counts as omitted.
should_be_omitted :: (decl: *Declaration, parent_decl: *Declaration) -> bool {
    if decl && decl.decl_flags & .OMIT_FROM_OUTPUT return true;
    if decl && decl.kind == .MACRO_DEFINITION && decl.macro_value_result == .DISCARD return true;

    seen: [..] *Declaration;

    defer array_free(seen);
    parent := parent_decl;
    while parent {
        if parent.decl_flags & .OMIT_FROM_OUTPUT return true;

        if array_find(seen, parent) {
            log_error("Bindings_Generator: cyclic omission parent chain\n");
            return true;
        }

        array_add(*seen, parent);
        parent = parent.parent;
    }

    return false;
}

// Appends the output names of `decl` and of its ancestors up to `current_scope`, outermost first, each
// followed by `separator`.
//
// Names that `current_scope` or its ancestors already provide are left out, and so are flattened namespaces
// when `skip_flattened_namespaces` is set. Returns whether any scope was written. A cyclic parent chain logs
// an error, sets `builder.failed` and gives false. `location` is not used.
append_parent_path :: (builder: *String_Builder, decl: *Declaration, current_scope: *Declaration, location: Location, separator := ".", skip_flattened_namespaces := true) -> bool {
    scope_ancestors: [..] *Declaration;
    path: [..] *Declaration;

    defer array_free(scope_ancestors);
    defer array_free(path);
    ancestor := current_scope;
    while ancestor {
        if array_find(scope_ancestors, ancestor) {
            log_error("Bindings_Generator: cyclic current scope in parent path\n");
            if builder builder.failed = true;
            return false;
        }

        array_add(*scope_ancestors, ancestor);
        ancestor = ancestor.parent;
    }

    current := decl;
    while current && !array_find(scope_ancestors, current) {
        if array_find(path, current) {
            log_error("Bindings_Generator: cyclic declaration parent path\n");
            if builder builder.failed = true;
            return false;
        }

        array_add(*path, current);
        current = current.parent;
    }

    processed := false;

    for index: 0..path.count - 1 {
        node := path[path.count - 1 - index];
        if skip_flattened_namespaces && node.kind == .NAMESPACE && (cast(*Namespace) node).flags & .FLATTEN continue;

        processed = true;

        if node.output_name {
            append(builder, node.output_name);
            append(builder, separator);
        }
    }

    return processed;
}

// A declaration visitor that omits typedefs and macros that would print as a copy of their own name.
//
// Descends into namespaces and structs. For a typedef or macro whose printed type or value equals its output
// name, sets `.OMIT_FROM_OUTPUT`. Stops at every other declaration and at one it cannot print.
omit_unnecessary_typedefs_and_macros :: (decl: *Declaration, parent: *Declaration, user_data: *void) -> Declaration_Visit_Result {
    if !decl return .STOP;
    if decl.kind == .NAMESPACE || decl.kind == .STRUCT return .RECURSE;

    fragment: String_Builder;

    defer free_buffers(*fragment);
    if decl.kind == .TYPEDEF print_type_to_builder(*fragment, (cast(*Typedef) decl).type, parent);
    else if decl.kind == .MACRO_DEFINITION print_expression_to_builder(*fragment, decl.expression, current_scope = parent);
    else return .STOP;
    if fragment.failed return .STOP;

    text := builder_to_string(*fragment);

    defer free(text);
    if text == decl.output_name decl.decl_flags |= .OMIT_FROM_OUTPUT;

    return .STOP;
}

#scope_module

binding_compact_name_is :: (source: string, expected: string) -> bool {
    index := 0;

    for byte: source {
        if byte == #char " " continue;
        if index >= expected.count || byte != expected[index] return false;
        index += 1;
    }

    return index == expected.count;
}

binding_store_repaired_name :: (decl: *Declaration, owned_name: string) {
    session := context._bindings_generator_session;
    if session {
        decl.output_name = GM_Sessions.copy_text(session, owned_name);
        free(owned_name);
    } else decl.output_name = owned_name;
}

binding_name_collides :: (declarations: [] *Declaration, decl: *Declaration, candidate: string, allow_overloads: bool) -> bool {
    for previous: declarations {
        if !previous || previous == decl || previous.output_name != candidate continue;

        if previous.kind == .FUNCTION && decl.kind == .FUNCTION {
            left := cast(*Function) previous;
            right := cast(*Function) decl;

            if left.type && left.type.type_of_function left.type.type_of_function.flags |= .ADD_C_TYPE_DETAILS;
            if right.type && right.type.type_of_function right.type.type_of_function.flags |= .ADD_C_TYPE_DETAILS;
            if allow_overloads && left.name == right.name && !functions_are_equivalent(left, right) continue;
        }

        return true;
    }

    return false;
}

#scope_file
#import "Basic";
#import "String";
