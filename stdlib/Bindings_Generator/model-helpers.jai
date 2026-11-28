// Explicit-input operations on the canonical model.
#load "generator-state.jai";

// Builds the type for an elaborated type `ty`, reading the children of `cur` to resolve names and template
// arguments.
create_elaborated_type :: (ty: CXType, cur: CXCursor) -> *CType {
    children := native_children(cur);
    defer array_free(children);
    result := gm_elaborated_type(ty, children, cur);
    return result;
}

// Builds the type for an elaborated type `ty` from `children`, which are the cursor's children.
//
// Returns the type, or null when it cannot be resolved, and the number of leading children it consumed.
// `location_cur` gives the source position.
create_elaborated_type :: (ty: CXType, children: [] CXCursor, location_cur: CXCursor) -> *CType, int {
    return gm_elaborated_type(ty, children, location_cur);
}

// Builds the macro declaration for `cur`; returns null for built-in or function-like macros, ignored names
// and include guards.
create_macro_definition :: (cur: CXCursor) -> *Declaration {
    return gm_macro_definition(cur);
}

// Builds the struct for an Objective-C interface, protocol or category cursor. With `resolving_type_only`
// only the type is resolved.
create_objc_interface :: (cur: CXCursor, resolving_type_only := false) -> *Struct {
    return gm_objc_interface(cur, resolving_type_only);
}

// Builds the declaration for an Objective-C type parameter cursor.
create_objc_type_param :: (cur: CXCursor) -> *Declaration {
    return gm_objc_type_param(cur);
}

// Returns the offset in bytes of the first field of `_struct` within `outer_struct_type`, or -1 if none is
// found.
//
// Anonymous nested structs are searched in order. `outer_struct` is not used.
find_first_member_offset :: (outer_struct_type: CXType, outer_struct: *Struct, _struct: *Struct) -> int {
    path: [..] *Struct;
    defer array_free(path);
    return gm_first_member_offset(outer_struct_type, _struct, *path);
}

// Returns the enum that collects macros named `name`, creating it when none exists.
//
// The second result is true when the enum was created. `declaration` supplies its location and comment.
find_or_create_enum_for_macro :: (name: string, declaration: *Declaration) -> *Enum, is_new_enum: bool {
    return gm_macro_enum(name, declaration);
}

// Looks up `name` among the identifiers a macro value can refer to.
//
// Returns whether it was found, the name to print, its type, and the enum it belongs to. Searches `_enum`
// first, then enums built from macros, unwrapped typedefs and global declarations. Declarations marked
// `OMIT_FROM_OUTPUT` are skipped unless `ignore_omit_flag` is set.
find_and_check_macro_identifier :: (name: string, _enum: *Enum = null, ignore_omit_flag := false) -> found: bool, output_name: string, is_of_type: *CType, parent_enum: *Enum {
    return gm_macro_identifier(name, _enum, ignore_omit_flag);
}

// Returns the [Struct_Extra_Info] for the struct that `type` names, and that struct.
//
// Returns a shared unknown placeholder and a null struct when the struct is a template, still incomplete, has
// an unknown layout or is an Objective-C type. Returns null for both on error.
find_or_create_extra_info :: (type: *CType, base_struct: *Declaration) -> *Struct_Extra_Info, *Struct {
    return gm_extra_info(type, base_struct);
}

// Builds the declaration for `cur` of the right kind, or returns null when nothing is produced.
//
// Sets `PRIVATE` or `PROTECTED` from the C++ access specifier. With `ignore_non_structs`, macros and
// functions
// are skipped. `parent` is the cursor's parent.
create_declaration :: (cur: CXCursor, parent: CXCursor, ignore_non_structs := false) -> *Declaration {
    result := create_declaration_inner(cur, parent, ignore_non_structs);
    if result {
        access := clang_getCXXAccessSpecifier(cur);
        if access == .CXXPrivate result.decl_flags |= .PRIVATE;
        if access == .CXXProtected result.decl_flags |= .PROTECTED;
    }

    return result;
}

// Builds the declaration for `cur` without applying access flags; see [create_declaration].
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

// Builds the [CType] for `ty`, with `cur` as the source cursor. `cursor_is_accurate` says whether `cur`
// describes `ty` itself. `type_arg_index` and `substitute_children` select a template argument and the cursor
// children to use for it. `loc` is accepted for compatibility and not used.
create_type :: (ty: CXType, cur: CXCursor, cursor_is_accurate := true, loc := #caller_location, type_arg_index := 0, substitute_children: [] CXCursor = .[]) -> *CType {
    return gm_public_type(ty, cur, cursor_is_accurate, type_arg_index, substitute_children);
}

