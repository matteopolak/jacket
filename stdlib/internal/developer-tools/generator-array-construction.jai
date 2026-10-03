#scope_module
#import "Basic";

// SDK child traversal order is not array dimension order. Match native expression
// extents to the declarator's bracket ranges, ordered from outermost to innermost.
// Ambiguous macro/source extents remain diagnosed instead of assigning a wrong bound.
gm_array_bound_position :: (cur: CXCursor, child: CXCursor) -> int {
    if clang_isExpression(child.kind) == 0 return -1;

    unit := gm_token_unit();
    if !unit return -1;

    name_file, begin_file, end_file: CXFile;
    name_offset, begin_offset, end_offset: u32;
    extent := clang_getCursorExtent(child);

    clang_getExpansionLocation(clang_getCursorLocation(cur), *name_file, null, null, *name_offset);
    clang_getExpansionLocation(clang_getRangeStart(extent), *begin_file, null, null, *begin_offset);
    clang_getExpansionLocation(clang_getRangeEnd(extent), *end_file, null, null, *end_offset);
    if !name_file || begin_file != name_file || end_file != name_file || begin_offset >= end_offset return -1;

    tokens: *CXToken;
    count: u32;

    clang_tokenize(unit, clang_getCursorExtent(cur), *tokens, *count);
    defer if tokens clang_disposeTokens(unit, tokens, count);
    level := 0;
    opening: s64 = -1;
    dimension := 0;
    attribute := false;
    children := native_children(cur);

    defer array_free(children);

    for i: 0..cast(s64) count - 1 {
        file: CXFile;
        offset: u32;

        clang_getExpansionLocation(clang_getTokenLocation(unit, tokens[i]), *file, null, null, *offset);
        if file != name_file || offset <= name_offset continue;

        // Parameter declarators own a separate dimension domain. In a callback
        // returning an array pointer their brackets occur before return bounds.
        in_parameter := false;

        for parameter: children {
            if parameter.kind != .ParmDecl continue;

            parameter_range := clang_getCursorExtent(parameter);
            first_file, last_file: CXFile;
            first_offset, last_offset: u32;

            clang_getExpansionLocation(clang_getRangeStart(parameter_range), *first_file, null, null, *first_offset);
            clang_getExpansionLocation(clang_getRangeEnd(parameter_range), *last_file, null, null, *last_offset);
            if file == first_file && file == last_file && offset >= first_offset && offset < last_offset in_parameter = true;
        }

        if in_parameter continue;
        if level == 0 && gm_is_token(tokens[i], "=") break;

        if gm_is_token(tokens[i], "[") {
            if level == 0 {
                opening = i;
                attribute = i + 1 < count && gm_is_token(tokens[i + 1], "[");
            }

            level += 1;
        } else if gm_is_token(tokens[i], "]") && level > 0 {
            level -= 1;
            if level != 0 || attribute continue;

            if i > opening + 1 {
                first_file, last_file: CXFile;
                first_offset, last_offset: u32;

                clang_getExpansionLocation(clang_getTokenLocation(unit, tokens[opening + 1]), *first_file, null, null, *first_offset);
                clang_getExpansionLocation(clang_getRangeEnd(clang_getTokenExtent(unit, tokens[i - 1])), *last_file, null, null, *last_offset);
                if first_file == begin_file && last_file == end_file && first_offset == begin_offset && last_offset == end_offset return dimension;
            }

            dimension += 1;
        }
    }

    return -1;
}

gm_array_bound_expression :: (cur: CXCursor, dimension: int, supplied: [] CXCursor, use_supplied: bool) -> *Expression {
    owned: [..] CXCursor;

    defer array_free(owned);
    children := supplied;

    if !use_supplied {
        owned = native_children(cur);
        children = owned;
    }

    selected: CXCursor;
    found := false;

    for child: children {
        if gm_array_bound_position(cur, child) != dimension continue;

        if found {
            gm_construction_error("ambiguous native array-bound source extent");
            return null;
        }

        selected = child;
        found = true;
    }

    if !found {
        gm_construction_error("native dependent array bound has no matching source expression");
        return null;
    }

    return gm_build_expression(selected, cur);
}
