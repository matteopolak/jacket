#scope_module
#import "Basic";

GM_LIFECYCLE_TYPES :: #string C
/* Independent builtin-type input for official SDK target layout queries. */
typedef void GM_Lifecycle_Void;
#ifdef __cplusplus
typedef bool GM_Lifecycle_Bool;
#else
__extension__ typedef _Bool GM_Lifecycle_Bool;
#endif
typedef char GM_Lifecycle_Char;
typedef signed char GM_Lifecycle_S8;
typedef unsigned char GM_Lifecycle_U8;
typedef short GM_Lifecycle_S16;
typedef unsigned short GM_Lifecycle_U16;
typedef int GM_Lifecycle_S32;
typedef unsigned int GM_Lifecycle_U32;
__extension__ typedef long long GM_Lifecycle_S64;
__extension__ typedef unsigned long long GM_Lifecycle_U64;
__extension__ typedef __int128 GM_Lifecycle_S128;
__extension__ typedef unsigned __int128 GM_Lifecycle_U128;
typedef float GM_Lifecycle_Float32;
typedef double GM_Lifecycle_Float64;
typedef void *GM_Lifecycle_Pointer;
C

gm_lifecycle_target :: (options: Generate_Bindings_Options) -> string {
    if options.cpu != .X64 && options.cpu != .ARM64 return "";

    arm := options.cpu == .ARM64;
    if options.os == .MACOS return ifx arm then "arm64-apple-darwin" else "x86_64-apple-darwin";
    if options.os == .WINDOWS return ifx arm then "aarch64-pc-windows-msvc" else "x86_64-pc-windows-msvc";
    if options.os == .LINUX return ifx arm then "aarch64-unknown-linux-gnu" else "x86_64-unknown-linux-gnu";
    if options.os == .ANDROID return ifx arm then "aarch64-linux-android" else "x86_64-linux-android";
    if options.os == .IOS return ifx arm then "arm64-apple-ios" else "x86_64-apple-ios-simulator";

    return "";
}