// The same procedure as [create_type].
create_type_inner :: create_type;

// Builds the [Function_Type] for the function or function-type cursor `cur`; `func`, when given, is the
// function it belongs to. `loc` is accepted for compatibility and not used.
create_function_type :: (cur: CXCursor, ty: CXType, func: *Function = null, cursor_is_accurate := true, loc := #caller_location) -> *Function_Type {
    return gm_build_function_type(cur, ty, func, cursor_is_accurate, 0);
}

// Builds the [Struct] for `cur`. With `resolving_type_only` only the type is resolved.
create_struct :: (cur: CXCursor, resolving_type_only := false) -> *Struct {
    return cast(*Struct) gm_build_expected(cur, .STRUCT, resolving_type_only);
}

// Builds the [Enum] for `cur`. With `resolving_type_only` only the type is resolved.
create_enum :: (cur: CXCursor, resolving_type_only := false) -> *Enum {
    return cast(*Enum) gm_build_expected(cur, .ENUM, resolving_type_only);
}

// Builds the [Typedef] for `cur`.
//
// Returns null when the typedef was already created or is one the generator unwraps into its target.
create_typedef :: (cur: CXCursor) -> *Typedef {
    result := cast(*Typedef) gm_build_expected(cur, .TYPEDEF);
    if !result return null;

    generator := gm_active_generator();
    if !generator return null;

    unwrapped, existing := GM_Table.table_find(*generator.unwrapped_typedefs, result.name);
    if unwrapped && existing == result return null;

    return result;
}

// Builds the [Namespace] for `cur`; returns null if `cur` is not a namespace.
create_namespace :: (cur: CXCursor, ignore_non_structs := false) -> *Namespace {
    declaration := gm_build_declaration(cur, clang_getCursorSemanticParent(cur), ignore_non_structs);
    if !declaration || declaration.kind != .NAMESPACE return null;
    return cast(*Namespace) declaration;
}

// Builds the [Function] for `cur`.
//
// Returns null if the function was already created (its location and comment are refreshed) or `cur` is not a
// function.
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

// Builds the variable or parameter declaration for `cur`.
//
// The second result holds flags the parent declaration should take on. `parent_function` is the function
// that owns the argument when `is_function_argument` is set.
create_variable_declaration :: (cur: CXCursor, parent: CXCursor, is_function_argument := false, parent_function: *Function = null) -> *Declaration, extra_parent_flags: Declaration_Flags {
    return gm_build_variable(cur, parent, is_function_argument, parent_function);
}

// Builds the literal expression for the expression cursor `cur`.
create_literal :: (cur: CXCursor, parent_cursor: CXCursor) -> *Literal {
    return gm_native_literal(cur, parent_cursor);
}

// Builds the expression for `cur`.
//
// The second result is true, with a null expression, when `cur` is not an expression and should be skipped.
create_expression_from_cursor :: (cur: CXCursor, parent_cursor: CXCursor) -> *Expression, skip: bool {
    if clang_isExpression(cur.kind) == 0 return null, true;
    expression := gm_build_expression(cur, parent_cursor);
    return expression, false;
}

// Returns whether the spelling of `token` is `expected`.
is_token :: (token: CXToken, expected: string) -> bool {
    return gm_is_token(token, expected);
}

// Returns how many `*` tokens start `tokens`, or -1 if the tokens are not valid.
count_asterisks :: (tokens: [] CXToken) -> int {
    return gm_count_asterisks(tokens);
}

// Returns the index of the `)` that closes a parenthesis already opened before `start`, or -1.
//
// Nested parentheses are skipped. Returns -1 when `start` is the end of `tokens`; an out-of-range `start` is
// an error.
find_closing_paren_token_index :: (tokens: [] CXToken, start_index: int) -> int {
    return gm_closing_paren(tokens, start_index);
}

// Returns whether `a` and `b` hold the same tokens.
macro_values_are_equal :: (a: Macro_Value, b: Macro_Value) -> bool {
    return gm_macro_values_equal(a, b);
}

// Returns whether `a` and `b` are both macro literals with equal tokens.
macro_literals_are_equal :: (a: *Literal, b: *Literal) -> bool {
    if !a || !b || a.literal_kind != .MACRO || b.literal_kind != .MACRO return false;
    return gm_macro_values_equal(a.macro_value, b.macro_value);
}

