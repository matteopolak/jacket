#scope_module
#import "Basic";
Binding_Pool :: #import "Pool";

binding_canonical_output :: (input: Generate_Bindings_Options, report: bool, bridge: bool) -> String_Builder, bool {
    result: String_Builder;
    result.allocator = context.allocator;
    state: Binding_Options_Normalization;
    state.options = input;
    defer binding_release_normalized_options(*state);
    if !binding_normalize_options(*state) return result, false;

    model_pool: Binding_Pool.Pool;

    Binding_Pool.set_allocators(*model_pool, context.allocator);
    defer Binding_Pool.release(*model_pool);

    push_context, defer_pop;
    push_allocator(Binding_Pool.pool_allocator_proc, *model_pool);
    generator: Generator;

    init_generator(*generator, state.options);
    defer deinit_generator(*generator);

    // Options without sources describe an empty header set (a build script may fill them in only
    // on some targets): the output has no declarations, which is not an error.
    if state.options.source_files.count > 0 && !run_generator(*generator, state.options) return result, false;
    if bridge binding_emit_canonical_bridge(*result, *generator, *state.options);
    else binding_emit_canonical_jai(*result, *generator, *state.options);

    if result.failed {
        free_buffers(*result);
        result.failed = true;
        return result, false;
    }

    if report log("Bindings_Generator: emitted % bytes\n", builder_string_length(*result));

    return result, true;
}

Binding_Output_Frame :: struct {
    declaration: *Declaration;
    next_child: int = -1;
}

binding_walk_output :: (declaration: *Declaration, visitor: #type (declaration: *Declaration, data: *void) -> bool, data: *void) -> bool {
    frames: [..] Binding_Output_Frame;
    active: [..] *Declaration;

    defer array_free(frames);
    defer array_free(active);
    array_add(*frames, Binding_Output_Frame.{ declaration });

    while frames.count {
        frame := *frames[frames.count - 1];
        node := frame.declaration;

        if !node return false;

        if frame.next_child == -1 {
            if should_be_omitted(node, node.parent) {
                frames.count -= 1;
                continue;
            }

            if array_find(active, node) {
                log_error("Bindings_Generator: cyclic declaration tree cannot be collected\n");
                return false;
            }

            if !visitor(node, data) return false;

            array_add(*active, node);
            frame.next_child = 0;
        }

        children: [] *Declaration;
        if node.kind == .NAMESPACE children = (cast(*Namespace) node).declarations;
        else if node.kind == .STRUCT children = (cast(*Struct) node).declarations;
        if children.count < 0 || (children.count && !children.data) return false;

        if frame.next_child < children.count {
            child := children[frame.next_child];
            frame.next_child += 1;
            array_add(*frames, Binding_Output_Frame.{ child });
        } else {
            active.count -= 1;
            frames.count -= 1;
        }
    }

    return true;
}

binding_bridge_collect :: (declaration: *Declaration, raw_data: *void) -> bool {
    data := cast(*Bridge_Data) raw_data;

    if declaration.kind == .FUNCTION {
        if declaration.decl_flags & (.PRIVATE | .PROTECTED | .DEPRECATED) || is_operator(declaration) || !declaration.library return true;

        function := cast(*Function) declaration;
        if !function.type || !function.type.type_of_function return false;

        // This workflow adapts C++ linkage. An already-unmangled C export would
        // otherwise produce a same-symbol body calling/interposing itself.
        cursor := gm_declaration_cursor(declaration);
        if clang_Cursor_isNull(cursor) != 0 return true;

        mangling := clang_Cursor_getMangling(cursor);
        spelling := clang_getCursorSpelling(cursor);

        defer clang_disposeString(mangling);
        defer clang_disposeString(spelling);
        native_name := to_string(clang_getCString(mangling));

        fix_macos_foreign_name(*native_name);
        if !native_name || native_name == to_string(clang_getCString(spelling)) return true;

        signature := function.type.type_of_function;
        if signature.flags & (.OBJC_METHOD | .OBJC_CLASS_METHOD) return true;
        if !array_find(data.functions, function) array_add(*data.functions, function);
        if signature.flags & .CONSTRUCTOR data.needs_placement_new = true;

        base := strip_type(signature.return_type, typedefs = true);
        if base && base.type_of_struct && !base.pointer_to && !(base.flags & .REFERENCE) data.needs_placement_new = true;
    }

    return true;
}

binding_emit_canonical_bridge :: (builder: *String_Builder, generator: *Generator, options: *Generate_Bindings_Options) {
    append(builder, options.header);
    data: Bridge_Data;

    defer array_free(data.functions);

    for declaration: generator.flattened_global_scope_members if !binding_walk_output(declaration, binding_bridge_collect, *data) {
        fail_type_emission(builder);
        return;
    }

    if data.needs_placement_new append(builder, "\n#include <new>\n");

    existing: [..] *Declaration;

    defer array_free(existing);

    for function: data.functions {
        append_cpp_wrapper(builder, function, *existing);
        if builder.failed return;
    }

    append(builder, options.footer);
}

