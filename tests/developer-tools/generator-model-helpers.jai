#import "Basic";
B :: #import "Bindings_Generator";
BW :: #import "Bindings_Generator"(TARGET_OS = .WINDOWS);
BI :: #import "Bindings_Generator"(TARGET_OS = .IOS);
BL :: #import "Bindings_Generator"(TARGET_OS = .LINUX);

errors: int;

capture :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

visited: int;

plain_visit :: (_decl: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    visited += 1;
    return .RECURSE;
}

data_visit :: (_decl: *B.Declaration, _parent: *B.Declaration, data: *void) -> B.Declaration_Visit_Result {
    (cast(*int) data).* += 1;
    return .RECURSE;
}

main :: () {
    generator: B.Generator;
    symbols: B.Library_Symbols;
    token_info: B.Token_Info;

    assert(generator.pointer_size == 0 && generator.translation_unit == null);
    assert(generator.tab_depth == 0 && generator.template_depth == 0);
    assert(generator.counts.functions == 0 && generator.declarations_by_cursor.count == 0);
    assert(generator.extra_infos.count == 0 && generator.tokens_to_free.count == 0);
    assert(symbols.symbol_names.count == 0 && token_info.num_tokens == 0);
    #assert type_of(generator.macros_converted_to_enum_values) == H.Table(string, *B.Enum);
    #assert type_of(generator.extra_infos) == H.Table(*B.Struct, B.Struct_Extra_Info);
    #assert type_of(generator.filenames) == H.Table(B.CXFile, string);
    #assert type_of(generator.macro_token_replacements[0]) == B.Generator.Replacement;
    integer := B.get_int_type(4, true);

    defer free(integer);
    alias: B.Typedef;
    alias.type = integer;
    wrapped: B.CType;
    wrapped.type_of_typedef = *alias;
    assert(B.find_underlying_type(*wrapped) == integer);
    assert(B.is_target_type_signed(*wrapped));
    assert(!B.is_target_type_unsigned(*wrapped));
    pointer := B.create_pointer_to(*wrapped);

    defer free(pointer);
    assert(B.is_pointer_type(pointer));
    assert(!B.is_target_type_signed(pointer));
    assert(!B.is_function_type(pointer));
    array: B.CType;
    array.array_element_type = integer;
    assert(B.is_incomplete_array(*array));
    array.array_element_count = 3;
    assert(!B.is_incomplete_array(*array));
    decl: B.Declaration;

    assert(B.get_primitive_or_array_primitive_type_size(*array, *decl) == 12);
    wide := integer.*;
    wide.size = 16;
    assert(B.is_16byte_integer_type(*wide));
    wide.number_flags |= .FLOAT;
    assert(!B.is_16byte_integer_type(*wide));
    zero: B.Literal;
    zero.literal_kind = .INTEGER;
    assert(B.is_zero_int_expression(cast(*B.Expression) *zero));
    zero.int_value = 1;
    assert(!B.is_zero_int_expression(cast(*B.Expression) *zero));
    created := B._create_declaration("owned", integer);

    defer free(created);
    defer free(created.name);
    assert(created.kind == .DECLARATION && created.type == integer);
    assert(created.output_name == "owned");
    assert(B.is_valid_identifier_start(#char "_"));
    assert(!B.is_valid_identifier_start(#char "9"));
    assert(B.is_operator_name("operator+"));
    assert(!B.is_operator_name("operator_name"));
    assert(B.is_assignment_operator("operator<<="));
    assert(!B.is_assignment_operator("operator=="));
    first, second: B.Function;
    first.kind = .FUNCTION;
    second.kind = .FUNCTION;
    first.name = "call";
    second.name = "call";
    ft, other_ft: B.Function_Type;
    ft.return_type = integer;
    other_ft.return_type = integer;
    t, other_t: B.CType;
    t.type_of_function = *ft;
    other_t.type_of_function = *other_ft;
    first.type = *t;
    second.type = *other_t;
    assert(B.functions_are_equivalent(*first, *second));
    declarations: [1] *B.Declaration;
    declarations[0] = cast(*B.Declaration) *first;
    assert(B.contains_equivalent_function(declarations, *second));
    found, other := B.find_name_clash(declarations, cast(*B.Declaration) *second, true);

    assert(found && other == cast(*B.Declaration) *first);
    other_ft.flags |= .VARIADIC;
    assert(!B.functions_are_equivalent(*first, *second));
    found, other = B.find_name_clash(declarations, cast(*B.Declaration) *second, true);
    assert(!found && !other);
    assert(B.is_targeting_apple_os());
    assert(BI.is_targeting_apple_os());
    assert(!BW.is_targeting_apple_os() && !BL.is_targeting_apple_os());
    assert(B.find_underlying_type(null) == null);
    assert(!B.is_pointer_type(null) && !B.is_function_type(null));
    assert(!B.is_target_type_signed() && !B.is_target_type_unsigned());
    unsigned := integer.*;
    unsigned.number_flags &= ~B.Number_Flags.SIGNED;
    assert(B.is_target_type_unsigned(*unsigned));
    character := integer.*;
    character.size = 1;
    character.number_flags |= .CHAR;
    char_pointer: B.CType;
    char_pointer.pointer_to = *character;
    assert(B.is_char_pointer_type(char_pointer));
    assert(!B.is_char_pointer_type(character));
    decl.storage_class = .Extern;
    assert(B.is_extern_declaration(*decl));
    decl.storage_class = .Static;
    assert(!B.is_extern_declaration(*decl));
    first.name = "operator+";
    assert(B.is_operator(cast(*B.Declaration) *first));
    assert(!B.is_operator(null));
    comment: B.Comment;
    comment.text = "/// before";
    assert(!B.is_prefix_comment(comment));
    comment.location.line = 3;
    assert(B.is_prefix_comment(comment));
    comment.text = "///< after";
    assert(B.is_prefix_comment(comment));
    comment.location.line = 0;
    assert(!B.is_prefix_comment(comment));
    comment.text = "one\ntwo";
    assert(B.is_prefix_comment(comment));
    comment.text = "";
    assert(!B.is_prefix_comment(comment));
    alias.name = "va_list";
    assert(B.is_va_list(*wrapped));
    assert(!B.is_va_list(integer));
    parent: B.Struct;
    parent.kind = .STRUCT;
    decl.kind = .DECLARATION;
    decl.parent = cast(*B.Declaration) *parent;
    decl.storage_class = .None;
    field, anon := B.is_struct_field(*decl);

    assert(field && !anon);
    parent.flags |= .IS_ANON;
    field, anon = B.is_struct_field(*decl);
    assert(field && anon == *parent);
    decl.decl_flags |= .IS_METADATA;
    field, anon = B.is_struct_field(*decl);
    assert(!field && !anon);
    alignment, natural := B.get_type_alignment(integer, *decl);

    assert(alignment == 4 && natural);
    decl.explicit_field_alignment = 16;
    alignment, natural = B.get_type_alignment(integer, *decl);
    assert(alignment == 16 && !natural);
    named_enum := B.create_enum("E");

    defer free(named_enum);
    defer free(named_enum.name);
    assert(named_enum.kind == .ENUM && named_enum.output_name == "E");
    empty := B.create_empty_struct_type("Opaque");

    defer free(empty);
    defer free(empty.type_of_struct);
    defer free(empty.type_of_struct.name);
    assert(empty.type_of_struct.kind == .STRUCT && empty.type_of_struct.name == "Opaque");
    children: [3] B.CXCursor;
    children[0].kind = .NamespaceRef;
    children[1].kind = .TypeRef;
    children[2].kind = .NamespaceRef;
    assert(B.count_namespace_refs(children) == 2);
    comment.text = copy_string("owned comment");
    mutable_name := copy_string("mutable");
    owned := B._create_declaration(mutable_name, integer, .{}, comment);

    mutable_name.data[0] = #char "X";
    comment.text.data[0] = #char "X";
    assert(owned.name == "mutable" && owned.comment.text == "owned comment");
    free(mutable_name);
    free(comment.text);
    free(owned.name);
    free(owned.comment.text);
    free(owned);
    // Every malformed graph produces an explicit diagnostic and terminates.
    alias.name = "cycle";
    alias.type = *wrapped;

    push_context {
        context.logger = capture;
        assert(!B.find_underlying_type(*wrapped));
        assert(!B.is_va_list(*wrapped));
        array.array_element_count = 0x7fffffffffffffff;
        assert(B.get_primitive_or_array_primitive_type_size(*array, *decl) == -1);
    }

    assert(errors == 3);
    recursive_enum: B.Enum;
    enum_type: B.CType;
    enum_type.type_of_enum = *recursive_enum;
    recursive_enum.type = *enum_type;

    push_context {
        context.logger = capture;
        assert(!B.is_target_type_signed(*enum_type));
    }

    assert(errors == 4);
    wrapper, wrapper_data := B.create_omit_cpp_functions_visitor(plain_visit, null, null);

    defer free(wrapper_data);
    ft.flags = .CPP_METHOD;
    assert(wrapper(cast(*B.Declaration) *first, null, wrapper_data) == .STOP);
    assert((first.decl_flags & .OMIT_FROM_OUTPUT) != 0 && visited == 0);
    ft.flags = 0;
    assert(wrapper(cast(*B.Declaration) *first, null, wrapper_data) == .RECURSE);
    assert(visited == 1);
    data_count: int;
    with_data, data_closure := B.create_omit_cpp_functions_visitor(null, data_visit, *data_count);

    defer free(data_closure);
    assert(with_data(*decl, null, data_closure) == .RECURSE && data_count == 1);
    no_callback, no_data := B.create_omit_cpp_functions_visitor(null, null, null);

    defer free(no_data);
    assert(no_callback(*decl, null, no_data) == .RECURSE);
    both, both_data := B.create_omit_cpp_functions_visitor(plain_visit, data_visit, *data_count);

    defer free(both_data);
    assert(both(*decl, null, both_data) == .RECURSE && visited == 2 && data_count == 2);
    left_arg, right_arg: B.Declaration;
    left_arg.type = integer;
    right_arg.type = *unsigned;
    array_add(*ft.arguments, *left_arg);
    array_add(*other_ft.arguments, *right_arg);
    defer array_free(ft.arguments);
    defer array_free(other_ft.arguments);
    other_ft.flags = 0;
    assert(!B.functions_are_equivalent(*first, *second));
    assert(B.functions_are_equivalent(*first, *second, true));
    invalid_ft: B.Function_Type;
    invalid_ft.return_type = integer;
    invalid_ft.arguments.count = -1;
    invalid_type: B.CType;
    invalid_type.type_of_function = *invalid_ft;
    malformed: B.Function;
    malformed.kind = .FUNCTION;
    malformed.type = *invalid_type;

    push_context {
        context.logger = capture;
        assert(!B.functions_are_equivalent(*malformed, *malformed));
    }

    assert(errors == 5);
    first.output_name = "same";
    second.output_name = "same";
    other_array := array;

    left_arg.type = *array;
    right_arg.type = *other_array;

    push_context {
        context.logger = capture;
        found, other = B.find_name_clash(declarations, cast(*B.Declaration) *second, true);
        assert(found && other == cast(*B.Declaration) *first);
    }

    assert(errors == 6);
    missing_return: B.Function_Type;
    missing_return_type: B.CType;
    missing_return_type.type_of_function = *missing_return;
    incomplete: B.Function;
    incomplete.type = *missing_return_type;

    push_context {
        context.logger = capture;
        assert(!B.functions_are_equivalent(*incomplete, *incomplete));
    }

    assert(errors == 7);
    primary: B.Struct;
    bad_template: B.CType;
    bad_template.type_of_struct = *primary;
    bad_template.template_instantiation_params.count = 1;
    left_arg.type = *bad_template;
    right_arg.type = *bad_template;

    push_context {
        context.logger = capture;
        assert(!B.functions_are_equivalent(*first, *second));
    }

    assert(errors == 8);
    cycle_left, cycle_right: B.CType;
    cycle_left.pointer_to = *cycle_left;
    cycle_right.pointer_to = *cycle_right;
    left_arg.type = *cycle_left;
    right_arg.type = *cycle_right;

    push_context {
        context.logger = capture;
        assert(!B.functions_are_equivalent(*first, *second));
    }

    assert(errors == 9);
    unsigned_wide: B.CType;
    unsigned_wide.size = 16;
    qualified_wide := unsigned_wide;
    qualified_wide.flags = .CONST;
    left_arg.type = *unsigned_wide;
    right_arg.type = *qualified_wide;
    assert(B.functions_are_equivalent(*first, *second));
    print("ok\n");
}

#scope_file
H :: #import "Hash_Table";
