#scope_module
#import "Basic";
GR_String :: #import "String";
GR_File :: #import "File";

gr_run :: (generator: *Generator, options: Generate_Bindings_Options) -> bool {
    if !generator {
        log_error("Bindings_Generator: run requires a Generator\n");
        return false;
    }

    owner := context._bindings_generator_session;
    if !owner || !owner.live || owner.generator != generator {
        owner = New(GM_Sessions.Session);
        owner.* = GM_Sessions.open(generator, TARGET_OS);
        owner.lifecycle_owned = true;
    }

    active := GM_Sessions.new_owned(owner, Generate_Bindings_Options);
    active.* = options;
    GM_Sessions.activate(generator, active, owner);

    if owner.target_os != TARGET_OS || owner.failed {
        gm_input_error("run requires a matching usable canonical Session");
        return false;
    }

    // The retained options record is stable. Callback/data pointers and option
    // collection storage remain caller-owned; the exact native receipt below
    // owns independent copies of every string used by subsequent evaluation.
    libraries_ok := gr_load_libraries(generator, owner, active);
    if !libraries_ok return false;

    input := gr_parse_input(owner, active);
    if !input || !gr_parse(generator, owner, input) return false;

    captured := context;

    clang_visitChildren(clang_getTranslationUnitCursor(generator.translation_unit), handle_toplevel_cursors, *captured);
    for declaration: generator.global_scope.members visit_declarations(gr_prepare_declaration, null, declaration);

    gr_macros(generator, owner, active);
    // Generic child preparation and grouping both change enumerator names.
    // Finalize every represented enum only after those stages, before visitors.
    for declaration: generator.global_scope.members visit_declarations(gr_prepare_enum_names, null, declaration);

    gr_merge_namespaces(*generator.global_scope.members, owner);

    if active.visitor_with_data {
        for declaration: generator.global_scope.members visit_declarations(active.visitor_with_data, active.visitor_data, declaration);
    }

    if active.visitor {
        for declaration: generator.global_scope.members visit_declarations(active.visitor, declaration);
    }

    for declaration: generator.global_scope.members visit_declarations(omit_unnecessary_typedefs_and_macros, null, declaration);

    for declaration: generator.global_scope.members {
        path: [..] *Declaration;

        defer array_free(path);

        if !gr_namespace_acyclic(declaration, *path) {
            log_error("Bindings_Generator: cyclic or excessive output namespace graph\n");
            return false;
        }
    }

    gr_final_view(generator, owner, active);

    // Unsupported later model diagnostics do not redefine the native
    // preparation result, and teardown belongs to the separate lifecycle API.
    return true;
}

gr_prepare_declaration :: (declaration: *Declaration, _parent: *Declaration, _data: *void) -> Declaration_Visit_Result {
    options := gm_active_options();
    owner := gm_current_session();

    if !declaration || !options || !owner return .STOP;

    declaration.output_name = GM_Sessions.copy_text(owner, avoid_reserved_names(stripped_name(declaration.name)));
    if declaration.kind == .NAMESPACE {
        declaration.decl_flags |= options.default_namespace_flags;
        namespace := cast(*Namespace) declaration;
        for name: options.flatten_namespaces if name == namespace.name || name == namespace.output_name namespace.flags |= .FLATTEN;
    }

    if declaration.kind == .FUNCTION {
        function := cast(*Function) declaration;
        fix_macos_foreign_name(*function.foreign_name);
        if !function.library function.library = find_lib_for_foreign_name(function.foreign_name);
        check_if_foreign_function_should_be_omitted(function);
    }

    if declaration.kind == .ENUM {
        enumeration := cast(*Enum) declaration;
        if options.alias_original_enum_names enumeration.flags |= .ALIAS_ORIGINAL_NAMES;
    }

    return .RECURSE;
}

gr_prepare_enum_names :: (declaration: *Declaration, _parent: *Declaration, _data: *void) -> Declaration_Visit_Result {
    if declaration.kind == .ENUM compute_enum_names(cast(*Enum) declaration);
    return .RECURSE;
}

