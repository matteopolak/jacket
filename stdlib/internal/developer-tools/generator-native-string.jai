#scope_module
#import "Basic";

GM_String_Extent :: struct {
    found: bool;
    ambiguous: bool;
    units: int;
    width: int;
}

gm_string_source_type :: (native: CXType) -> bool {
    return native.kind == .ConstantArray || native.kind == .IncompleteArray || native.kind == .Pointer || native.kind == .ObjCObjectPointer;
}

gm_native_string_extent :: (cur: CXCursor, depth := 0) -> GM_String_Extent {
    result: GM_String_Extent;

    if depth > 128 {
        result.found = true;
        result.ambiguous = true;
        return result;
    }

    if cur.kind == .StringLiteral {
        type := clang_getCanonicalType(clang_getCursorType(cur));
        count := clang_getArraySize(type);
        width := clang_Type_getSizeOf(clang_getArrayElementType(type));

        result.found = true;
        result.ambiguous = count < 1 || width < 1;
        if !result.ambiguous {
            result.units = count - 1;
            result.width = width;
        }

        return result;
    }

    children := native_children(cur);

    defer array_free(children);

    for child: children {
        if clang_isExpression(child.kind) == 0 continue;

        nested := gm_native_string_extent(child, depth + 1);
        if !nested.found continue;
        if !result.found result = nested;
        else if nested.ambiguous || nested.units != result.units || nested.width != result.width result.ambiguous = true;
    }

    return result;
}