// Returns whether the type of `decl` can be left out of the printed declaration.
//
// True when the declaration has a default value, defaults are being printed, and `type_to_print` is a bool.
can_omit_type :: (decl: *Declaration, flags: Print_Decl_Flags, type_to_print: *CType) -> bool {
    return (flags & .NO_ARGUMENT_DEFAULTS) == 0 && decl != null && decl.expression != null && is_bool_type(type_to_print);
}

// Builds a literal of kind `RAW_CPP_TOKENS` holding the text `raw_tokens_str`; returns null without an active
// session.
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

// Builds a macro literal from `macro_value`, copying its tokens.
create_literal :: (macro_value: Macro_Value, location: Location) -> *Literal {
    return gm_macro_literal(macro_value, location);
}

// Returns the declaration that `cur` refers to, or null if it is not a known declaration.
find_referenced_declaration :: (cur: CXCursor) -> *Declaration {
    return gm_referenced_declaration(cur);
}

// Builds an expression for a reference in `cur` to a constant declaration, or returns null.
//
// Only references to constants qualify; a static declaration without a value does not.
create_reference_expression :: (cur: CXCursor, location: Location) -> *Literal {
    return gm_reference_expression(cur, location);
}

// Wraps a declaration visitor so it also omits C++ functions.
//
// Returns the combined visitor and its data. A function whose foreign name differs from its name gets
// `OMIT_FROM_OUTPUT` and the visit stops there; other declarations go to `visitor` and then
// `visitor_with_data`.
// The data is allocated and not freed.
create_omit_cpp_functions_visitor :: (visitor: Declaration_Visitor, visitor_with_data: Declaration_Visitor_With_Data, visitor_data: *void) -> Declaration_Visitor_With_Data, *void {
    return gm_omit_visitor(visitor, visitor_with_data, visitor_data);
}

// Returns `type` with typedefs resolved to the first non-typedef type, or null on a typedef cycle.
find_underlying_type :: (type: *CType) -> *CType {
    return gm_underlying(type);
}

// Returns whether `ty`, after resolving typedefs, is a pointer.
is_pointer_type :: (ty: *CType) -> bool {
    t := gm_underlying(ty);
    return t != null && t.pointer_to != null;
}

// Returns whether `ty`, after resolving typedefs, is a function type.
is_function_type :: (ty: *CType) -> bool {
    t := gm_underlying(ty);
    return t != null && t.type_of_function != null;
}

// Returns whether `type`, after resolving typedefs, is an array with no known length.
is_incomplete_array :: (type: *CType) -> bool {
    t := gm_underlying(type);
    return t != null && t.array_element_type != null && t.array_element_count < 0;
}

// Returns whether `target_type` resolves to a signed integer type. False for non-integers.
is_target_type_signed :: (target_type: *CType = null) -> bool {
    t := gm_integer(target_type);
    return t != null && (t.number_flags & .SIGNED) != 0;
}

// Returns whether `target_type` resolves to an unsigned integer type. False for non-integers.
is_target_type_unsigned :: (target_type: *CType = null) -> bool {
    t := gm_integer(target_type);
    return t != null && (t.number_flags & .SIGNED) == 0;
}

// Returns whether `ty` resolves to a 16-byte integer type.
is_16byte_integer_type :: (ty: *CType) -> bool {
    t := gm_integer(ty);
    return t != null && t.size == 16;
}

// Returns whether `expression` is an integer literal with the value zero.
is_zero_int_expression :: (expression: *Expression) -> bool {
    if !expression || expression.kind != .LITERAL return false;

    lit := cast(*Literal) expression;
    if lit.literal_kind == .INTEGER return lit.int_value == 0;
    if lit.literal_kind == .UNSIGNED_INTEGER return lit.uint_value == 0;

    return false;
}

// Returns whether `type` is a pointer to a one-byte character type.
is_char_pointer_type :: (type: CType) -> bool {
    t := gm_underlying(*type);
    if !t || !t.pointer_to return false;
    child := gm_integer(t.pointer_to);
    return child != null && child.size == 1 && (child.number_flags & .CHAR) != 0;
}

// Returns whether `decl` has `extern` storage.
is_extern_declaration :: (decl: *Declaration) -> bool {
    return decl != null && decl.storage_class == .Extern;
}

// Returns whether `char` can start an identifier: an ASCII letter or `_`.
is_valid_identifier_start :: (char: u8) -> bool {
    return char == #char "_" || (char >= #char "a" && char <= #char "z") || (char >= #char "A" && char <= #char "Z");
}