gr_load_libraries :: (generator: *Generator, owner: *GM_Sessions.Session, options: *Generate_Bindings_Options) -> bool {
    ok := true;

    for system: 0..1 {
        records := ifx system == 0 then options.libraries else options.system_libraries;
        paths := ifx system == 0 then options.library_search_paths else options.system_library_search_paths;

        if records.count < 0 || (records.count > 0 && !records.data) || !gm_lifecycle_arguments_valid(paths) {
            gm_input_error("malformed library configuration");
            ok = false;
            continue;
        }

        for record: records {
            if !gm_lifecycle_arguments_valid(string.[record.identifier, record.filename]) {
                ok = false;
                continue;
            }

            library := os_load_library(paths, record.identifier, record.filename, system != 0,
                                       options.static_library_suffix, options.shared_library_suffix);
            if !library ok = false;
            else GM_Sessions.append_owned(owner, *generator.libraries, library);
        }
    }

    return ok;
}

gr_macro_group :: (declaration: *Declaration, options: *Generate_Bindings_Options) -> string {
    if options.convert_macro_value_to_enum_callback {
        name := options.convert_macro_value_to_enum_callback(declaration);
        if name return name;
    }

    for prefix: options.generate_enums_from_macros_with_prefixes {
        if prefix && GR_String.begins_with(declaration.name, prefix) {
            name := prefix;
            while name.count > 0 && name[name.count - 1] == #char "_" name.count -= 1;
            return name;
        }
    }

    return "";
}

gr_macros :: (generator: *Generator, owner: *GM_Sessions.Session, options: *Generate_Bindings_Options) {
    retained := 0;
    count := generator.global_scope.members.count;

    for index: 0..count - 1 {
        declaration := generator.global_scope.members[index];
        keep := true;

        if declaration.kind == .MACRO_DEFINITION {
            if declaration.macro_value_result == .NEEDS_CHECKING check_macro_value(declaration);

            if declaration.macro_value_result == .DISCARD keep = false;
            else if declaration.macro_value_result == .KEEP {
                automatic_unwrap := false;

                for prefix: options.macro_prefixes_to_unwrap {
                    if GR_String.begins_with(declaration.name, prefix) {
                        declaration.should_unwrap = true;
                        declaration.decl_flags |= .OMIT_FROM_OUTPUT;
                        automatic_unwrap = true;
                    }
                }

                group := gr_macro_group(declaration, options);
                if group {
                    enumeration, is_new := find_or_create_enum_for_macro(group, declaration);
                    if enumeration {
                        if is_new enumeration.output_name = GM_Sessions.copy_text(owner, avoid_reserved_names(stripped_name(enumeration.name)));
                        if options.alias_original_enum_names enumeration.flags |= .ALIAS_ORIGINAL_NAMES;

                        member := GM_Sessions.new_owned(owner, Declaration);
                        member.* = declaration.*;
                        member.kind = .DECLARATION;
                        // Group membership is nominal. Native expression validation
                        // does not choose or widen the enum's underlying CType.
                        member.type = gm_enum_identifier_type(enumeration);
                        if automatic_unwrap member.decl_flags &= ~Declaration_Flags.OMIT_FROM_OUTPUT;

                        member.parent = cast(*Declaration) enumeration;
                        member.output_name = GM_Sessions.copy_text(owner, get_enum_value_name(enumeration, declaration.name));
                        GM_Sessions.append_owned(owner, *enumeration.enumerates, member);
                        GM_Table.table_add(*generator.macros_converted_to_enum_values, declaration.name, enumeration);
                        keep = false;
                    }
                }
            }
        }

        if keep {
            generator.global_scope.members[retained] = declaration;
            retained += 1;
        }
    }

    generator.global_scope.members.count = retained;
    for enumeration: generator.enum_macros {
        if !array_find(generator.global_scope.members, cast(*Declaration) enumeration) {
            GM_Sessions.append_owned(owner, *generator.global_scope.members, cast(*Declaration) enumeration);
        }
    }
}

