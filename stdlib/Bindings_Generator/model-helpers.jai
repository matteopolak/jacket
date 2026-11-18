// Explicit-input operations on the canonical model.
#load "generator-state.jai";

create_elaborated_type :: (ty: CXType, cur: CXCursor) -> *CType {
    children := native_children(cur);
    defer array_free(children);
    result := gm_elaborated_type(ty, children, cur);
    return result;
}

create_elaborated_type :: (ty: CXType, children: [] CXCursor, location_cur: CXCursor) -> *CType, int {
    return gm_elaborated_type(ty, children, location_cur);
}

create_macro_definition :: (cur: CXCursor) -> *Declaration {
    return gm_macro_definition(cur);
}

create_objc_interface :: (cur: CXCursor, resolving_type_only := false) -> *Struct {
    return gm_objc_interface(cur, resolving_type_only);
}

create_objc_type_param :: (cur: CXCursor) -> *Declaration {
    return gm_objc_type_param(cur);
}

find_first_member_offset :: (outer_struct_type: CXType, outer_struct: *Struct, _struct: *Struct) -> int {
    path: [..] *Struct;
    defer array_free(path);
    return gm_first_member_offset(outer_struct_type, _struct, *path);
}

find_or_create_enum_for_macro :: (name: string, declaration: *Declaration) -> *Enum, is_new_enum: bool {
    return gm_macro_enum(name, declaration);
}

find_and_check_macro_identifier :: (name: string, _enum: *Enum = null, ignore_omit_flag := false) -> found: bool, output_name: string, is_of_type: *CType, parent_enum: *Enum {
    return gm_macro_identifier(name, _enum, ignore_omit_flag);
}

find_or_create_extra_info :: (type: *CType, base_struct: *Declaration) -> *Struct_Extra_Info, *Struct {
    return gm_extra_info(type, base_struct);
}

create_declaration :: (cur: CXCursor, parent: CXCursor, ignore_non_structs := false) -> *Declaration {
    result := create_declaration_inner(cur, parent, ignore_non_structs);
    if result {
        access := clang_getCXXAccessSpecifier(cur);
        if access == .CXXPrivate result.decl_flags |= .PRIVATE;
        if access == .CXXProtected result.decl_flags |= .PROTECTED;
    }

    return result;
}

create_declaration_inner :: (cur: CXCursor, parent: CXCursor, ignore_non_structs := false) -> *Declaration {
    if cur.kind == .MacroDefinition {
        if ignore_non_structs return null;
        return create_macro_definition(cur);
    }

    if cur.kind == .ObjCInterfaceDecl || cur.kind == .ObjCProtocolDecl || cur.kind == .ObjCCategoryDecl return cast(*Declaration) create_objc_interface(cur);
    if cur.kind == .TemplateTypeParameter return create_objc_type_param(cur);
    if cur.kind == .NonTypeTemplateParameter || cur.kind == .TemplateTemplateParameter return gm_template_formal(cur);
    if gm_cursor_kind(cur) == .TYPEDEF return cast(*Declaration) create_typedef(cur);

    if gm_cursor_kind(cur) == .FUNCTION {
        if ignore_non_structs return null;
        return cast(*Declaration) create_function(cur, parent);
    }

    return gm_build_declaration(cur, parent, ignore_non_structs);
}

create_type :: (ty: CXType, cur: CXCursor, cursor_is_accurate := true, loc := #caller_location, type_arg_index := 0, substitute_children: [] CXCursor = .[]) -> *CType {
    return gm_public_type(ty, cur, cursor_is_accurate, type_arg_index, substitute_children);
}

create_type_inner :: (ty: CXType, cur: CXCursor, cursor_is_accurate := true, loc := #caller_location, type_arg_index := 0, substitute_children: [] CXCursor = .[]) -> *CType {
    return gm_public_type(ty, cur, cursor_is_accurate, type_arg_index, substitute_children);
}

create_function_type :: (cur: CXCursor, ty: CXType, func: *Function = null, cursor_is_accurate := true, loc := #caller_location) -> *Function_Type {
    return gm_build_function_type(cur, ty, func, cursor_is_accurate, 0);
}

create_struct :: (cur: CXCursor, resolving_type_only := false) -> *Struct {
    return cast(*Struct) gm_build_expected(cur, .STRUCT, resolving_type_only);
}

create_enum :: (cur: CXCursor, resolving_type_only := false) -> *Enum {
    return cast(*Enum) gm_build_expected(cur, .ENUM, resolving_type_only);
}

create_typedef :: (cur: CXCursor) -> *Typedef {
    result := cast(*Typedef) gm_build_expected(cur, .TYPEDEF);
    if !result return null;

    generator := gm_active_generator();
    if !generator return null;

    unwrapped, existing := GM_Table.table_find(*generator.unwrapped_typedefs, result.name);
    if unwrapped && existing == result return null;

    return result;
}