binding_collect_printed_records :: (declaration: *Declaration, raw_records: *void) -> bool {
    records := cast(*[..] *Struct) raw_records;

    if declaration.kind == .STRUCT {
        record := cast(*Struct) declaration;
        if !array_find(records.*, record) array_add(records, record);
    }

    return true;
}

binding_generated_libraries :: (builder: *String_Builder, generator: *Generator, options: *Generate_Bindings_Options) {
    for library: generator.libraries {
        if !library || !looks_like_valid_identifier(library.identifier) {
            fail_type_emission(builder);
            return;
        }

        print_to_builder(builder, "% :: #library", library.identifier);
        if library.is_system_library append(builder, ",system");
        if !library.is_shared_library append(builder, ",no_dll");
        if library.flags & .LINK_ALWAYS append(builder, ",link_always");

        append(builder, " ");
        path := ifx library.is_system_library then library.full_path else tprint("%%", options.generated_library_path_prefix, library.full_path);

        emit_model_quoted(builder, path);
        append(builder, ";\n");
    }
}

binding_generated_checks :: (builder: *String_Builder, records: [] *Struct) {
    for record: records {
        if !record || !record.type || !record.output_name || record.flags & (.IS_ANON_INLINE_TYPE | .NO_STRUCT_CHECKS) || record.decl_flags & (.OMIT_FROM_CHECKS | .TEMPLATE) continue;
        if record.template_type_params.count > 0 continue;
        if record.type.size < 0 || record.type.alignment < 1 continue;

        name := type_to_string(record.type, null);

        defer free(name);

        if !name {
            fail_type_emission(builder);
            return;
        }

        print_to_builder(builder, "#assert(size_of(%) == %);\n#assert(align_of(%) == %);\n", name, record.type.size, name, record.type.alignment);
        info, _created := find_or_create_extra_info(record.type, cast(*Declaration) record);
        if !info || info == *struct_extra_info_unknown {
            fail_type_emission(builder);
            return;
        }

        for field: info.modified_declarations {
            if !field || (field.kind != .DECLARATION && field.kind != .BITFIELD) || field.storage_class == .Static || field.decl_flags & (.OMIT_FROM_OUTPUT | .OMIT_FROM_CHECKS) continue;
            if field.bit_offset_in_parent < 0 || field.bit_offset_in_parent % 8 != 0 || !field.output_name continue;

            print_to_builder(builder, "#assert(offset_of(%, ", name);
            emit_model_quoted(builder, field.output_name);
            print_to_builder(builder, ") == %);\n", field.bit_offset_in_parent / 8);
        }
    }
}

binding_emit_canonical_jai :: (builder: *String_Builder, generator: *Generator, options: *Generate_Bindings_Options) {
    if options.will_print_bindings options.will_print_bindings();

    append(builder, "// Generated by Bindings_Generator.\n");

    if options.add_generator_command {
        args := get_command_line_arguments();

        defer array_free(args);

        if args.count {
            name := args[0];
            for byte, index: args[0] if byte == #char "/" || byte == #char "\\" name = string.{ args[0].count - index - 1, args[0].data + index + 1 };

            append(builder, "// Generator command: ");
            emit_model_quoted(builder, name);
            append(builder, "\n");
        }
    }

    append(builder, options.header);
    existing: [..] *Declaration;
    records: [..] *Struct;

    defer array_free(existing);
    defer array_free(records);

    for declaration: generator.flattened_global_scope_members {
        if should_be_omitted(declaration, declaration.parent) continue;

        avoid_name_clash(*existing, declaration);
        print_declaration_to_builder(builder, declaration, null, .IS_GLOBAL, existing_declarations = *existing);
        if builder.failed return;

        if !binding_walk_output(declaration, binding_collect_printed_records, *records) {
            fail_type_emission(builder);
            return;
        }
    }

    index := 0;
    while index < generator.declarations_for_global_scope.count {
        declaration := generator.declarations_for_global_scope[index];

        index += 1;
        if array_find(existing, declaration) || should_be_omitted(declaration, declaration.parent) continue;

        avoid_name_clash(*existing, declaration, try_parent_prefix = true);
        print_declaration_to_builder(builder, declaration, null, .IS_GLOBAL, existing_declarations = *existing);
        if builder.failed return;

        if !binding_walk_output(declaration, binding_collect_printed_records, *records) {
            fail_type_emission(builder);
            return;
        }
    }

    append(builder, "\n#scope_file\n");
    if generator.generated_printf_wrapper || generator.int128_was_referenced append(builder, "#import \"Basic\";\n");
    if options.generate_library_declarations binding_generated_libraries(builder, generator, options);
    if options.generate_compile_time_struct_checks binding_generated_checks(builder, records);

    if generator.pending_bitfields.count {
        append(builder, "\n#scope_export\n");

        for group: generator.pending_bitfields {
            scope := cast(*Struct) group.parent;
            while scope && scope.flags & .IS_ANON {
                if !scope.parent || scope.parent.kind != .STRUCT {
                    fail_type_emission(builder);
                    return;
                }

                scope = cast(*Struct) scope.parent;
            }

            if !scope {
                fail_type_emission(builder);
                return;
            }

            append_bitfield_helpers(builder, group, scope);
        }
    }

    append(builder, options.footer);
}
