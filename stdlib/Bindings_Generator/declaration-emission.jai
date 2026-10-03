// Public emission entry points; algorithms and state stay in the shared private implementation.

append_cpp_wrapper :: (builder: *String_Builder, func: *Function, existing_declarations: *[..] *Declaration) {
    fragment: String_Builder;

    defer free_buffers(*fragment);
    valid := func != null && func.type != null && func.type.type_of_function != null;
    if valid {
        prefix: String_Builder;

        defer free_buffers(*prefix);
        append_parent_path(*prefix, func.parent, null, func.location, "_", true);
        append(*prefix, func.output_name);
        if func.decl_flags & .NEEDS_PRINTF_WRAPPER append(*prefix, "_message");

        repaired := builder_to_string(*prefix);

        binding_store_repaired_name(cast(*Declaration) func, repaired);
        avoid_name_clash(existing_declarations, cast(*Declaration) func, false);
        valid = binding_cpp_bridge(*fragment, func);
    }

    commit_type_fragment(builder, *fragment, valid);
}

print_declaration_to_builder :: (builder: *String_Builder, _decl: *Declaration, current_scope: *Declaration, flags: Print_Decl_Flags = 0, align_declarations := 0, existing_declarations: *[..] *Declaration = null, explicit_alignment := 0) {
    fragment: String_Builder;

    defer free_buffers(*fragment);
    valid := _decl != null;
    if valid valid = binding_declaration(*fragment, _decl, current_scope, flags, align_declarations, existing_declarations, explicit_alignment);

    commit_type_fragment(builder, *fragment, valid);
}

print_function :: (builder: *String_Builder, func: *Function, current_scope: *Declaration, foreign_name: string) {
    fragment: String_Builder;

    defer free_buffers(*fragment);
    valid := func != null && func.type != null && func.type.type_of_function != null;
    if valid valid = binding_function(*fragment, func, current_scope, foreign_name);

    commit_type_fragment(builder, *fragment, valid);
    if valid && builder && !builder.failed gp_current_generator().counts.functions += 1;
}

print_struct :: (builder: *String_Builder, _struct: *Struct, existing_declarations: *[..] *Declaration, explicit_alignment := 0) {
    fragment: String_Builder;

    defer free_buffers(*fragment);
    valid := _struct != null && looks_like_valid_identifier(_struct.output_name);
    if valid {
        print_to_builder(*fragment, "% :: ", _struct.output_name);
        valid = binding_struct_body(*fragment, _struct, existing_declarations, explicit_alignment);
        append(*fragment, "\n");
    }

    commit_type_fragment(builder, *fragment, valid);
    if valid && builder && !builder.failed gp_current_generator().counts.structs += 1;
}

print_namespace :: (builder: *String_Builder, ns: *Namespace) {
    entered := BE_Traversal.enter(.DECLARATION_BODY, ns);

    defer if entered BE_Traversal.leave();
    fragment: String_Builder;

    defer free_buffers(*fragment);
    valid := entered && looks_like_valid_identifier(ns.output_name);
    generator := gp_current_generator();

    if !generator valid = false;

    if valid {
        print_to_builder(*fragment, "% :: struct {\n", ns.output_name);
        generator.tab_depth += 1;

        for declaration: ns.declarations {
            if !declaration {
                valid = false;
                break;
            }

            if should_be_omitted(declaration, cast(*Declaration) ns) continue;
            if declaration.kind == .NAMESPACE && (cast(*Namespace) declaration).flags & .IS_EMPTY continue;

            if should_be_moved_to_global_scope(declaration) {
                if !array_find(generator.declarations_for_global_scope, declaration) {
                    session := context._bindings_generator_session;
                    if session GM_Sessions.append_owned(session, *generator.declarations_for_global_scope, declaration);
                    else array_add(*generator.declarations_for_global_scope, declaration);
                }

                continue;
            }

            print_declaration_to_builder(*fragment, declaration, cast(*Declaration) ns);

            if fragment.failed {
                valid = false;
                break;
            }
        }

        generator.tab_depth -= 1;
        append_indentation(*fragment);
        append(*fragment, "}\n");
    }

    commit_type_fragment(builder, *fragment, valid);
}

print_struct_vtable :: (builder: *String_Builder, _struct: *Struct, extra_info: Struct_Extra_Info) {
    if !extra_info.vtable_struct return;

    print_struct(builder, extra_info.vtable_struct, null);
    if builder.failed return;

    fragment: String_Builder;

    defer free_buffers(*fragment);
    valid := binding_vtable_helpers(*fragment, _struct, extra_info);

    commit_type_fragment(builder, *fragment, valid);
}