gm_lifecycle_primitives :: (generator: *Generator, options: Generate_Bindings_Options) -> bool {
    referenced := generator.int128_was_referenced;

    defer generator.int128_was_referenced = referenced;
    if !gm_lifecycle_arguments_valid(options.include_paths) || !gm_lifecycle_arguments_valid(options.system_include_paths) || !gm_lifecycle_arguments_valid(options.extra_clang_arguments) return false;

    if options.include_paths.count > 0x3ffffffe || options.system_include_paths.count > 0x3ffffffe ||
       options.include_paths.count * 2 + options.system_include_paths.count * 2 + options.extra_clang_arguments.count > 0x7ffffffb {
        gm_input_error("generator SDK argument vector exceeds native bounds");
        return false;
    }

    triple := gm_lifecycle_target(options);
    if !triple {
        gm_input_error("generator initialization lacks a supported SDK target for the requested OS/CPU");
        return false;
    }

    index := clang_createIndex(0, 0);
    if !index {
        gm_input_error("generator initialization could not create an SDK layout index");
        return false;
    }

    defer clang_disposeIndex(index);
    filename := to_c_string("generator-lifecycle-types.c");
    source := GM_LIFECYCLE_TYPES;
    unsaved: CXUnsavedFile;
    unsaved.Filename = filename;
    unsaved.Contents = source.data;
    unsaved.Length = cast(type_of(unsaved.Length)) source.count;

    defer free(filename);
    arguments: [..] *u8;

    defer array_free(arguments);
    array_add(*arguments, to_c_string("-x"));
    array_add(*arguments, to_c_string("c"));
    array_add(*arguments, to_c_string("-target"));
    array_add(*arguments, to_c_string(triple));

    for path: options.include_paths {
        array_add(*arguments, to_c_string("-I"));
        array_add(*arguments, to_c_string(path));
    }

    for path: options.system_include_paths {
        array_add(*arguments, to_c_string("-isystem"));
        array_add(*arguments, to_c_string(path));
    }

    for argument: options.extra_clang_arguments array_add(*arguments, to_c_string(argument));

    defer for argument: arguments free(argument);
    unit: CXTranslationUnit;
    status := clang_parseTranslationUnit2(index, filename, arguments.data, cast(s32) arguments.count, *unsaved, 1, 0, *unit);

    defer if unit clang_disposeTranslationUnit(unit);

    if status != .Success || !unit || clang_getNumDiagnostics(unit) != 0 {
        gm_input_error("generator initialization could not obtain SDK builtin type layouts");
        return false;
    }

    children := native_children(clang_getTranslationUnitCursor(unit));

    defer array_free(children);

    for child: children {
        if child.kind != .TypedefDecl continue;
        if clang_Location_isFromMainFile(clang_getCursorLocation(child)) == 0 continue;

        native := clang_getTypedefDeclUnderlyingType(child);
        if native.kind == .Pointer {
            width := clang_Type_getSizeOf(native);
            if width <= 0 || (generator.pointer_size > 0 && generator.pointer_size != width) {
                gm_input_error("generator pointer width disagrees with its SDK target");
                return false;
            }

            generator.pointer_size = width;
            if !generator.type_def_pointer_to_void && !generator.type_def_ptr_void {
                pointer := gm_build_type(native, child, false, 0);
                if !pointer return false;

                pointer.flags |= .SHARED;
                generator.type_def_pointer_to_void = pointer;
                generator.type_def_ptr_void = pointer;
            } else {
                if !generator.type_def_pointer_to_void generator.type_def_pointer_to_void = generator.type_def_ptr_void;
                if !generator.type_def_ptr_void generator.type_def_ptr_void = generator.type_def_pointer_to_void;
            }
        } else if !gm_build_type(native, child, false, 0) {
            if !gm_current_session().failed gm_input_error("generator initialization encountered an unsupported builtin type");
            return false;
        }
    }

    if !generator.type_def_void || !generator.type_def_bool || !generator.type_def_char ||
       !generator.type_def_s8 || !generator.type_def_u8 || !generator.type_def_s16 || !generator.type_def_u16 ||
       !generator.type_def_s32 || !generator.type_def_u32 || !generator.type_def_s64 || !generator.type_def_u64 ||
       !generator.type_def_s128 || !generator.type_def_u128 || !generator.type_def_float32 || !generator.type_def_float64 ||
       !generator.type_def_pointer_to_void {
        gm_input_error("generator initialization did not obtain all required canonical primitive types");
        return false;
    }

    gm_named_canonical_type(*generator.type_def_objc_class, "Class");
    gm_named_canonical_type(*generator.type_def_objc_selector, "SEL");
    gm_named_canonical_type(*generator.type_def_objc_id, "ID");
    gm_named_canonical_type(*generator.type_def_type, "Type");
    gm_opaque_block_type();

    if !generator.type_def_jai_string {
        generator.type_def_jai_string = gm_named_canonical_type(*generator.type_def_jai_string, "string");
        generator.type_def_jai_string.size = generator.pointer_size * 2;
    }

    if !generator.type_def_msvc_destructor_flags generator.type_def_msvc_destructor_flags = generator.type_def_u32;

    return true;
}

gm_lifecycle_arguments_valid :: (values: [] string) -> bool {
    if values.count < 0 || values.count > 0x7fffffff || (values.count > 0 && !values.data) {
        gm_input_error("generator SDK argument array is malformed");
        return false;
    }

    for value: values {
        if value.count < 0 || (value.count > 0 && !value.data) {
            gm_input_error("generator SDK argument string is malformed");
            return false;
        }

        for byte: value if byte == 0 {
            gm_input_error("generator SDK argument contains an embedded NUL");
            return false;
        }
    }

    return true;
}

gm_init_generator :: (generator: *Generator, options: Generate_Bindings_Options) {
    if !generator {
        log_error("Bindings_Generator: initialization needs a Generator\n");
        return;
    }

    owner := context._bindings_generator_session;
    if !owner || !owner.live || owner.generator != generator {
        owner = New(GM_Sessions.Session);
        owner.* = GM_Sessions.open(generator, TARGET_OS);
        owner.lifecycle_owned = true;
    }

    GM_Sessions.activate(generator, null, owner);

    if owner.target_os != TARGET_OS {
        gm_input_error("generator initialization does not match the existing ownership target");
        return;
    }

    if owner.failed return;

    if !gm_lifecycle_primitives(generator, options) && !owner.failed gm_input_error("generator canonical primitive initialization failed");
}