gr_merge_namespaces :: (members: *[..] *Declaration, owner: *GM_Sessions.Session) {
    retained := 0;
    count := members.count;

    for index: 0..count - 1 {
        declaration := members.data[index];
        merged := false;

        if declaration.kind == .NAMESPACE && !(declaration.decl_flags & .OMIT_FROM_OUTPUT) {
            namespace := cast(*Namespace) declaration;

            for previous_index: 0..retained - 1 {
                previous := members.data[previous_index];
                if previous.kind != .NAMESPACE || previous.output_name != namespace.output_name || (previous.decl_flags & .OMIT_FROM_OUTPUT) != 0 continue;

                destination := cast(*Namespace) previous;

                for child: namespace.declarations {
                    if array_find(destination.declarations, child) continue;
                    // Merge the visible collection while retaining native
                    // source ancestry used by C++ qualification/bridges.
                    GM_Sessions.append_owned(owner, *destination.declarations, child);
                }

                gr_merge_namespaces(*destination.declarations, owner);
                merged = true;
                break;
            }

            if !merged gr_merge_namespaces(*namespace.declarations, owner);
        }

        if !merged {
            members.data[retained] = declaration;
            retained += 1;
        }
    }

    members.count = retained;
}

// Visitors may rewire namespace members; the final view needs a finite tree.
gr_namespace_acyclic :: (declaration: *Declaration, path: *[..] *Declaration) -> bool {
    if !declaration || declaration.kind != .NAMESPACE return true;
    if path.count >= 128 || array_find(path.*, declaration) return false;

    array_add(path, declaration);

    defer path.count -= 1;

    for child: (cast(*Namespace) declaration).declarations if !gr_namespace_acyclic(child, path) return false;

    return true;
}

gr_namespace_visible :: (declaration: *Declaration, options: *Generate_Bindings_Options) -> bool {
    if !declaration return false;
    if declaration.kind != .NAMESPACE return (declaration.decl_flags & .OMIT_FROM_OUTPUT) == 0;

    namespace := cast(*Namespace) declaration;
    any := false;

    for child: namespace.declarations if gr_namespace_visible(child, options) any = true;

    namespace.flags &= ~Namespace.Flags.IS_EMPTY;
    if !any namespace.flags |= .IS_EMPTY;

    return any && (declaration.decl_flags & .OMIT_FROM_OUTPUT) == 0;
}

gr_flatten :: (members: [] *Declaration, output: *[..] *Declaration, options: *Generate_Bindings_Options) {
    for declaration: members {
        if !gr_namespace_visible(declaration, options) continue;

        if declaration.kind == .NAMESPACE {
            namespace := cast(*Namespace) declaration;
            if namespace.flags & .FLATTEN {
                gr_flatten(namespace.declarations, output, options);
                continue;
            }
        }

        if !options.omit_global_declarations || declaration.parent != null || declaration.kind == .NAMESPACE {
            if !array_find(output.*, declaration) array_add(output, declaration);
        }
    }
}

gr_final_view :: (generator: *Generator, owner: *GM_Sessions.Session, options: *Generate_Bindings_Options) {
    // Printing/tooling can grow its separate queue. The final view therefore
    // owns an independent stable allocation in the SAME model ledger.
    generator.declarations_for_global_scope.count = 0;
    view: [..] *Declaration;

    defer array_free(view);
    gr_flatten(generator.global_scope.members, *view, options);
    generator.flattened_global_scope_members = GM_Sessions.copy_array(owner, view);
}

