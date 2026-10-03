fill_location :: (location: *Location, cur: CXCursor) {
    location.* = .{};
    clang_getSpellingLocation(clang_getCursorLocation(cur), *location.file, *location.line, *location.column, null);
}

get_clang_type_alignment :: (ty: CXType) -> s64 {
    return clang_Type_getAlignOf(ty);
}

create_null_literal :: () -> *Literal {
    literal := New(Literal);
    literal.literal_kind = .NULL;
    return literal;
}

#scope_file
#import "Basic";
