#import "Basic";
B :: #import "Bindings_Generator";
H :: #import "Hash_Table";

S :: type_of(context._bindings_generator_session.*);

Capture :: struct {
    saved: #Context;
    children: [..] B.CXCursor;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        array_add(*capture.children, cur);
    }

    return .Continue;
}

diagnostics: int;
logger_cursor, logger_parent: B.CXCursor;
logger_declaration: *B.Declaration;
reenter_log: bool;

log :: (_text: string, _data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 diagnostics += 1;

    if reenter_log {
        reenter_log = false;
        logger_declaration = B.create_declaration(logger_cursor, logger_parent);
    }
}

main :: () {
    push_context, defer_pop;
    context.logger = log;
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    session := context._bindings_generator_session;

    S.activate(*generator, *options, session);
    generator.clang_index = B.clang_createIndex(0, 0);
    path := tprint("%generator-model-boundaries.hpp", #filepath);
    filename := to_c_string(path);

    defer free(filename);
    arguments := (*u8).[to_c_string("-std=c++17"), to_c_string("-x"), to_c_string("objective-c++")];

    defer for argument: arguments free(argument);
    assert(B.clang_parseTranslationUnit2(generator.clang_index, filename, arguments.data, 3, null, 0, cast(u32) B.CXTranslationUnit_Flags.DetailedPreprocessingRecord, *generator.translation_unit) == .Success);
    assert(B.clang_getNumDiagnostics(generator.translation_unit) == 0);
    receipt: type_of(session.parse_input.*);
    receipt.filename = path;
    receipt.arguments = string.["-std=c++17", "-x", "objective-c++"];
    session.parse_input = *receipt;
    root := B.clang_getTranslationUnitCursor(generator.translation_unit);
    capture: Capture;
    capture.saved = context;
    B.clang_visitChildren(root, visit, *capture);
    defer array_free(capture.children);
    rejected, accepted, filtered, macro, dependent, container, invalid_cycle, dependent_cycle, unsupported_field: B.CXCursor;
    enum_owner, enum_default, objc_root, objc_child: B.CXCursor;
    bits_cursor, poly_bits_cursor: B.CXCursor;

    for child: capture.children {
        name := B.clang_getCursorSpelling(child);
        text := to_string(B.clang_getCString(name));

        if text == "rejected_value" rejected = child;
        if text == "AcceptedAfter" accepted = child;
        if text == "LoggerAccepted" logger_cursor = child;
        if text == "FilteredBoundary" filtered = child;
        if text == "MODEL_BOUNDARY_VALUE" macro = child;
        if text == "MODEL_BOUNDARY_DEPENDENT" dependent = child;
        if text == "BoundaryContainer" container = child;
        if text == "InvalidCycle" invalid_cycle = child;
        if text == "DependentCycle" dependent_cycle = child;
        if text == "BoundaryUnsupportedField" unsupported_field = child;
        if text == "BoundaryEnumOwner" enum_owner = child;
        if text == "boundary_enum_default" enum_default = child;
        if text == "BoundaryObjRoot" objc_root = child;
        if text == "BoundaryObjChild" objc_child = child;
        if text == "BoundaryBits" bits_cursor = child;
        if text == "BoundaryPolyBits" poly_bits_cursor = child;

        B.clang_disposeString(name);
    }

    assert(B.create_declaration(rejected, root) == null && diagnostics > 0);
    assert(!session.failed);
    assert(!H.table_contains(*generator.declarations_by_cursor, B.clang_getCanonicalCursor(rejected)));
    before := diagnostics;

    assert(B.create_declaration(rejected, root) == null && diagnostics > before);
    after := B.create_declaration(accepted, root);

    assert(after && after.type == generator.type_def_s32);
    assert(B.create_declaration(accepted, root) == after);
    enum_children: Capture;
    enum_children.saved = context;
    B.clang_visitChildren(enum_owner, visit, *enum_children);
    defer array_free(enum_children.children);
    assert(B.create_enum(enum_children.children[0]) != null);
    default_function := B.create_function(enum_default, root);

    assert(default_function && (cast(*B.Literal) default_function.type.type_of_function.arguments[0].expression).literal_kind == .ENUM);
    assert(B.create_objc_interface(objc_root) != null && B.create_objc_interface(objc_child) != null);
    assert(B.create_struct(enum_owner) == null && !session.failed);
    assert(!H.table_contains(*generator.declarations_by_cursor, B.clang_getCanonicalCursor(enum_default)));
    assert(!H.table_contains(*generator.declarations_by_cursor, B.clang_getCanonicalCursor(objc_root)));
    assert(!H.table_contains(*generator.declarations_by_cursor, B.clang_getCanonicalCursor(objc_child)));
    unsupported_record := B.create_struct(unsupported_field);

    assert(unsupported_record == null && !session.failed);
    assert(!H.table_contains(*generator.declarations_by_cursor, B.clang_getCanonicalCursor(unsupported_field)));
    logger_parent = root;
    dependent_record := B.create_struct(dependent_cycle);

    assert(dependent_record != null);
    assert(B.find_or_create_extra_info(dependent_record.type, cast(*B.Declaration) dependent_record) != null);
    S.append_owned(session, *generator.global_scope.members, after);
    S.append_owned(session, *generator.global_scope.members, cast(*B.Declaration) dependent_record);
    generator.flattened_global_scope_members = S.copy_array(session, generator.global_scope.members);
    reenter_log = true;
    assert(B.create_struct(invalid_cycle) == null && !session.failed);
    assert(logger_declaration && B.create_declaration(logger_cursor, root) == logger_declaration);
    assert(!H.table_contains(*generator.declarations_by_cursor, B.clang_getCanonicalCursor(invalid_cycle)));
    assert(!H.table_contains(*generator.declarations_by_cursor, B.clang_getCanonicalCursor(dependent_cycle)));
    assert(!H.table_contains(*generator.extra_infos, dependent_record));
    assert(B.find_or_create_extra_info(dependent_record.type, cast(*B.Declaration) dependent_record) == null);
    assert(generator.global_scope.members.count == 1 && generator.global_scope.members[0] == after);
    assert(generator.flattened_global_scope_members.count == 1 && generator.flattened_global_scope_members[0] == after);
    assert(B.create_declaration(accepted, root) == after);
    namespace := cast(*B.Namespace) B.create_declaration(filtered, root, true);

    assert(namespace && namespace.declarations.count == 2);
    record := cast(*B.Struct) namespace.declarations[0];
    enumeration := cast(*B.Enum) namespace.declarations[1];

    assert(record.name == "Retained" && record.declarations.count == 1 && record.declarations[0].name == "member");
    assert(enumeration.name == "RetainedEnum" && enumeration.enumerates.count == 1);
    full := B.create_declaration(filtered, root);

    assert(full == cast(*B.Declaration) namespace && namespace.declarations.count == 4);
    assert(namespace.declarations[0] == cast(*B.Declaration) record && record.declarations.count == 1);
    composite := B.create_struct(container);

    assert(composite && composite.declarations.count == 3);
    nested := cast(*B.Struct) composite.declarations[0];

    assert(nested.kind == .STRUCT && nested.name == "" && nested.output_name == "");
    assert(composite.declarations[1].name == "inner" && composite.declarations[1].type.type_of_struct == nested);
    assert(nested.parent == cast(*B.Declaration) composite && nested.declarations[0].name == "payload");
    bits := B.create_struct(bits_cursor);

    assert(bits && bits.declarations.count == 2 && bits.type.size == 4 && bits.type.alignment == 4);
    status: B.Bitfield_Status;
    status.parent = bits;
    group := cast(*B.Bitfield) B.maybe_add_to_bitfield(*status, bits.declarations[0]);

    assert(group && B.maybe_add_to_bitfield(*status, bits.declarations[1]) == null);
    B.finish_bitfield(group, 0);
    bits.declarations.count = 0;
    S.append_owned(session, *bits.declarations, cast(*B.Declaration) group);
    bits_info := B.find_or_create_extra_info(bits.type, cast(*B.Declaration) bits);

    assert(bits_info && bits_info.modified_declarations.count == 1 && !bits_info.padding_declaration);
    assert(bits_info.has_field && bits_info.has_nonzero_sized_field && bits_info.largest_natural_alignment == 4);
    poly_bits := B.create_struct(poly_bits_cursor);

    assert(poly_bits && poly_bits.declarations.count == 3 && poly_bits.virtual_functions.count == 1);
    poly_status: B.Bitfield_Status;
    poly_status.parent = poly_bits;
    poly_group := cast(*B.Bitfield) B.maybe_add_to_bitfield(*poly_status, poly_bits.declarations[1]);

    assert(poly_group != null);
    B.finish_bitfield(poly_group, 0);
    poly_bits.declarations[1] = cast(*B.Declaration) poly_group;
    prior_errors := diagnostics;

    assert(B.find_or_create_extra_info(poly_bits.type, cast(*B.Declaration) poly_bits) == null);
    assert(diagnostics > prior_errors && !session.failed && !H.table_contains(*generator.extra_infos, poly_bits));
    value := B.create_macro_definition(macro);

    S.append_owned(session, *generator.global_scope.members, value);
    dependent_value := B.create_macro_definition(dependent);

    S.append_owned(session, *generator.global_scope.members, dependent_value);
    found, _, _, _ := B.find_and_check_macro_identifier("MODEL_BOUNDARY_VALUE");

    assert(found && value.macro_value_result == .KEEP && !session.failed);
    dependency_text := B.string_from_macro_value((cast(*B.Literal) dependent_value.expression).macro_value, name = dependent_value.name);

    defer free(dependency_text);
    assert(dependency_text == "MODEL_BOUNDARY_VALUE + 1");
    value.should_unwrap = true;
    value.decl_flags |= .OMIT_FROM_OUTPUT;
    hidden, _, _, _ := B.find_and_check_macro_identifier("MODEL_BOUNDARY_VALUE");

    assert(!hidden);
    unwrapped, text, type, _ := B.find_and_check_macro_identifier("MODEL_BOUNDARY_VALUE", ignore_omit_flag = true);

    assert(unwrapped && text == "17" && type == value.type);
    again, second, _, _ := B.find_and_check_macro_identifier("MODEL_BOUNDARY_VALUE", ignore_omit_flag = true);

    assert(again && second == text && second.data != text.data);
    substituted_text := B.string_from_macro_value((cast(*B.Literal) dependent_value.expression).macro_value, name = dependent_value.name);

    defer free(substituted_text);
    assert(substituted_text == "17 + 1" && dependency_text == "MODEL_BOUNDARY_VALUE + 1");
    // Fatal malformed input remains sticky and prevents native macro validation.
    malformed: [] B.CXCursor;
    malformed.count = 1;
    assert(B.create_type_inner(B.clang_getCursorType(accepted), accepted, false, substitute_children = malformed) == null);
    assert(session.failed);
    value.macro_value_result = .NEEDS_CHECKING;
    found, _, _, _ = B.find_and_check_macro_identifier("MODEL_BOUNDARY_VALUE", ignore_omit_flag = true);
    assert(!found && value.macro_value_result == .NEEDS_CHECKING);
    print("ok\n");
}