// Retain the exact strings passed to the native parser. These allocations share
// the canonical model ledger and are released by its existing lifecycle.
gr_parse_input :: (owner: *GM_Sessions.Session, options: *Generate_Bindings_Options) -> *GM_Sessions.Parse_Input {
    if !gm_lifecycle_arguments_valid(options.source_files) ||
       !gm_lifecycle_arguments_valid(options.include_paths) ||
       !gm_lifecycle_arguments_valid(options.system_include_paths) ||
       !gm_lifecycle_arguments_valid(options.extra_clang_arguments) return null;

    if options.source_files.count == 0 {
        gm_input_error("generator requires at least one source input");
        return null;
    }

    if options.include_paths.count > 0x3ffffffe || options.system_include_paths.count > 0x3ffffffe ||
       options.include_paths.count * 2 + options.system_include_paths.count * 2 + options.extra_clang_arguments.count > 0x7ffffffb {
        gm_input_error("generator parse arguments exceed native bounds");
        return null;
    }

    triple := gm_lifecycle_target(options.*);
    if !triple {
        gm_input_error("generator parse target is unsupported");
        return null;
    }

    arguments: [..] string;

    defer array_free(arguments);
    array_add(*arguments, "-target");
    array_add(*arguments, triple);
    if options.try_to_preserve_comments array_add(*arguments, "-fparse-all-comments");

    for path: options.include_paths {
        array_add(*arguments, "-I");
        array_add(*arguments, path);
    }

    for path: options.system_include_paths {
        array_add(*arguments, "-isystem");
        array_add(*arguments, path);
    }

    array_add(*arguments, ..options.extra_clang_arguments);

    if arguments.count > 0x7fffffff {
        gm_input_error("generator parse arguments exceed native bounds");
        return null;
    }

    input := GM_Sessions.new_owned(owner, GM_Sessions.Parse_Input);
    input.arguments = GM_Sessions.copy_array(owner, arguments);
    for *argument: input.arguments argument.* = GM_Sessions.copy_text(owner, argument.*);

    input.filename = GM_Sessions.copy_text(owner, options.source_files[0]);
    if !input.filename {
        gm_input_error("generator source filename is empty");
        return null;
    }

    overlay: GM_Sessions.Source_Overlay;

    if options.source_files.count == 1 {
        contents, ok := GR_File.read_entire_file(input.filename);

        defer free(contents);

        if !ok {
            gm_input_error("generator could not read the source input");
            return null;
        }

        overlay.contents = GM_Sessions.copy_text(owner, contents);
    } else {
        combined: String_Builder;

        defer free_buffers(*combined);

        for source: options.source_files {
            absolute, ok := GR_File.get_absolute_path(source);

            defer free(absolute);

            if !ok || !absolute {
                gm_input_error("generator could not resolve an aggregate source input");
                return null;
            }

            for byte: absolute if byte == 0 || byte == #char "\n" || byte == #char "\r" || byte == #char "\"" {
                gm_input_error("generator aggregate include path is unrepresentable");
                return null;
            }

            print_to_builder(*combined, "#include \"%\"\n", absolute);
        }

        if combined.failed {
            gm_input_error("generator aggregate source allocation failed");
            return null;
        }

        text := builder_to_string(*combined);

        defer free(text);
        overlay.contents = GM_Sessions.copy_text(owner, text);
        // Clang infers a language from the primary filename when no explicit
        // -x was supplied. Preserve that property of the first source rather
        // than silently converting a .hpp/.cpp aggregate into C.
        extension := ".h";

        for byte, index: input.filename {
            if byte == #char "/" || byte == 92 extension = ".h";
            else if byte == #char "." extension = .{ input.filename.count - index, input.filename.data + index };
        }

        input.filename = GM_Sessions.copy_text(owner, tprint("%.__generator_run__%", input.filename, extension));
    }

    overlay.filename = input.filename;
    input.overlays = GM_Sessions.copy_array(owner, GM_Sessions.Source_Overlay.[overlay]);

    return input;
}

gr_parse :: (generator: *Generator, owner: *GM_Sessions.Session, input: *GM_Sessions.Parse_Input) -> bool {
    if generator.clang_index || generator.translation_unit {
        gm_input_error("generator parse cannot replace live native resources; deinitialize first");
        return false;
    }

    if input.overlays.count != 1 {
        gm_input_error("generator parse requires its captured primary overlay");
        return false;
    }

    source := input.overlays[0].contents;
    unsaved: CXUnsavedFile;

    #if size_of(type_of(unsaved.Length)) == 4 {
        if source.count > 0xffffffff {
            gm_input_error("generator source length exceeds native bounds");
            return false;
        }
    }

    filename := to_c_string(input.filename);

    defer free(filename);
    arguments: [..] *u8;

    defer array_free(arguments);
    for argument: input.arguments array_add(*arguments, to_c_string(argument));

    defer for argument: arguments free(argument);

    unsaved.Filename = filename;
    unsaved.Contents = source.data;
    unsaved.Length = cast(type_of(unsaved.Length)) source.count;
    generator.clang_index = clang_createIndex(0, 0);
    if !generator.clang_index {
        gm_input_error("generator could not create its native index");
        return false;
    }

    owner.parse_input = input;
    status := clang_parseTranslationUnit2(generator.clang_index, filename, arguments.data, cast(s32) arguments.count,
                                         *unsaved, 1, 1, *generator.translation_unit);
    if status != .Success || !generator.translation_unit {
        gm_input_error("generator native parse failed");
        return false;
    }

    failed := false;
    count := cast(s64) clang_getNumDiagnostics(generator.translation_unit);

    for index: 0..count - 1 {
        diagnostic := clang_getDiagnostic(generator.translation_unit, cast(u32) index);

        defer clang_disposeDiagnostic(diagnostic);
        severity := clang_getDiagnosticSeverity(diagnostic);
        if severity == .Error || severity == .Fatal failed = true;
        if severity == .Ignored continue;

        message := clang_formatDiagnostic(diagnostic, clang_defaultDiagnosticDisplayOptions());

        defer clang_disposeString(message);
        log("Bindings_Generator: %\n", to_string(clang_getCString(message)));
    }

    if failed {
        gm_input_error("generator source contains native errors");
        return false;
    }

    return true;
}

