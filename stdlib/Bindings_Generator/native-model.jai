// Public data contract from the specification; private algorithms are independent.

CXFile :: *void;

CXToken :: struct {
    int_data: [4] u32;
    ptr_data: *void;
}

#if OS == .WINDOWS {
    CXTokenKind :: enum s32 {
        Punctuation :: 0;
        Keyword :: 1;
        Identifier :: 2;
        Literal :: 3;
        Comment :: 4;
    }
} else {
    CXTokenKind :: enum u32 {
        Punctuation :: 0;
        Keyword :: 1;
        Identifier :: 2;
        Literal :: 3;
        Comment :: 4;
    }
}

#if OS == .WINDOWS {
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

CXTypeLayoutError :: enum s32 {
    Invalid :: -1;
    Incomplete :: -2;
    Dependent :: -3;
    NotConstantSize :: -4;
    InvalidFieldName :: -5;
    Undeduced :: -6;
}