create_namespace :: (cur: CXCursor, ignore_non_structs := false) -> *Namespace {
    declaration := gm_build_declaration(cur, clang_getCursorSemanticParent(cur), ignore_non_structs);
    if !declaration || declaration.kind != .NAMESPACE return null;
    return cast(*Namespace) declaration;
}

create_function :: (cur: CXCursor, parent: CXCursor) -> *Function {
    generator := gm_active_generator();
    if !generator return null;

    found, existing := GM_Table.table_find(*generator.declarations_by_cursor, clang_getCanonicalCursor(cur));
    if found {
        fill_location(*existing.location, cur);
        gm_fill_native_comment(*existing.comment, cur, existing.location);
        return null;
    }

    declaration := gm_build_declaration(cur, parent, false);
    if !declaration || declaration.kind != .FUNCTION return null;

    return cast(*Function) declaration;
}

create_variable_declaration :: (cur: CXCursor, parent: CXCursor, is_function_argument := false, parent_function: *Function = null) -> *Declaration, extra_parent_flags: Declaration_Flags {
    return gm_build_variable(cur, parent, is_function_argument, parent_function);
}

create_literal :: (cur: CXCursor, parent_cursor: CXCursor) -> *Literal {
    return gm_native_literal(cur, parent_cursor);
}

create_expression_from_cursor :: (cur: CXCursor, parent_cursor: CXCursor) -> *Expression, skip: bool {
    if clang_isExpression(cur.kind) == 0 return null, true;
    expression := gm_build_expression(cur, parent_cursor);
    return expression, false;
}

is_token :: (token: CXToken, expected: string) -> bool {
    return gm_is_token(token, expected);
}

count_asterisks :: (tokens: [] CXToken) -> int {
    return gm_count_asterisks(tokens);
}

find_closing_paren_token_index :: (tokens: [] CXToken, start_index: int) -> int {
    return gm_closing_paren(tokens, start_index);
}

macro_values_are_equal :: (a: Macro_Value, b: Macro_Value) -> bool {
    return gm_macro_values_equal(a, b);
}

macro_literals_are_equal :: (a: *Literal, b: *Literal) -> bool {
    if !a || !b || a.literal_kind != .MACRO || b.literal_kind != .MACRO return false;
    return gm_macro_values_equal(a.macro_value, b.macro_value);
}

can_omit_type :: (decl: *Declaration, flags: Print_Decl_Flags, type_to_print: *CType) -> bool {
    return (flags & .NO_ARGUMENT_DEFAULTS) == 0 && decl != null && decl.expression != null && is_bool_type(type_to_print);
}

create_literal :: (raw_tokens_str: string, location: Location = .{}) -> *Literal {
    session := gm_current_session();
    if !session return null;

    result := GM_Sessions.new_owned(session, Literal);
    result.literal_kind = .RAW_CPP_TOKENS;
    result.string_value = GM_Sessions.copy_text(session, raw_tokens_str);
    result.raw_value = GM_Sessions.copy_text(session, raw_tokens_str);
    result.location = location;

    return result;
}

create_literal :: (macro_value: Macro_Value, location: Location) -> *Literal {
    return gm_macro_literal(macro_value, location);
}

find_referenced_declaration :: (cur: CXCursor) -> *Declaration {
    return gm_referenced_declaration(cur);
}

create_reference_expression :: (cur: CXCursor, location: Location) -> *Literal {
    return gm_reference_expression(cur, location);
}

create_omit_cpp_functions_visitor :: (visitor: Declaration_Visitor, visitor_with_data: Declaration_Visitor_With_Data, visitor_data: *void) -> Declaration_Visitor_With_Data, *void {
    return gm_omit_visitor(visitor, visitor_with_data, visitor_data);
}

find_underlying_type :: (type: *CType) -> *CType {
    return gm_underlying(type);
}

is_pointer_type :: (ty: *CType) -> bool {
    t := gm_underlying(ty);
    return t != null && t.pointer_to != null;
}

is_function_type :: (ty: *CType) -> bool {
    t := gm_underlying(ty);
    return t != null && t.type_of_function != null;
}

is_incomplete_array :: (type: *CType) -> bool {
    t := gm_underlying(type);
    return t != null && t.array_element_type != null && t.array_element_count < 0;
}

is_target_type_signed :: (target_type: *CType = null) -> bool {
    t := gm_integer(target_type);
    return t != null && (t.number_flags & .SIGNED) != 0;
}

is_target_type_unsigned :: (target_type: *CType = null) -> bool {
    t := gm_integer(target_type);
    return t != null && (t.number_flags & .SIGNED) == 0;
}

is_16byte_integer_type :: (ty: *CType) -> bool {
    t := gm_integer(ty);
    return t != null && t.size == 16;
}

