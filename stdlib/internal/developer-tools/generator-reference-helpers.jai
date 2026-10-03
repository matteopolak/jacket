#scope_module
GM_Table :: #import "Hash_Table";
#import "Basic";

gm_referenced_declaration :: (cur: CXCursor) -> *Declaration {
    if clang_Cursor_isNull(cur) != 0 return null;

    generator := gm_active_generator();
    if !generator return null;

    referenced := clang_getCursorReferenced(cur);
    if clang_Cursor_isNull(referenced) != 0 return null;

    canonical := clang_getCanonicalCursor(referenced);
    found, declaration := GM_Table.table_find(*generator.declarations_by_cursor, canonical);

    if !found return null;

    return declaration;
}

gm_reference_expression :: (cur: CXCursor, location: Location) -> *Literal {
    session := gm_current_session();
    if !session return null;

    declaration := gm_referenced_declaration(cur);
    if !declaration {
        return null;
    }

    is_const := (declaration.decl_flags & .IS_CONST) != 0 || (declaration.type && (declaration.type.flags & .CONST) != 0);
    if !is_const || (declaration.storage_class == .Static && !declaration.expression) return null;

    result := GM_Sessions.new_owned(session, Literal);
    result.location = location;

    if declaration.parent && declaration.parent.kind == .ENUM {
        result.literal_kind = .ENUM;
        result.enum_type = cast(*Enum) declaration.parent;
        result.enum_value = declaration;
    } else {
        result.literal_kind = .IDENTIFIER;
        result.identifier = declaration;
    }

    return result;
}
