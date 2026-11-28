// Sets `location` to the file, line and column where the cursor `cur` is spelled.
//
// Clears `location` first.
fill_location :: (location: *Location, cur: CXCursor) {
    location.* = .{};
    clang_getSpellingLocation(clang_getCursorLocation(cur), *location.file, *location.line, *location.column, null);
}

// Returns the alignment in bytes of `ty` as libclang reports it.
//
// A negative result is a [CXTypeLayoutError] value.
get_clang_type_alignment :: (ty: CXType) -> s64 {
    return clang_Type_getAlignOf(ty);
}

// Returns a new literal of kind `.NULL`, allocated with `New`.
create_null_literal :: () -> *Literal {
    literal := New(Literal);
    literal.literal_kind = .NULL;
    return literal;
}

#scope_file
#import "Basic";
