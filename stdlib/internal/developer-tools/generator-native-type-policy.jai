#scope_module
#import "Basic";

gm_native_extended_float :: (native: CXType, seen: *[..] CXType, depth := 0) -> bool {
    if depth >= 64 return false;

    native := clang_getCanonicalType(native);
    for previous: seen.* if clang_equalTypes(previous, native) != 0 return false;

    array_add(seen, native);
    if native.kind == .LongDouble return clang_Type_getSizeOf(native) > 8;
    if native.kind == .Pointer || native.kind == .LValueReference || native.kind == .RValueReference || native.kind == .BlockPointer return gm_native_extended_float(clang_getPointeeType(native), seen, depth + 1);
    if native.kind == .ConstantArray || native.kind == .IncompleteArray return gm_native_extended_float(clang_getArrayElementType(native), seen, depth + 1);

    if native.kind == .FunctionProto || native.kind == .FunctionNoProto {
        if gm_native_extended_float(clang_getResultType(native), seen, depth + 1) return true;
        for index: 0..clang_getNumArgTypes(native) - 1 if gm_native_extended_float(clang_getArgType(native, cast(u32) index), seen, depth + 1) return true;
    }

    if native.kind == .Record {
        children := native_children(clang_getTypeDeclaration(native));

        defer array_free(children);

        for child: children {
            if child.kind != .FieldDecl && child.kind != .CXXBaseSpecifier && child.kind != .CXXMethod && child.kind != .StructDecl && child.kind != .UnionDecl continue;
            if gm_native_extended_float(clang_getCursorType(child), seen, depth + 1) return true;
        }
    }

    return false;
}

gm_ignored_extended_declaration :: (cur: CXCursor) -> bool {
    kind := gm_cursor_kind(cur);
    if kind != .STRUCT && kind != .TYPEDEF return false;

    seen: [..] CXType;

    defer array_free(seen);

    return gm_native_extended_float(clang_getCursorType(cur), *seen);
}
