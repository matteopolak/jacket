// Public data contract from the specification; private algorithms are independent.

CXFile :: *void;

// A libclang token. Its fields are opaque and are read only through the `clang_*` token functions.
CXToken :: struct {
    int_data: [4] u32;
    ptr_data: *void;
}

#if OS == .WINDOWS {
    // The kind of a libclang token, as returned by `clang_getTokenKind`.
    //
    // The underlying type is `s32` on Windows and `u32` elsewhere, as for the C enum.
    CXTokenKind :: enum s32 {
        Punctuation :: 0;
        Keyword :: 1;
        Identifier :: 2;
        Literal :: 3;
        Comment :: 4;
    }
} else {
    // The kind of a libclang token, as returned by `clang_getTokenKind`.
    //
    // The underlying type is `s32` on Windows and `u32` elsewhere, as for the C enum.
    CXTokenKind :: enum u32 {
        Punctuation :: 0;
        Keyword :: 1;
        Identifier :: 2;
        Literal :: 3;
        Comment :: 4;
    }
}

#if OS == .WINDOWS {
    // The storage class of a declaration, as returned by `clang_Cursor_getStorageClass`.
    //
    // The underlying type is `s32` on Windows and `u32` elsewhere, as for the C enum.
    CX_StorageClass :: enum s32 {
        Invalid :: 0;
        None :: 1;
        Extern :: 2;
        Static :: 3;
        PrivateExtern :: 4;
        OpenCLWorkGroupLocal :: 5;
        Auto :: 6;
        Register :: 7;
    }
} else {
    // The storage class of a declaration, as returned by `clang_Cursor_getStorageClass`.
    //
    // The underlying type is `s32` on Windows and `u32` elsewhere, as for the C enum.
    CX_StorageClass :: enum u32 {
        Invalid :: 0;
        None :: 1;
        Extern :: 2;
        Static :: 3;
        PrivateExtern :: 4;
        OpenCLWorkGroupLocal :: 5;
        Auto :: 6;
        Register :: 7;
    }
}

// The negative values that the libclang layout queries return when a size, alignment or offset is not
// available.
//
// These come from `clang_Type_getSizeOf`, `clang_Type_getAlignOf` and `clang_Type_getOffsetOf`.
CXTypeLayoutError :: enum s32 {
    Invalid :: -1; // The type is invalid.
    Incomplete :: -2; // The type is incomplete.
    Dependent :: -3; // The type depends on a template parameter.
    NotConstantSize :: -4; // The size is not a constant.
    InvalidFieldName :: -5; // The field name given for an offset does not exist.
    Undeduced :: -6; // The type is an undeduced `auto` type.
}