gr_system_ignored :: (cur: CXCursor, options: *Generate_Bindings_Options) -> bool {
    location := clang_getCursorLocation(cur);
    if clang_Location_isInSystemHeader(location) == 0 return false;

    spelling := clang_getCursorSpelling(cur);

    defer clang_disposeString(spelling);
    name := to_string(clang_getCString(spelling));
    for included: options.system_types_to_include if name == included return false;

    file: CXFile;

    clang_getExpansionLocation(location, *file, null, null, null);

    if file {
        filename := clang_getFileName(file);

        defer clang_disposeString(filename);
        path := to_string(clang_getCString(filename));

        for fragment: options.path_fragments_to_treat_as_non_system_paths {
            if fragment && GR_String.contains(path, fragment) return false;
        }
    }

    return true;
}

gr_toplevel :: (cur: CXCursor, parent: CXCursor) -> CXChildVisitResult {
    generator := gm_active_generator();
    options := gm_active_options();
    owner := gm_current_session();

    if !generator || !options || !owner || owner.failed {
        log_error("Bindings_Generator: top-level traversal requires a live canonical generator/options/Session\n");
        return .Break;
    }

    if cur.kind == .LinkageSpec || cur.kind == .UnexposedDecl return .Recurse;
    if gm_cursor_kind(cur) == .NONE && cur.kind != .MacroDefinition return .Continue;

    // The canonical macro constructor distinguishes builtin and caller -D
    // definitions. A missing source-file handle alone cannot distinguish them.
    effective_parent := parent;
    while effective_parent.kind == .LinkageSpec || effective_parent.kind == .UnexposedDecl {
        effective_parent = clang_getCursorSemanticParent(effective_parent);
    }

    declaration := create_declaration(cur, effective_parent, ignore_non_structs = gr_system_ignored(cur, options));
    if !declaration return ifx owner.failed then .Break else .Continue;

    for existing, index: generator.global_scope.members {
        if existing == declaration return .Continue;
        if !existing || existing.name != declaration.name continue;

        if declaration.kind == .MACRO_DEFINITION && existing.kind == .MACRO_DEFINITION {
            if macro_literals_are_equal(cast(*Literal) existing.expression, cast(*Literal) declaration.expression) return .Continue;

            log(cur, "warning: macro % replaces a differing previous definition\n", declaration.name);
            generator.global_scope.members[index] = declaration;
            gr_mark_system(cur, declaration, options);

            return .Continue;
        }

        if declaration.kind == .FUNCTION && existing.kind == .FUNCTION {
            if functions_are_equivalent(cast(*Function) existing, cast(*Function) declaration) declaration.decl_flags |= .OMIT_FROM_OUTPUT;
        }
    }

    gr_mark_system(cur, declaration, options);
    GM_Sessions.append_owned(owner, *generator.global_scope.members, declaration);

    return .Continue;
}

gr_mark_system :: (cur: CXCursor, declaration: *Declaration, options: *Generate_Bindings_Options) {
    if !gr_system_ignored(cur, options) return;
    declaration.decl_flags |= .OMIT_FROM_OUTPUT;
    gm_active_generator().counts.ignored_system_declarations += 1;
    if options.log_ignored_system_declarations log(cur, "ignored system declaration %\n", declaration.name);
}