gm_lifecycle_clear_model :: (generator: *Generator) {
    GM_Table.table_reset(*generator.declarations_by_cursor);
    GM_Table.table_reset(*generator.macros_converted_to_enum_values);
    GM_Table.table_reset(*generator.unwrapped_typedefs);
    GM_Table.table_reset(*generator.extra_infos);
    GM_Table.table_reset(*generator.filenames);
}

gm_deinit_generator :: (generator: *Generator) {
    if !generator {
        log_error("Bindings_Generator: deinitialization needs a Generator\n");
        return;
    }

    tokens_valid := generator.tokens_to_free.count >= 0 &&
                    generator.tokens_to_free.count <= 0x7fffffffffffffff / size_of(Token_Info) &&
                    (generator.tokens_to_free.count == 0 || generator.tokens_to_free.data != null);
    if !tokens_valid log_error("Bindings_Generator: malformed native token ownership list\n");

    if tokens_valid for token_set: generator.tokens_to_free {
        if !token_set.token_pointer && token_set.num_tokens == 0 continue;

        if !generator.translation_unit || !token_set.token_pointer {
            log_error("Bindings_Generator: native token ownership lacks a live translation unit or token pointer\n");
            continue;
        }

        clang_disposeTokens(generator.translation_unit, token_set.token_pointer, token_set.num_tokens);
    }

    if generator.translation_unit clang_disposeTranslationUnit(generator.translation_unit);
    if generator.clang_index clang_disposeIndex(generator.clang_index);

    generator.translation_unit = null;
    generator.clang_index = null;
    if tokens_valid generator.tokens_to_free.count = 0;

    owner := context._bindings_generator_session;
    if !owner || !owner.live || owner.generator != generator || !owner.lifecycle_owned return;

    gm_release_generator_model(generator, owner);
}

// One typed cleanup path for public init-owned and caller-owned orchestration.
// The lifecycle owner disposes native handles/tokens before releasing the graph.
gm_release_generator_model :: (generator: *Generator, owner: *GM_Sessions.Session) {
    if !generator || !owner || !owner.live || owner.generator != generator {
        log_error("Bindings_Generator: model release requires its matching live ownership ledger\n");
        return;
    }

    gm_lifecycle_clear_model(generator);
    allocator := owner.allocator;
    heap_owned := owner.lifecycle_owned;

    if context._bindings_generator_session == owner {
        context._bindings_generator_session = null;
        context._bindings_generator_options = null;
    }

    GM_Sessions.close(owner);
    if heap_owned allocator.proc(.FREE, 0, size_of(GM_Sessions.Session), owner, allocator.data);

    gm_lifecycle_clear_retained(generator);
}

gm_lifecycle_clear_retained :: (generator: *Generator) {
    generator.global_scope = .{};
    generator.type_def_void = null;
    generator.type_def_bool = null;
    generator.type_def_char = null;
    generator.type_def_u8 = null;
    generator.type_def_u16 = null;
    generator.type_def_u32 = null;
    generator.type_def_u64 = null;
    generator.type_def_u128 = null;
    generator.type_def_s8 = null;
    generator.type_def_s16 = null;
    generator.type_def_s32 = null;
    generator.type_def_s64 = null;
    generator.type_def_s128 = null;
    generator.type_def_float32 = null;
    generator.type_def_float64 = null;
    generator.type_def_pointer_to_void = null;
    generator.type_def_ptr_void = null;
    generator.type_def_objc_class = null;
    generator.type_def_objc_selector = null;
    generator.type_def_objc_block_pointer = null;
    generator.type_def_objc_id = null;
    generator.type_def_jai_string = null;
    generator.type_def_type = null;
    generator.type_def_msvc_destructor_flags = null;
    generator.anon_declarations = .{};
    generator.macro_token_replacements = .{};
    generator.enum_macros = .{};
    generator.extern_declarations = .{};
    generator.unique_objc_selectors = .{};
    generator.com_uuids = .{};
    generator.pending_bitfields = .{};
    generator.structs_to_check = .{};
    generator.flattened_global_scope_members = .{};
    generator.declarations_for_global_scope = .{};
    generator.tokens_to_free = .{};
    generator.libraries = .{};
    generator.pointer_size = 0;
}