is_zero_int_expression :: (expression: *Expression) -> bool {
    if !expression || expression.kind != .LITERAL return false;

    lit := cast(*Literal) expression;
    if lit.literal_kind == .INTEGER return lit.int_value == 0;
    if lit.literal_kind == .UNSIGNED_INTEGER return lit.uint_value == 0;

    return false;
}

is_char_pointer_type :: (type: CType) -> bool {
    t := gm_underlying(*type);
    if !t || !t.pointer_to return false;
    child := gm_integer(t.pointer_to);
    return child != null && child.size == 1 && (child.number_flags & .CHAR) != 0;
}

is_extern_declaration :: (decl: *Declaration) -> bool {
    return decl != null && decl.storage_class == .Extern;
}

is_valid_identifier_start :: (char: u8) -> bool {
    return char == #char "_" || (char >= #char "a" && char <= #char "z") || (char >= #char "A" && char <= #char "Z");
}

is_operator_name :: (name: string) -> bool {
    return gm_operator(name);
}

is_assignment_operator :: (name: string) -> bool {
    return gm_assignment(name);
}

is_operator :: (decl: *Declaration) -> bool {
    return decl != null && decl.kind == .FUNCTION && is_operator_name(decl.name);
}

is_targeting_apple_os :: () -> bool {
    return TARGET_OS == .MACOS || TARGET_OS == .IOS;
}

is_prefix_comment :: (comment: Comment) -> bool {
    return gm_prefix_comment(comment);
}

is_va_list :: (type: *CType) -> bool {
    return gm_va_list(type);
}

is_struct_field :: (decl: *Declaration) -> is_field: bool, anon_struct: *Struct {
    return gm_struct_field(decl);
}

get_type_alignment :: (type: *CType, decl: *Declaration) -> alignment: s64, is_natural_alignment: bool {
    return gm_alignment(type, decl);
}

get_primitive_or_array_primitive_type_size :: (type: *CType, decl: *Declaration) -> int {
    return gm_primitive_size(type, decl);
}

functions_are_equivalent :: (a: *Function, b: *Function, ignore_first_argument := false) -> bool {
    return gm_functions_equal(a, b, ignore_first_argument);
}

contains_equivalent_function :: (declarations: [] *Declaration, fn: *Function) -> bool {
    for decl: declarations if decl && decl.kind == .FUNCTION && functions_are_equivalent(cast(*Function) decl, fn) return true;
    return false;
}

find_name_clash :: (existing_declarations: [] *Declaration, decl: *Declaration, allow_overloads: bool) -> found: bool, other: *Declaration {
    return gm_name_clash(existing_declarations, decl, allow_overloads);
}

_create_declaration :: (name: string, type: *CType, location: Location = .{}, comment: Comment = .{}) -> *Declaration {
    return gm_declaration(name, type, location, comment);
}

_create_declaration :: (name: string, type: *CType, cur: CXCursor) -> *Declaration {
    location: Location;

    fill_location(*location, cur);
    native := clang_Cursor_getRawCommentText(cur);

    defer clang_disposeString(native);
    comment: Comment;
    comment.location = gm_comment_source_location(cur, location);
    comment.text = trim(to_string(clang_getCString(native)));

    return gm_declaration(name, type, location, comment);
}

create_enum :: (name: string) -> *Enum {
    return gm_enum(name);
}

create_empty_struct_type :: (name: string) -> *CType {
    return gm_empty_struct(name);
}

count_namespace_refs :: (children: [] CXCursor) -> int {
    count := 0;
    for child: children if child.kind == .NamespaceRef count += 1;
    return count;
}

#scope_file
#import "Basic";
#load "../internal/developer-tools/generator-model-helpers.jai";
#load "../internal/developer-tools/generator-session-access.jai";
#load "../internal/developer-tools/generator-token-helpers.jai";
#load "../internal/developer-tools/generator-reference-helpers.jai";
#load "../internal/developer-tools/generator-canonical-construction.jai";
#load "../internal/developer-tools/generator-model-lookup.jai";
#load "../internal/developer-tools/generator-special-construction.jai";
#load "../internal/developer-tools/generator-aggregate-construction.jai";
#load "../internal/developer-tools/generator-template-construction.jai";
#load "../internal/developer-tools/generator-template-defaults.jai";
#load "../internal/developer-tools/generator-native-string.jai";

#scope_export
init_generator :: (using generator: *Generator, options: Generate_Bindings_Options) {
    gm_init_generator(generator, options);
}

deinit_generator :: (generator: *Generator) {
    gm_deinit_generator(generator);
}

#scope_module
#load "../internal/developer-tools/generator-lifecycle.jai";
#load "../internal/developer-tools/generator-macro-evaluation.jai";
#load "../internal/developer-tools/generator-array-construction.jai";
#load "../internal/developer-tools/generator-construction-state.jai";