// Returns whether `name` spells a C++ operator function, such as `operator==`.
is_operator_name :: (name: string) -> bool {
    return gm_operator(name);
}

// Returns whether `name` is an assignment operator function, `operator=` or a compound form such as
// `operator+=`.
is_assignment_operator :: (name: string) -> bool {
    return gm_assignment(name);
}

// Returns whether `decl` is a function whose name spells an operator.
is_operator :: (decl: *Declaration) -> bool {
    return decl != null && decl.kind == .FUNCTION && is_operator_name(decl.name);
}

// Returns whether the generator's target OS is macOS or iOS.
is_targeting_apple_os :: () -> bool {
    return TARGET_OS == .MACOS || TARGET_OS == .IOS;
}

// Returns whether `comment` was found on lines before its declaration, or spans more than one line.
is_prefix_comment :: (comment: Comment) -> bool {
    return gm_prefix_comment(comment);
}

// Returns whether `type` is a `va_list` typedef, directly or through other typedefs.
is_va_list :: (type: *CType) -> bool {
    return gm_va_list(type);
}

// Returns whether `decl` is an instance field of a struct, and the enclosing struct when that struct is
// anonymous.
//
// Static members and metadata do not count.
is_struct_field :: (decl: *Declaration) -> is_field: bool, anon_struct: *Struct {
    return gm_struct_field(decl);
}

// Returns the alignment in bytes of the field `decl` of type `type`, and whether that is the type's natural
// alignment.
//
// An explicit field alignment wins over the field's alignment, which wins over the type's.
get_type_alignment :: (type: *CType, decl: *Declaration) -> alignment: s64, is_natural_alignment: bool {
    return gm_alignment(type, decl);
}

// Returns the size in bytes of `type`, an integer, float, bool or enum primitive or an array of them, or -1.
//
// Returns -1 for pointers, structs, functions, void and incomplete arrays, and on overflow. `decl` is not
// used.
get_primitive_or_array_primitive_type_size :: (type: *CType, decl: *Declaration) -> int {
    return gm_primitive_size(type, decl);
}

// Returns whether `a` and `b` have the same name and argument types.
//
// With `ignore_first_argument` the first argument is not compared.
functions_are_equivalent :: (a: *Function, b: *Function, ignore_first_argument := false) -> bool {
    return gm_functions_equal(a, b, ignore_first_argument);
}

// Returns whether `declarations` contains a function equivalent to `fn`; see [functions_are_equivalent].
contains_equivalent_function :: (declarations: [] *Declaration, fn: *Function) -> bool {
    for decl: declarations if decl && decl.kind == .FUNCTION && functions_are_equivalent(cast(*Function) decl, fn) return true;
    return false;
}

// Looks for a declaration in `existing_declarations` that uses the same output name as `decl`.
//
// Returns whether one was found and which. With `allow_overloads`, functions with different argument types do
// not clash.
find_name_clash :: (existing_declarations: [] *Declaration, decl: *Declaration, allow_overloads: bool) -> found: bool, other: *Declaration {
    return gm_name_clash(existing_declarations, decl, allow_overloads);
}

// Creates a plain declaration named `name` of type `type`. The name and comment text are copied.
_create_declaration :: (name: string, type: *CType, location: Location = .{}, comment: Comment = .{}) -> *Declaration {
    return gm_declaration(name, type, location, comment);
}

// Creates a plain declaration like the one above, taking location and raw comment from `cur`.
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

// Creates an empty [Enum] named `name`.
create_enum :: (name: string) -> *Enum {
    return gm_enum(name);
}

// Returns a type for an empty struct named `name`.
create_empty_struct_type :: (name: string) -> *CType {
    return gm_empty_struct(name);
}

// Returns how many of `children` are namespace references.
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
// Prepares `generator` for the options in `options`; deinitialise it with [deinit_generator].
init_generator :: (using generator: *Generator, options: Generate_Bindings_Options) {
    gm_init_generator(generator, options);
}

// Releases the translation unit, index and tokens `generator` owns.
deinit_generator :: (generator: *Generator) {
    gm_deinit_generator(generator);
}

#scope_module
#load "../internal/developer-tools/generator-lifecycle.jai";
#load "../internal/developer-tools/generator-macro-evaluation.jai";
#load "../internal/developer-tools/generator-array-construction.jai";
#load "../internal/developer-tools/generator-construction-state.jai";
